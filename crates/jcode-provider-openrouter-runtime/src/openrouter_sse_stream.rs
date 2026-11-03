use super::*;
use futures::Stream;
use jcode_provider_openrouter::stream::OpenRouterStream;
use std::pin::Pin;

const GATEWAY_RETRY_BUDGET: std::time::Duration = std::time::Duration::from_secs(120);

#[derive(Debug)]
struct GatewayEligibilityError {
    retry_delay: Option<std::time::Duration>,
    observed: std::time::Instant,
}

impl GatewayEligibilityError {
    fn remaining(&self) -> Option<std::time::Duration> {
        self.retry_delay
            .map(|delay| delay.saturating_sub(self.observed.elapsed()))
    }
}

impl std::fmt::Display for GatewayEligibilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.remaining() {
            Some(delay) => write!(
                formatter,
                "Gateway next-eligible hint: {}s remaining; retries are bounded to 120s elapsed.",
                delay.as_secs()
            ),
            None => write!(
                formatter,
                "Gateway supplied no valid retry deadline; automatic retry stopped."
            ),
        }
    }
}

impl std::error::Error for GatewayEligibilityError {}

#[derive(Debug)]
struct ErrorBodyReadTimeout {
    status: u16,
    endpoint: String,
    model: String,
    timeout: std::time::Duration,
}

impl std::fmt::Display for ErrorBodyReadTimeout {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Timed out reading HTTP {} error body from {} for model {} after {:?}",
            self.status, self.endpoint, self.model, self.timeout
        )
    }
}

impl std::error::Error for ErrorBodyReadTimeout {}

async fn read_http_error_body(
    response: reqwest::Response,
    status: u16,
    endpoint: &str,
    model: &str,
    tx: &mpsc::Sender<Result<StreamEvent>>,
    timeout: std::time::Duration,
) -> Result<Option<String>> {
    tokio::select! {
        _ = tx.closed() => Ok(None),
        body = tokio::time::timeout(timeout, jcode_base::util::http_error_body(response, "HTTP error")) => {
            body.map(Some).map_err(|_| anyhow::Error::new(ErrorBodyReadTimeout {
                status,
                endpoint: endpoint.to_owned(),
                model: model.to_owned(),
                timeout,
            }))
        }
    }
}

fn gateway_retry_delay_within_budget(
    delay: std::time::Duration,
    elapsed: std::time::Duration,
) -> bool {
    delay < GATEWAY_RETRY_BUDGET.saturating_sub(elapsed)
}

fn retryable_attempt_allowed(attempt: u32, max_retries: u32, gateway_retry: bool) -> bool {
    attempt + 1
        < if gateway_retry {
            max_retries.min(8)
        } else {
            max_retries
        }
}

fn gateway_retry_allowed(
    saw_output: bool,
    attempt: u32,
    max_retries: u32,
    delay: Option<std::time::Duration>,
    elapsed: std::time::Duration,
) -> bool {
    !saw_output
        && attempt + 1 < max_retries.min(8)
        && delay.is_some_and(|delay| gateway_retry_delay_within_budget(delay, elapsed))
}

async fn wait_for_retry_or_close(
    tx: &mpsc::Sender<Result<StreamEvent>>,
    delay: std::time::Duration,
) -> bool {
    tokio::select! {
        _ = tx.closed() => false,
        _ = tokio::time::sleep(delay) => true,
    }
}
fn is_gateway_eligibility_failure(status: u16, body: &str) -> bool {
    if status != 503 {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return false;
    };
    matches!(
        value.pointer("/error/code").and_then(Value::as_str),
        Some("ALL_TARGETS_SKIPPED" | "PROVIDER_CIRCUIT_OPEN" | "CIRCUIT_OPEN")
    ) || value
        .pointer("/diagnostics/terminalReason")
        .and_then(Value::as_str)
        == Some("all_targets_skipped")
}

