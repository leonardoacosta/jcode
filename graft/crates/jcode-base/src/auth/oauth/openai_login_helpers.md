# crates/jcode-base/src/auth/oauth/openai_login_helpers.rs

- openai_auth_url · function · L3-L5 — pub fn openai_auth_url(redirect_uri: &str, challenge: &str, state: &str) -> String
- openai_auth_url_with_prompt · function · L7-L26 — pub fn openai_auth_url_with_prompt(
- callback_listener_available · function · L28-L35 — pub fn callback_listener_available(port: u16) -> bool
- exchange_openai_code_at_url · function · L37-L80 — pub(super) async fn exchange_openai_code_at_url(
- TokenResponse · struct · L63-L68 — struct TokenResponse
- exchange_openai_code · function · L82-L88 — pub async fn exchange_openai_code(
- exchange_openai_callback_input · function · L90-L101 — pub async fn exchange_openai_callback_input(
- start_openai_device_login · function · L103-L105 — pub async fn start_openai_device_login() -> Result<OpenAiDeviceCode>
- start_openai_device_login_at · function · L107-L170 — pub async fn start_openai_device_login_at(base: &str) -> Result<OpenAiDeviceCode>
- UserCodeResponse · struct · L109-L117 — struct UserCodeResponse
- poll_openai_device_login_at · function · L173-L203 — pub async fn poll_openai_device_login_at(
- TokenResponse · struct · L179-L182 — struct TokenResponse
- finish_openai_device_login · function · L206-L208 — pub async fn finish_openai_device_login(device: &OpenAiDeviceCode) -> Result<OAuthTokens>
- finish_openai_device_login_at · function · L210-L232 — pub async fn finish_openai_device_login_at(
- login_openai · function · L235-L303 — pub async fn login_openai(no_browser: bool) -> Result<OAuthTokens>
