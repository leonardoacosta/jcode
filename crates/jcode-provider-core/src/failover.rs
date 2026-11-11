use serde::{Deserialize, Serialize};

const PROVIDER_FAILOVER_PROMPT_PREFIX: &str = "[jcode-provider-failover]";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderFailoverPrompt {
    pub from_provider: String,
    pub from_label: String,
    pub to_provider: String,
    pub to_label: String,
    /// Exact pool-approved model request. Old prompts without a route require manual consent.
    #[serde(default)]
    pub target_route: Option<String>,
    pub reason: String,
    pub estimated_input_chars: usize,
    pub estimated_input_tokens: usize,
}

impl ProviderFailoverPrompt {
    pub fn to_error_message(&self) -> String {
        let payload = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());
        format!(
            "{PROVIDER_FAILOVER_PROMPT_PREFIX}{payload}\n{} is unavailable; switching to {} would resend about {} input tokens (~{} chars).",
            self.from_label, self.to_label, self.estimated_input_tokens, self.estimated_input_chars,
        )
    }
}

pub fn parse_failover_prompt_message(message: &str) -> Option<ProviderFailoverPrompt> {
    let line = message.lines().next()?.trim();
    let json = line.strip_prefix(PROVIDER_FAILOVER_PROMPT_PREFIX)?;
    serde_json::from_str(json).ok()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailoverDecision {
    None,
    RetryNextProvider,
    RetryAndMarkUnavailable,
}

impl FailoverDecision {
    pub fn should_failover(self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn should_mark_provider_unavailable(self) -> bool {
        matches!(self, Self::RetryAndMarkUnavailable)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::RetryNextProvider => "retry-next-provider",
            Self::RetryAndMarkUnavailable => "retry-and-mark-unavailable",
        }
    }
}

fn contains_independent_status_code(haystack: &str, code: &str) -> bool {
    let haystack_bytes = haystack.as_bytes();
    let code_len = code.len();

    haystack.match_indices(code).any(|(start, _)| {
        let before_ok = start == 0 || !haystack_bytes[start - 1].is_ascii_digit();
        let end = start + code_len;
        let after_ok = end == haystack_bytes.len() || !haystack_bytes[end].is_ascii_digit();
        before_ok && after_ok
    })
}

/// Only explicit plan/window or balance exhaustion bypasses the TUI countdown.
/// HTTP 429, rate limits, billing errors and generic quota errors are ambiguous.
pub fn is_definitive_usage_exhaustion(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    [
        "usage_limit_reached",
        "usage limit reached",
        "usage limit has been reached",
        "you have reached your usage limit",
        "you have reached your weekly limit",
        "usage exhausted",
        "quota exhausted",
        "quota has been exhausted",
        "credits have run out",
        "credit balance is too low",
        "insufficient_quota",
        "insufficient credit",
        "weekly limit reached",
        "weekly limit has been reached",
        "weekly usage limit reached",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderFailureKind {
    AuthExpired,
    AuthRevoked,
    QuotaExhausted,
    RateLimited,
    ModelUnavailable,
    Transient,
    Permanent,
}

impl ProviderFailureKind {
    pub fn remedy(self, provider: crate::DualAuthProvider) -> &'static str {
        match self {
            Self::AuthRevoked => match provider {
                crate::DualAuthProvider::Anthropic => {
                    "AuthRevoked: Run `jcode login --provider claude`, then retry."
                }
                crate::DualAuthProvider::OpenAI => {
                    "AuthRevoked: Run `jcode login --provider openai`, then retry."
                }
            },
            Self::AuthExpired => {
                "AuthExpired: Authentication still failed after recovery. Retry with the same login or sign in again."
            }
            Self::QuotaExhausted => {
                "QuotaExhausted: Check this account's usage and billing before retrying."
            }
            Self::RateLimited => "RateLimited: Wait for the rate limit to reset, then retry.",
            Self::ModelUnavailable => {
                "ModelUnavailable: Check model availability for this account."
            }
            Self::Transient => {
                "Transient: Temporary service or transport failure. Retry with the same account."
            }
            Self::Permanent => "Permanent: Check the request and provider configuration.",
        }
    }
}

/// Classify evidence, not recovery permission. Callers still own route, identity
/// and refresh-once limits. Unknown failures are never automatically retried.
pub fn classify_provider_failure(message: &str) -> ProviderFailureKind {
    use ProviderFailureKind::*;
    let lower = message.to_ascii_lowercase();
    let has = |markers: &[&str]| markers.iter().any(|marker| lower.contains(marker));
    let code = |status| contains_independent_status_code(&lower, status);
    if has(&[
        "invalid_grant",
        "authrevoked:",
        "invalid_api_key",
        "invalid api key",
        "token revoked",
        "refresh token is invalid",
        "refresh token has been revoked",
        "refresh_token_reused",
        "refresh token was previously rejected",
    ]) {
        return AuthRevoked;
    }
    if has(&[
        "insufficient_quota",
        "quota exhausted",
        "quotaexhausted",
        "quota exceeded",
        "usage exhausted",
        "credit balance",
        "insufficient credit",
        "credits have run out",
        "billing",
        "payment required",
        "usage_limit_reached",
        "usage limit reached",
        "usage limit has been reached",
    ]) || code("402")
    {
        return QuotaExhausted;
    }
    let transport = crate::is_transient_transport_error(message)
        || has(&[
            "timeout",
            "timed out",
            "overloaded",
            "service unavailable",
            "bad gateway",
            "internal server error",
            "stream ended",
            "stream closed",
            "failed to send request to openai api",
            "incomplete message",
            "stream disconnected before completion",
            "ended before message completion marker",
            "falling back from websockets to https transport",
            "an error occurred while processing your request",
            "please include the request id",
        ])
        || ["408", "500", "502", "503", "504", "529"]
            .iter()
            .any(|status| code(status));
    if lower.contains("refresh") && transport {
        return Transient;
    }
    if lower.contains("model")
        && has(&[
            "not found",
            "unavailable",
            "not available",
            "does not have access",
            "unknown model",
            "unsupported model",
        ])
    {
        return ModelUnavailable;
    }
    if code("401")
        || has(&[
            "token has expired",
            "token expired",
            "access token expired",
            "invalid token",
            "authentication_error",
            "authexpired:",
            "unauthorized",
        ])
    {
        return AuthExpired;
    }
    if code("403")
        || has(&[
            "forbidden",
            "permission",
            "invalid request",
            "invalid_request",
            "not configured",
            "no credentials",
        ])
    {
        return Permanent;
    }
    if code("429")
        || has(&[
            "rate limit",
            "rate_limit",
            "rate-limited",
            "too many requests",
        ])
    {
        return RateLimited;
    }
    if transport || has(&["api_error", "server_error"]) {
        Transient
    } else {
        Permanent
    }
}

pub fn classify_failover_error_message(message: &str) -> FailoverDecision {
    let lower = message.to_ascii_lowercase();
    // Integrated runtimes own auth recovery. Do not turn their terminal remedy
    // or uncertain refresh failure into an account/provider substitution.
    if lower.contains("authrevoked:")
        || lower.contains("authexpired:")
        || (lower.contains("refresh")
            && classify_provider_failure(message) == ProviderFailureKind::Transient)
    {
        return FailoverDecision::None;
    }

    let request_size_or_context = [
        "context length",
        "context_length",
        "context window",
        "maximum context",
        "prompt is too long",
        "input is too long",
        "too many tokens",
        "max tokens",
        "token limit",
        "token_limit",
        "413 payload too large",
        "413 request entity too large",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
        || contains_independent_status_code(&lower, "413");
    if request_size_or_context {
        return FailoverDecision::RetryNextProvider;
    }

    let rate_or_quota = is_definitive_usage_exhaustion(message)
        || [
            "rate limit",
            "rate-limited",
            "too many requests",
            "quota",
            "credit balance",
            "credits have run out",
            "insufficient credit",
            "billing",
            "payment required",
            "usage tier",
            // Plan/usage-window exhaustion, e.g. OpenAI OAuth `usage_limit_reached`.
            "usage_limit",
            "usage limit",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        || contains_independent_status_code(&lower, "429")
        || contains_independent_status_code(&lower, "402");
    if rate_or_quota {
        return FailoverDecision::RetryAndMarkUnavailable;
    }

    let auth_or_access = [
        "access denied",
        "not accessible by integration",
        "provider unavailable",
        "provider not available",
        "provider is unavailable",
        "provider currently unavailable",
        "provider not configured",
        "credentials are not configured",
        "no credentials",
        "token exchange failed",
        // Cached permanent refresh rejection no longer carries the original
        // OAuth error code or HTTP status, but still needs account failover.
        "refresh token was previously rejected",
        "authentication failed",
        "unauthorized",
        "forbidden",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
        || contains_independent_status_code(&lower, "401")
        || contains_independent_status_code(&lower, "403");
    if auth_or_access {
        return FailoverDecision::RetryAndMarkUnavailable;
    }

    FailoverDecision::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitive_exhaustion_excludes_ambiguous_rate_limits() {
        for message in [
            "429 rate_limit_error: This request would exceed your account's rate limit. Please try again later.",
            "429 Too Many Requests",
            "quota exceeded",
            "billing error",
            "usage tier limit",
            "rate limit exceeded; retry in 60 seconds",
        ] {
            assert!(!is_definitive_usage_exhaustion(message), "{message}");
        }
        for message in [
            "usage_limit_reached: The usage limit has been reached",
            "You have reached your usage limit",
            "You have reached your weekly limit",
            "OAuth usage exhausted",
            "429 quota exhausted",
            "credits have run out",
            "insufficient_quota",
            "Your credit balance is too low",
        ] {
            assert!(is_definitive_usage_exhaustion(message), "{message}");
        }
    }

    #[test]
    fn provider_failure_taxonomy_distinguishes_recovery_actions() {
        use ProviderFailureKind::*;
        for (message, expected) in [
            ("HTTP 401 Unauthorized", AuthExpired),
            ("invalid_grant: refresh token revoked", AuthRevoked),
            ("HTTP 429 insufficient_quota", QuotaExhausted),
            ("HTTP 429 billing hard limit reached", QuotaExhausted),
            ("HTTP 429 Too Many Requests", RateLimited),
            ("HTTP 404 model not found", ModelUnavailable),
            ("HTTP 503 Service Unavailable", Transient),
            ("connection reset by peer", Transient),
            ("HTTP 400 invalid_request_error", Permanent),
            ("HTTP 403 forbidden rate limit", Permanent),
            ("model version 14290 failed", Permanent),
        ] {
            assert_eq!(classify_provider_failure(message), expected, "{message}");
        }
    }

    #[test]
    fn refresh_transport_failure_is_not_revocation() {
        assert_eq!(
            classify_provider_failure("token refresh after 401 failed: HTTP 503 timeout"),
            ProviderFailureKind::Transient
        );
        assert_eq!(
            classify_provider_failure("token refresh after 401 failed: invalid_grant"),
            ProviderFailureKind::AuthRevoked
        );
    }

    #[test]
    fn failure_remedies_name_exact_login_and_do_not_infer_revocation() {
        use crate::DualAuthProvider::{Anthropic, OpenAI};
        assert_eq!(
            ProviderFailureKind::AuthRevoked.remedy(Anthropic),
            "AuthRevoked: Run `jcode login --provider claude`, then retry."
        );
        assert_eq!(
            ProviderFailureKind::AuthRevoked.remedy(OpenAI),
            "AuthRevoked: Run `jcode login --provider openai`, then retry."
        );
        assert!(
            !ProviderFailureKind::Transient
                .remedy(OpenAI)
                .contains("login")
        );
        assert!(
            ProviderFailureKind::QuotaExhausted
                .remedy(OpenAI)
                .contains("billing")
        );
        assert_eq!(
            classify_provider_failure("refresh token is invalid"),
            ProviderFailureKind::AuthRevoked
        );
        assert_eq!(
            classify_provider_failure("HTTP 403 token expired"),
            ProviderFailureKind::AuthExpired
        );
    }

    #[test]
    fn classified_auth_remedies_never_offer_another_identity() {
        for error in [
            "invalid_grant: AuthRevoked: Run `jcode login --provider claude`, then retry.",
            "HTTP 401 Unauthorized\nAuthExpired: Authentication still failed after recovery.",
            "token refresh after 401 failed: 503 Service Unavailable",
        ] {
            assert_eq!(
                classify_failover_error_message(error),
                FailoverDecision::None,
                "{error}"
            );
        }
    }

    #[test]
    fn failover_prompt_roundtrips_from_error_message() {
        let prompt = ProviderFailoverPrompt {
            from_provider: "claude".to_string(),
            from_label: "Anthropic".to_string(),
            to_provider: "openai".to_string(),
            to_label: "OpenAI".to_string(),
            target_route: Some("openai-oauth@work:gpt-5.4".into()),
            reason: "rate limit".to_string(),
            estimated_input_chars: 1200,
            estimated_input_tokens: 300,
        };

        let parsed = parse_failover_prompt_message(&prompt.to_error_message()).expect("prompt");
        assert_eq!(parsed, prompt);
    }

    #[test]
    fn legacy_prompt_without_target_route_requires_manual_selection() {
        let wire = r#"[jcode-provider-failover]{"from_provider":"claude","from_label":"Anthropic","to_provider":"openai","to_label":"OpenAI","reason":"rate limit","estimated_input_chars":1200,"estimated_input_tokens":300}"#;
        let prompt = parse_failover_prompt_message(wire).expect("legacy prompt");
        assert_eq!(prompt.target_route, None);
    }

    #[test]
    fn classifier_marks_rate_limits_unavailable() {
        assert_eq!(
            classify_failover_error_message("429 Too Many Requests"),
            FailoverDecision::RetryAndMarkUnavailable
        );
    }

    #[test]
    fn classifier_marks_usage_limit_unavailable() {
        assert_eq!(
            classify_failover_error_message(
                "usage_limit_reached: The usage limit has been reached"
            ),
            FailoverDecision::RetryAndMarkUnavailable
        );
    }

    #[test]
    fn classifier_marks_cached_refresh_rejection_unavailable() {
        for message in [
            "Claude OAuth refresh token was previously rejected by Anthropic and cannot be refreshed. Run `jcode login --provider claude` to mint a fresh token.",
            "claude refresh token was previously rejected by the provider and cannot be refreshed. Run `jcode login`.",
            "Failed to load credentials: REFRESH TOKEN WAS PREVIOUSLY REJECTED by the provider",
        ] {
            assert_eq!(
                classify_failover_error_message(message),
                FailoverDecision::RetryAndMarkUnavailable,
                "cached permanent refresh rejection must permit account failover: {message}",
            );
        }
        // Generic refresh failures are not evidence of invalid credentials.
        for message in [
            "Claude token refresh failed: 503 Service Unavailable",
            "Claude token refresh failed: connection reset by peer",
            "Claude token refresh failed: request timed out",
        ] {
            assert_eq!(
                classify_failover_error_message(message),
                FailoverDecision::None,
                "transient refresh failure must not poison an account: {message}",
            );
        }
    }

    #[test]
    fn classifier_retries_context_errors_without_marking_unavailable() {
        assert_eq!(
            classify_failover_error_message("context length exceeded"),
            FailoverDecision::RetryNextProvider
        );
    }

    #[test]
    fn classifier_ignores_embedded_status_digits() {
        assert_eq!(
            classify_failover_error_message("model version 4130 failed"),
            FailoverDecision::None
        );
    }
}
