use super::*;

pub(super) fn should_refresh_token(status: StatusCode, body: &str) -> bool {
    // Fork safeguard: never refresh into revoked auth or exhausted quota.
    if matches!(
        jcode_provider_core::classify_provider_failure(body),
        jcode_provider_core::ProviderFailureKind::AuthRevoked
            | jcode_provider_core::ProviderFailureKind::QuotaExhausted
    ) {
        return false;
    }
    if status == StatusCode::UNAUTHORIZED {
        return true;
    }
    if status == StatusCode::FORBIDDEN {
        let lower = body.to_lowercase();
        return lower.contains("token")
            || lower.contains("expired")
            || lower.contains("unauthorized");
    }
    false
}

/// Terminal HTTP error text. Auth and quota failures carry the shared
/// kind-specific remedy; other statuses keep the raw provider error.
pub(super) fn format_http_error(status: StatusCode, body: &str) -> String {
    use jcode_provider_core::ProviderFailureKind::{AuthExpired, AuthRevoked, QuotaExhausted};
    let message = format!("OpenAI API error {status}: {body}");
    let kind = jcode_provider_core::classify_provider_failure(&message);
    if matches!(kind, AuthExpired | AuthRevoked | QuotaExhausted) {
        format!(
            "{message}\n{}",
            kind.remedy(jcode_provider_core::DualAuthProvider::OpenAI)
        )
    } else {
        message
    }
}

/// Terminal in-stream (WebSocket) error events get the same shared remedy as
/// terminal HTTP errors. Messages that already carry it are left unchanged.
pub(super) fn with_failure_remedy(event: StreamEvent) -> StreamEvent {
    use jcode_provider_core::ProviderFailureKind::{AuthExpired, AuthRevoked, QuotaExhausted};
    let StreamEvent::Error {
        message,
        retry_after_secs,
    } = event
    else {
        return event;
    };
    let kind = jcode_provider_core::classify_provider_failure(&message);
    let message = if matches!(kind, AuthExpired | AuthRevoked | QuotaExhausted) {
        let remedy = kind.remedy(jcode_provider_core::DualAuthProvider::OpenAI);
        let label = remedy.split_once(':').map_or(remedy, |(label, _)| label);
        if message.contains(&format!("{label}:")) {
            message
        } else {
            format!("{message}\n{remedy}")
        }
    } else {
        message
    };
    StreamEvent::Error {
        message,
        retry_after_secs,
    }
}

/// Refresh credentials at most once per logical request so a persistently
/// rejected token cannot trigger a refresh on every retry attempt.
pub(super) fn claim_auth_refresh(status: StatusCode, body: &str, attempted: &mut bool) -> bool {
    if *attempted || !should_refresh_token(status, body) {
        return false;
    }
    *attempted = true;
    true
}

pub(super) fn maybe_record_runtime_model_unavailable_from_stream_error(
    catalog_scope: &str,
    model: &str,
    message: &str,
) {
    let reason = classify_unavailable_model_error(StatusCode::BAD_REQUEST, message)
        .or_else(|| classify_unavailable_model_error(StatusCode::FORBIDDEN, message));

    if let Some(reason) = reason {
        jcode_base::provider::record_model_unavailable_for_scope(catalog_scope, model, &reason);
        jcode_base::logging::warn(&format!(
            "Recorded OpenAI model '{}' as unavailable from stream error: {}",
            model, reason
        ));
    }
}

pub(super) fn classify_unavailable_model_error(status: StatusCode, body: &str) -> Option<String> {
    let lower = body.to_ascii_lowercase();

    let mentions_model = lower.contains("model")
        || lower.contains("slug")
        || lower.contains("engine")
        || lower.contains("deployment");
    let unavailable = lower.contains("not available")
        || lower.contains("unavailable")
        || lower.contains("does not have access")
        || lower.contains("not enabled")
        || lower.contains("not found")
        || lower.contains("unknown model")
        || lower.contains("unsupported model")
        || lower.contains("invalid model");

    if !mentions_model || !unavailable {
        return None;
    }

    if status == StatusCode::NOT_FOUND
        || status == StatusCode::FORBIDDEN
        || status == StatusCode::BAD_REQUEST
        || status == StatusCode::UNPROCESSABLE_ENTITY
    {
        let trimmed = body.trim();
        let reason = if trimmed.is_empty() {
            format!("model denied by OpenAI API (status {})", status)
        } else {
            format!(
                "model denied by OpenAI API (status {}): {}",
                status, trimmed
            )
        };
        return Some(reason);
    }

    None
}

/// Check if an error is transient and should be retried
pub(in super::super) fn is_retryable_error(error_str: &str) -> bool {
    // Fork safeguard: delegate to the shared failure taxonomy so confirmed
    // quota/billing exhaustion, revoked auth, and permanent failures are never
    // retried, even when the message also mentions 429 or a transport fault.
    use jcode_provider_core::ProviderFailureKind;
    if error_str.contains("token refresh failed") {
        return false;
    }
    if error_str.contains("openai token refreshed, retrying")
        || error_str.contains("openai credentials changed before request, retrying")
    {
        return true;
    }
    match jcode_provider_core::classify_provider_failure(error_str) {
        ProviderFailureKind::QuotaExhausted
        | ProviderFailureKind::AuthRevoked
        | ProviderFailureKind::Permanent => false,
        ProviderFailureKind::Transient | ProviderFailureKind::RateLimited => true,
        ProviderFailureKind::AuthExpired | ProviderFailureKind::ModelUnavailable => {
            is_openai_transient_message(error_str)
        }
    }
}

/// OpenAI-specific transient signatures layered over the shared transport
/// classifier (issue #338). Only consulted when the shared taxonomy has not
/// already marked the failure as terminal.
fn is_openai_transient_message(error_str: &str) -> bool {
    jcode_provider_core::is_transient_transport_error(error_str)
        || error_str.contains("failed to send request to openai api")
        || error_str.contains("incomplete message")
        || error_str.contains("stream disconnected before completion")
        || error_str.contains("ended before message completion marker")
        || error_str.contains("falling back from websockets to https transport")
        || error_str.contains("500 internal server error")
        || error_str.contains("502 bad gateway")
        || error_str.contains("503 service unavailable")
        || error_str.contains("504 gateway timeout")
        || error_str.contains("overloaded")
        || error_str.contains("429 too many requests")
        || error_str.contains("rate limit")
        || error_str.contains("rate_limit")
        || error_str.contains("api_error")
        || error_str.contains("server_error")
        || error_str.contains("internal server error")
        || error_str.contains("an error occurred while processing your request")
        || error_str.contains("please include the request id")
}