fn local_endpoint_troubleshooting_hint(api_base: &str, model: &str) -> &'static str {
    let lower = api_base.to_ascii_lowercase();
    if lower.contains("localhost:11434") || lower.contains("127.0.0.1:11434") {
        return "Ollama hint: make sure `ollama serve` is running, the model is installed with `ollama pull <model>`, and run jcode with an installed model, for example `jcode --provider ollama --model llama3.2 run 'hello'`. If replies ignore earlier turns, Ollama is truncating the prompt to its serving context: restart it with a larger window, e.g. `OLLAMA_CONTEXT_LENGTH=65536 ollama serve`.";
    }

    if lower.contains("localhost:1234") || lower.contains("127.0.0.1:1234") {
        return "LM Studio hint: start the Local Server in LM Studio, load a chat model, and run jcode with the exact model id shown by LM Studio's /v1/models endpoint.";
    }

    if lower.contains("localhost") || lower.contains("127.0.0.1") || lower.contains("[::1]") {
        return "Local endpoint hint: make sure the server is running, the base URL includes /v1, the selected model is loaded, and the server supports streaming POST /chat/completions.";
    }

    let _ = model;
    "Hint: check network connectivity, DNS/TLS, that the base URL includes the API version (usually /v1), and that the model exists on the provider."
}

/// Hint for a request the server answered with an error status. The network
/// is working (a response came back), so connectivity advice would mislead:
/// point at the key, the account balance, or the model instead.
fn http_status_hint(status: u16, api_base: &str, model: &str) -> &'static str {
    let endpoint_hint = local_endpoint_troubleshooting_hint(api_base, model);
    let is_local = !endpoint_hint.starts_with("Hint: check network");
    match status {
        401 | 403 => {
            "Hint: the provider rejected the API key. Check that the key is valid and allowed to use this model, or switch to another provider with /model."
        }
        402 => {
            "Hint: this is a billing limit, not a network problem. The key's balance or token allowance is used up: raise the limit or top up in the provider's dashboard, use another key, or switch to another provider with /model."
        }
        // Local servers (Ollama, LM Studio) answer 404 for a model that is
        // not installed or loaded, which their own hint already explains.
        404 if !is_local => {
            "Hint: the endpoint or model was not found. Check that the base URL includes the API version (usually /v1) and that the model exists on the provider."
        }
        // The provider answered, so this is server-side overload or failure,
        // not connectivity (#1596). These are retried before surfacing.
        500..=599 if !is_local => {
            "Hint: the provider is overloaded or having a temporary server problem, not a network problem. jcode already retried; try again shortly or switch to another provider with /model."
        }
        _ => endpoint_hint,
    }
}

// ============================================================================
// SSE Stream Parser
// ============================================================================

