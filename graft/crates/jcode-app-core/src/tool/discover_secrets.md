# crates/jcode-app-core/src/tool/discover_secrets.rs

- contains_recognizable_secret · function · L12-L37 — pub(super) fn contains_recognizable_secret(value: &str) -> bool
- contains_prefixed_secret · function · L39-L67 — fn contains_prefixed_secret(value: &str) -> bool
- SECRET_PREFIXES · constant · L40-L57 — const SECRET_PREFIXES: &[&str] = &[
- contains_credential_assignment · function · L69-L96 — fn contains_credential_assignment(lower: &str) -> bool
- LABELS · constant · L70-L80 — const LABELS: &[&str] = &[
- contains_bearer_token · function · L98-L105 — fn contains_bearer_token(lower: &str) -> bool
- contains_email_address · function · L107-L118 — fn contains_email_address(value: &str) -> bool
- contains_ssn · function · L120-L132 — fn contains_ssn(value: &str) -> bool
- contains_credential_url · function · L134-L146 — fn contains_credential_url(value: &str) -> bool
- contains_international_phone_number · function · L148-L159 — fn contains_international_phone_number(value: &str) -> bool
- looks_like_jwt · function · L161-L163 — fn looks_like_jwt(token: &str) -> bool
- contains_payment_card_sequence · function · L165-L169 — fn contains_payment_card_sequence(value: &str) -> bool
- looks_like_payment_card · function · L171-L193 — fn looks_like_payment_card(candidate: &str) -> bool
