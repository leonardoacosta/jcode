# src/cli/login/google.rs

- DOWNLOAD_WAIT · constant · L15-L15 — const DOWNLOAD_WAIT: Duration = Duration::from_secs(600);
- login_google_flow · function · L17-L77 — pub(super) async fn login_google_flow(
- choose_gmail_tier · function · L79-L110 — fn choose_gmail_tier(
- print_summary · function · L112-L147 — fn print_summary(services: &[GoogleService], tokens: &auth::google::GoogleTokens) -> Result<()>
- prompt · function · L149-L155 — fn prompt(label: &str) -> Result<String>
- pause · function · L157-L160 — fn pause(label: &str) -> Result<()>
- run_client_setup · function · L164-L187 — fn run_client_setup(services: &[GoogleService], no_browser: bool) -> Result<()>
- guided_setup · function · L189-L235 — fn guided_setup(services: &[GoogleService], no_browser: bool) -> Result<()>
- prepare_project · function · L240-L316 — fn prepare_project(services: &[GoogleService], no_browser: bool) -> Result<Option<String>>
- api_names · function · L318-L324 — fn api_names(services: &[GoogleService]) -> String
- open_page · function · L326-L329 — fn open_page(url: &str, no_browser: bool)
- import_client · function · L331-L354 — fn import_client(parsed: &setup::ClientJson, services: &[GoogleService]) -> Result<()>
- import_from_path · function · L356-L370 — fn import_from_path(services: &[GoogleService]) -> Result<()>
- paste_credentials · function · L372-L392 — fn paste_credentials() -> Result<()>
- default_pending_google_services · function · L394-L396 — pub(super) fn default_pending_google_services() -> Vec<GoogleService>
- import_google_client_json · function · L401-L437 — pub(super) fn import_google_client_json(source: &str, quiet: bool) -> Result<()>
- default_google_services · function · L441-L449 — pub(super) fn default_google_services(options: &LoginOptions) -> Vec<GoogleService>
- prompt_google_services · function · L451-L477 — fn prompt_google_services(existing: Option<&[GoogleService]>) -> Result<Vec<GoogleService>>