#[expect(
    clippy::too_many_arguments,
    reason = "stream helpers thread transport, auth, request, event channel, and pin state explicitly"
)]
pub(super) async fn run_stream_with_retries(
    client: Client,
    api_base: String,
    auth: ProviderAuth,
    send_openrouter_headers: bool,
    conversation_id: String,
    request: Value,
    azure_responses: bool,
    tx: mpsc::Sender<Result<StreamEvent>>,
    provider_pin: Arc<Mutex<Option<ProviderPin>>>,
    model: String,
    responses_api: bool,
    openai_responses_api: bool,
) {
    let mut last_error = None;
    let mut next_retry_delay = None;
    let mut gateway_retry = false;
    let started = std::time::Instant::now();
    let config = jcode_base::config::config();
    let max_retries = config.provider.max_retries.max(1);
    let retry_backoff_cap =
        std::time::Duration::from_secs(config.provider.retry_backoff_cap_secs.max(1));

    for attempt in 0..max_retries {
        if attempt > 0 {
            let requested = jcode_provider_core::retry_after::retry_delay(
                attempt,
                RETRY_BASE_DELAY_MS,
                next_retry_delay.take(),
            );
            let delay = if gateway_retry {
                requested
            } else {
                requested.min(retry_backoff_cap)
            };
            if gateway_retry && !gateway_retry_delay_within_budget(delay, started.elapsed()) {
                let terminal_error = last_error.take().unwrap_or_else(|| {
                    anyhow::anyhow!(
                        "Gateway eligibility deadline exceeds the remaining retry budget"
                    )
                });
                if tx.send(Err(terminal_error)).await.is_err() {
                    jcode_base::logging::info(
                        "Gateway retry budget exhausted after the consumer disconnected",
                    );
                }
                return;
            }
            if gateway_retry && !wait_for_retry_or_close(&tx, delay).await {
                return;
            }
            if !gateway_retry {
                tokio::time::sleep(delay).await;
            }
            jcode_base::logging::info(&format!(
                "Retrying API request using {} (attempt {}/{})",
                auth.label(),
                attempt + 1,
                max_retries
            ));
        }

        jcode_base::logging::info(&format!(
            "API stream attempt {}/{} over HTTPS transport (model: {}, endpoint: {}, auth: {})",
            attempt + 1,
            max_retries,
            model,
            api_base,
            auth.label()
        ));

        // Track whether this attempt streams replay-visible output so a
        // mid-stream transport fault can roll the partial output back on the
        // consumer before the retry replays the response from the top.
        let (attempt_tx, attempt_guard) =
            jcode_provider_core::attempt_tracker::track_attempt_output(tx.clone());

        // Retries use a fresh unpooled client: the fault that broke attempt N
        // (e.g. TLS BadRecordMac from a corrupting middlebox) may also have
        // poisoned other idle pooled connections opened through the same path,
        // so reusing the shared pool can fail identically. A fresh client
        // guarantees a brand-new TCP+TLS connection.
        let attempt_client = if attempt == 0 {
            client.clone()
        } else {
            jcode_provider_core::fresh_transport_client()
        };

        match stream_response(
            attempt_client,
            api_base.clone(),
            auth.clone(),
            send_openrouter_headers,
            &conversation_id,
            request.clone(),
            azure_responses,
            attempt_tx,
            Arc::clone(&provider_pin),
            model.clone(),
            started + GATEWAY_RETRY_BUDGET,
            gateway_retry.then(|| started + GATEWAY_RETRY_BUDGET),
            responses_api,
            openai_responses_api,
        )
        .await
        {
            Ok(()) => {
                let _ = attempt_guard.finish().await;
                return;
            }
            Err(e) => {
                let saw_output = attempt_guard.finish().await;
                // Full anyhow chain ({:#}) so a `.context(...)`-wrapped transport
                // cause (e.g. TLS BadRecordMac) is visible to the classifier.
                if e.downcast_ref::<ErrorBodyReadTimeout>().is_some() {
                    if tx.send(Err(e)).await.is_err() {
                        jcode_base::logging::info(
                            "HTTP error-body read timed out after the consumer disconnected",
                        );
                    }
                    return;
                }
                if let Some(eligibility) = e.downcast_ref::<GatewayEligibilityError>() {
                    let delay = eligibility.remaining();
                    if !gateway_retry_allowed(
                        saw_output,
                        attempt,
                        max_retries,
                        delay,
                        started.elapsed(),
                    ) {
                        if tx.send(Err(e)).await.is_err() {
                            jcode_base::logging::info(
                                "Gateway retry stopped after the consumer disconnected",
                            );
                        }
                        return;
                    }
                    gateway_retry = true;
                    next_retry_delay = delay;
                    last_error = Some(e);
                    continue;
                }
                if retryable_stream_failure(&e)
                    && retryable_attempt_allowed(attempt, max_retries, gateway_retry)
                {
                    if saw_output {
                        // Partial output already reached the consumer; tell it
                        // to discard the partial attempt so the retried
                        // response replays cleanly instead of duplicating.
                        jcode_base::logging::warn(&format!(
                            "Transient API error after partial output; rolling back partial attempt and retrying: {}",
                            e
                        ));
                        let _ = tx
                            .send(Ok(StreamEvent::RetryRollback {
                                attempt: attempt + 2,
                                max: max_retries,
                            }))
                            .await;
                    } else {
                        jcode_base::logging::info(&format!(
                            "Transient API error, will retry: {}",
                            e
                        ));
                    }
                    next_retry_delay = jcode_provider_core::retry_after::retry_after_from_error(&e);
                    last_error = Some(e);
                    continue;
                }

                let _ = tx.send(Err(e)).await;
                return;
            }
        }
    }

    if let Some(e) = last_error {
        let _ = tx
            .send(Err(anyhow::anyhow!(
                "Failed after {} retries: {}",
                max_retries,
                e
            )))
            .await;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "stream helpers thread transport, auth, request, event channel, and pin state explicitly"
)]
async fn stream_response(
    client: Client,
    api_base: String,
    auth: ProviderAuth,
    send_openrouter_headers: bool,
    conversation_id: &str,
    request: Value,
    azure_responses: bool,
    tx: mpsc::Sender<Result<StreamEvent>>,
    provider_pin: Arc<Mutex<Option<ProviderPin>>>,
    model: String,
    error_body_deadline: std::time::Instant,
    gateway_deadline: Option<std::time::Instant>,
    responses_api: bool,
    openai_responses_api: bool,
) -> Result<()> {
    use jcode_message_types::ConnectionPhase;
    let _ = tx
        .send(Ok(StreamEvent::ConnectionPhase {
            phase: ConnectionPhase::SendingRequest,
        }))
        .await;
    let connect_start = std::time::Instant::now();
    let stream_idle_timeout = jcode_base::provider::stream_idle_timeout();

    let route = if azure_responses || responses_api || openai_responses_api {
        "responses"
    } else {
        "chat/completions"
    };
    let request_kind = if azure_responses {
        "Azure OpenAI Responses"
    } else if openai_responses_api {
        "OpenAI Responses"
    } else if responses_api {
        "OpenCode Go Responses"
    } else {
        "OpenAI-compatible chat"
    };
    let url = format!("{}/{}", api_base, route);
    let mut req = apply_kimi_coding_agent_headers(
        auth.apply(
            client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Accept-Encoding", "identity"),
        )
        .await?,
        &api_base,
        Some(&model),
    );

    if send_openrouter_headers {
        req = req
            .header("HTTP-Referer", "https://github.com/jcode")
            .header("X-Title", "jcode");
    }
    req = apply_opencode_session_header(req, &api_base, conversation_id);
    req = apply_grok_cli_turn_headers(req, &auth, &model, conversation_id);

    let response_timeout = gateway_deadline
        .map(|deadline| {
            deadline
                .saturating_duration_since(std::time::Instant::now())
                .min(stream_idle_timeout)
        })
        .unwrap_or(stream_idle_timeout);
    let response = jcode_provider_core::transport::send_with_initial_response_timeout(
        req.json(&request),
        response_timeout,
    )
    .await
    .with_context(|| {
        let hint = if azure_responses {
            "Hint: check the Azure deployment and that the endpoint supports /openai/v1/responses."
        } else {
            local_endpoint_troubleshooting_hint(&api_base, &model)
        };
        format!(
            "Failed to send {request_kind} request\n  endpoint: {}\n  model: {}\n  auth: {}\n{}",
            url,
            model,
            auth.label(),
            hint
        )
    })?;

    let connect_ms = connect_start.elapsed().as_millis();
    jcode_base::logging::info(&format!(
        "HTTP connection established in {}ms (status={})",
        connect_ms,
        response.status()
    ));

    if !response.status().is_success() {
        let status = response.status();
        let retry_after = jcode_provider_core::retry_after::retry_after(response.headers());
        let gateway_delay =
            jcode_provider_core::retry_after::uncapped_retry_after_delay(response.headers());
        let observed = std::time::Instant::now();
        let error_body_timeout = if status.as_u16() == 503 || gateway_deadline.is_some() {
            error_body_deadline
                .saturating_duration_since(std::time::Instant::now())
                .min(stream_idle_timeout)
        } else {
            stream_idle_timeout
        };
        let body = match read_http_error_body(
            response,
            status.as_u16(),
            &url,
            &model,
            &tx,
            error_body_timeout,
        )
        .await
        {
            Ok(Some(body)) => body,
            Ok(None) => return Ok(()),
            Err(error) => return Err(error),
        };
        if is_gateway_eligibility_failure(status.as_u16(), &body) {
            return Err(anyhow::Error::new(GatewayEligibilityError {
                retry_delay: gateway_delay,
                observed,
            }).context(format!("Gateway eligibility request failed\\n  endpoint: {url}\\n  model: {model}\\n  status: {status}\\n  response: {body}")));
        }
        let hint = if azure_responses {
            match status.as_u16() {
                401 | 402 | 403 | 500..=599 => http_status_hint(status.as_u16(), &api_base, &model),
                429 => {
                    "Hint: Azure rate limited the request; jcode will retry before returning this error."
                }
                _ => {
                    "Hint: check the Azure deployment and that the endpoint supports /openai/v1/responses."
                }
            }
        } else {
            http_status_hint(status.as_u16(), &api_base, &model)
        };
        return Err(jcode_provider_core::retry_after::error_with_retry_after(
            format!(
                "{request_kind} request failed\n  endpoint: {}\n  model: {}\n  auth: {}\n  status: {}\n  response: {}\n{}",
                url,
                model,
                auth.label(),
                status,
                body,
                hint
            ),
            retry_after,
        ));
    }

    let _ = tx
        .send(Ok(StreamEvent::ConnectionPhase {
            phase: ConnectionPhase::WaitingForResponse,
        }))
        .await;

    let mut stream: Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> =
        if azure_responses || openai_responses_api {
            Box::pin(jcode_provider_openai::stream::OpenAIResponsesStream::new(
                response.bytes_stream(),
            ))
        } else if responses_api {
            Box::pin(super::opencode_go_responses::ResponsesStream::new(
                response.bytes_stream(),
            ))
        } else {
            Box::pin(OpenRouterStream::new(
                response.bytes_stream(),
                model.clone(),
                provider_pin,
            ))
        };

    // Idle timeout between streamed chunks. Configurable so slow reasoning
    // models (e.g. DeepSeek) that think silently for minutes before emitting
    // tokens don't trip a premature timeout (issue #196). Resolved from
    // `[provider] stream_idle_timeout_secs` / `JCODE_STREAM_IDLE_TIMEOUT_SECS`,
    // defaulting to 180s. Shared with the native provider paths (issue #434).
    let idle_timeout_secs = stream_idle_timeout.as_secs();

    let mut saw_message_end = false;
    loop {
        let event = match tokio::time::timeout(stream_idle_timeout, stream.next()).await {
            Ok(Some(Ok(event))) => event,
            Ok(Some(Err(e))) => anyhow::bail!(
                "OpenAI-compatible stream error\n  endpoint: {}\n  model: {}\n  auth: {}\n  error: {}",
                url,
                model,
                auth.label(),
                e
            ),
            Ok(None) => break, // stream ended normally
            Err(_) => {
                jcode_base::logging::warn(&format!(
                    "OpenRouter SSE stream timed out (no data for {}s)",
                    idle_timeout_secs
                ));
                anyhow::bail!(
                    "OpenAI-compatible stream timeout\n  endpoint: {}\n  model: {}\n  auth: {}\n  timeout: no data received for {} seconds\n{}",
                    url,
                    model,
                    auth.label(),
                    idle_timeout_secs,
                    local_endpoint_troubleshooting_hint(&api_base, &model)
                );
            }
        };
        if azure_responses || openai_responses_api {
            if let StreamEvent::Error {
                message,
                retry_after_secs,
            } = &event
            {
                if azure_responses {
                    return Err(azure_response_failure(message, *retry_after_secs));
                }
                anyhow::bail!("Responses request failed: {}", message);
            }
            saw_message_end |= matches!(event, StreamEvent::MessageEnd { .. });
        }
        if tx.send(Ok(event)).await.is_err() {
            return Ok(());
        }
        if saw_message_end {
            break;
        }
    }

    if (azure_responses || openai_responses_api) && !saw_message_end {
        if azure_responses {
            return Err(anyhow::Error::new(AzureResponseFailure {
                retryable: true,
                source: anyhow::anyhow!(
                    "Azure OpenAI Responses stream ended before response.completed"
                ),
            }));
        }
        anyhow::bail!("Responses stream ended before message completion marker");
    }

    Ok(())
}

