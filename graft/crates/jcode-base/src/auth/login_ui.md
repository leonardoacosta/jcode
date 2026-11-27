# crates/jcode-base/src/auth/login_ui.rs

- LoginPresenter · interface · L13-L27 — pub trait LoginPresenter: Send + Sync
- qr_section · function · L17-L23 — fn qr_section(
- read_secret_line · function · L26-L26 — fn read_secret_line(&self) -> Result<String>;
- PRESENTER · constant · L29-L29 — static PRESENTER: OnceLock<Box<dyn LoginPresenter>> = OnceLock::new();
- install · function · L32-L38 — pub fn install(presenter: Box<dyn LoginPresenter>)
- qr_section · function · L40-L49 — pub fn qr_section(
- read_secret_line · function · L51-L60 — pub fn read_secret_line() -> Result<String>