/// An Azure protocol failure is not necessarily a transient transport failure.
/// Keep its retry decision separate from the legacy string classifier, and
/// retain the shared retry-hint error as a source for deadline extraction.
#[derive(Debug)]
struct AzureResponseFailure {
    retryable: bool,
    source: anyhow::Error,
}

impl std::fmt::Display for AzureResponseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.source, formatter)
    }
}

impl std::error::Error for AzureResponseFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

fn retryable_stream_failure(error: &anyhow::Error) -> bool {
    if let Some(failure) = error.downcast_ref::<AzureResponseFailure>() {
        return failure.retryable;
    }
    // Preserve the existing classifier and full transport context for every
    // non-Azure failure, including HTTP errors and reqwest body errors.
    is_retryable_error(&format!("{error:#}").to_lowercase())
}

fn azure_response_failure(message: &str, retry_after_secs: Option<u64>) -> anyhow::Error {
    let lower = message.to_ascii_lowercase();
    // The shared Responses parser preserves error type/code in the message.
    // Permanent failures must win even if their prose mentions a timeout or
    // overload, or the server supplies a retry hint.
    let permanent = [
        "invalid_request",
        "invalid_argument",
        "content_filter",
        "content_policy",
        "authentication",
        "permission_denied",
        "insufficient_quota",
        "billing",
        "unauthorized",
        "forbidden",
        "model_not_found",
        "deployment_not_found",
        "tool argument snapshot differs",
    ]
    .iter()
    .any(|marker| lower.contains(marker));
    let transient = [
        "rate_limit",
        "rate limit",
        "server_error",
        "internal_error",
        "internal server error",
        "service_unavailable",
        "overloaded",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || jcode_provider_core::is_transient_transport_error(&lower);
    let retry_after = retry_after_secs.and_then(|seconds| {
        jcode_provider_core::retry_after::retry_after_body(
            &serde_json::json!({"retry_after_seconds": seconds}).to_string(),
        )
    });
    anyhow::Error::new(AzureResponseFailure {
        retryable: !permanent && (transient || retry_after.is_some()),
        source: jcode_provider_core::retry_after::error_with_retry_after(
            format!("Azure OpenAI Responses stream error: {message}"),
            retry_after,
        ),
    })
}

#[cfg(test)]
#[path = "azure_retry_tests.rs"]
mod azure_retry_tests;

/// Extract the HTTP status code reported in a formatted provider error string.
///
/// Error strings produced in this module embed the status as `status: <code>`
/// (e.g. `status: 402 Payment Required`). The input may be lowercased before
/// it reaches here, so matching is case-insensitive.
fn parsed_http_status(error_str: &str) -> Option<u16> {
    let lower = error_str.to_ascii_lowercase();
    let idx = lower.find("status:")?;
    let rest = lower[idx + "status:".len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 3 {
        digits.parse().ok()
    } else {
        None
    }
}

fn is_retryable_error(error_str: &str) -> bool {
    // Explicit non-retryable HTTP statuses take precedence over the loose
    // substring heuristics below. These are deterministic client-side failures
    // (auth, billing, malformed request) where retrying is futile and just
    // burns time/credits. 429 (rate limit) is classified explicitly so it does
    // not depend on provider-specific body wording.
    match parsed_http_status(error_str) {
        Some(400 | 401 | 402 | 403 | 404 | 405 | 406 | 422) => return false,
        // 429 rate limit, and every 5xx: the server answered but is overloaded
        // or failing on its side. This includes non-standard codes such as the
        // 529 "overloaded" some OpenAI-compatible providers send (#1596).
        Some(429 | 500..=599) => return true,
        _ => {}
    }

    jcode_provider_core::is_transient_transport_error(error_str)
        || error_str.contains("stream error")
        || error_str.contains("eof")
        || error_str.contains("5")
            && (error_str.contains("50")
                || error_str.contains("502")
                || error_str.contains("503")
                || error_str.contains("504")
                || error_str.contains("internal server error"))
        || error_str.contains("overloaded")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_failure_after_gateway_eligibility_keeps_eight_attempt_limit() {
        let configured_max = 12;
        let eligibility_body = r#"{"error":{"code":"ALL_TARGETS_SKIPPED"}}"#;
        let gateway_retry = is_gateway_eligibility_failure(503, eligibility_body);
        let generic_500 = "chat request failed\\n  status: 500 internal server error";
        assert!(is_retryable_error(generic_500));
        assert!(retryable_attempt_allowed(1, configured_max, gateway_retry));
        assert!(retryable_attempt_allowed(6, configured_max, gateway_retry));
        assert!(!retryable_attempt_allowed(7, configured_max, gateway_retry));

        // Ordinary retries retain the configured limit when no gateway
        // eligibility response has activated the bounded policy.
        let ordinary_retries = false;
        assert!(retryable_attempt_allowed(
            7,
            configured_max,
            ordinary_retries
        ));
    }
    #[test]
    fn gateway_eligibility_requires_matching_503_response() {
        let skipped = r#"{"error":{"code":"ALL_TARGETS_SKIPPED"}}"#;
        assert!(is_gateway_eligibility_failure(503, skipped));
        assert!(!is_gateway_eligibility_failure(429, skipped));
        assert!(!is_gateway_eligibility_failure(503, "unavailable"));
    }

    #[test]
    fn gateway_retry_waits_until_full_hint_and_stays_inside_120_second_budget() {
        assert!(gateway_retry_delay_within_budget(
            std::time::Duration::from_secs(119),
            std::time::Duration::ZERO
        ));
        assert!(!gateway_retry_delay_within_budget(
            GATEWAY_RETRY_BUDGET,
            std::time::Duration::ZERO
        ));
        assert!(!gateway_retry_delay_within_budget(
            std::time::Duration::from_secs(119),
            std::time::Duration::from_secs(1)
        ));
        assert!(gateway_retry_allowed(
            false,
            0,
            8,
            Some(std::time::Duration::from_secs(119)),
            std::time::Duration::ZERO
        ));
        assert!(!gateway_retry_allowed(
            true,
            0,
            8,
            Some(std::time::Duration::from_secs(1)),
            std::time::Duration::ZERO
        ));
    }

    #[tokio::test]
    async fn gateway_retry_wait_is_cancelled_when_consumer_closes() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        assert!(!wait_for_retry_or_close(&tx, std::time::Duration::from_secs(120)).await);
    }
    #[test]
    fn local_endpoint_hint_mentions_ollama_actions() {
        let hint = local_endpoint_troubleshooting_hint("http://localhost:11434/v1", "llama3.2");
        assert!(hint.contains("ollama serve"));
        assert!(hint.contains("ollama pull"));
        assert!(hint.contains("--provider ollama"));
    }

    #[test]
    fn local_endpoint_hint_mentions_lm_studio_server() {
        let hint = local_endpoint_troubleshooting_hint("http://127.0.0.1:1234/v1", "local-model");
        assert!(hint.contains("LM Studio"));
        assert!(hint.contains("Local Server"));
        assert!(hint.contains("/v1/models"));
    }

    /// #1596: an OpenAI-compatible provider answering 529 "heavy usage" (or
    /// any other 5xx) must be retried, not fail the turn on the first try.
    #[test]
    fn every_5xx_status_including_529_is_retryable() {
        let body = r#"data: {"error":{"message":"We're experiencing heavy usage right now, please try again in a moment.","type":"server_error"}}"#;
        for status in [
            "500 internal server error",
            "502 bad gateway",
            "503 service unavailable",
            "520 <unknown status code>",
            "529 <unknown status code>",
        ] {
            let err = format!(
                "openai-compatible chat request failed\n  endpoint: https://compat.example.test/v1/chat/completions\n  model: glm-5.3\n  auth: compat_api_key\n  status: {status}\n  response: {body}"
            )
            .to_lowercase();
            assert!(is_retryable_error(&err), "{status} must be retried");
        }
    }

    #[test]
    fn server_error_hint_points_at_provider_not_network() {
        let hint = http_status_hint(529, "https://compat.example.test/v1", "glm-5.3");
        assert!(hint.contains("overloaded"), "{hint}");
        assert!(!hint.contains("check network"), "{hint}");
    }

    #[test]
    fn parsed_http_status_extracts_code() {
        assert_eq!(
            parsed_http_status("status: 402 payment required"),
            Some(402)
        );
        assert_eq!(parsed_http_status("  status:404 not found"), Some(404));
        assert_eq!(parsed_http_status("no status here"), None);
        // Embedded numbers elsewhere must not be misread as a status.
        assert_eq!(parsed_http_status("you requested 65536 tokens"), None);
    }

    #[test]
    fn payment_required_is_not_retryable() {
        let err = "openai-compatible chat request failed\n  endpoint: \
            https://openrouter.ai/api/v1/chat/completions\n  model: openai/gpt-5.4\n  \
            auth: openrouter_api_key\n  status: 402 payment required\n  response: \
            {\"error\":{\"message\":\"this request requires more credits, or fewer \
            max_tokens. you requested up to 65536 tokens, but can only afford 34424\"}}";
        assert!(!is_retryable_error(err));
    }

    #[test]
    fn client_errors_are_not_retryable() {
        for status in [400u16, 401, 402, 403, 404, 405, 406, 422] {
            let err = format!("chat request failed\n  status: {status} client error");
            assert!(
                !is_retryable_error(&err),
                "status {status} should not be retryable"
            );
        }
    }

    #[test]
    fn server_errors_remain_retryable() {
        assert!(is_retryable_error(
            "chat request failed\n  status: 503 service unavailable"
        ));
        assert!(is_retryable_error(
            "chat request failed\n  status: 500 internal server error"
        ));
        // Provider overload messages should still be retried.
        assert!(is_retryable_error("overloaded"));
    }

    #[test]
    fn http_429_is_retryable_without_rate_limit_words_in_body() {
        assert!(is_retryable_error(
            "chat request failed\n  status: 429 unknown\n  response: {}"
        ));
    }

    /// A 402 means the server answered: the hint must name the billing limit,
    /// not send the user to check their network.
    #[test]
    fn status_hint_names_billing_for_402_not_network() {
        let hint = http_status_hint(402, "https://apiduck.servepics.com/v1", "glm-5.3");
        assert!(hint.contains("billing limit"), "{hint}");
        assert!(hint.contains("/model"), "{hint}");
        assert!(!hint.contains("network connectivity"), "{hint}");

        for status in [401u16, 403] {
            let hint = http_status_hint(status, "https://api.example.com/v1", "m");
            assert!(hint.contains("API key"), "{status}: {hint}");
            assert!(!hint.contains("network connectivity"), "{status}: {hint}");
        }
        assert!(http_status_hint(404, "https://api.example.com/v1", "m").contains("/v1"));
        // Local servers keep their own advice (e.g. `ollama pull` for a 404).
        assert!(
            http_status_hint(404, "http://localhost:11434/v1", "llama3.2").contains("ollama pull")
        );
        // Server errors keep the endpoint-specific advice.
        assert!(
            http_status_hint(503, "http://localhost:11434/v1", "llama3.2").contains("ollama serve")
        );
    }

    #[tokio::test]
    async fn stalled_http_error_body_is_bounded_by_deadline() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            use std::io::{Read, Write};
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 2048];
                let _ = stream.read(&mut request).unwrap();
                stream
                    .write_all(
                        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 1024\r\nConnection: close\r\n\r\n{\"error\":",
                    )
                    .unwrap();
                stream.flush().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });

        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let url = format!("http://{address}/v1/chat/completions");
        let response = client.get(&url).send().await.unwrap();
        assert_eq!(response.status().as_u16(), 503);

        let (tx, _rx) = mpsc::channel(1);
        let timeout = std::time::Duration::from_millis(25);
        let error = read_http_error_body(response, 503, "fixture", "model", &tx, timeout)
            .await
            .unwrap_err();
        assert!(error.downcast_ref::<ErrorBodyReadTimeout>().is_some());
        assert!(error.to_string().contains("HTTP 503 error body"));

        let response = client.get(&url).send().await.unwrap();
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        assert!(
            read_http_error_body(response, 503, "fixture", "model", &tx, timeout)
                .await
                .unwrap()
                .is_none()
        );
        server.join().unwrap();
    }
}
