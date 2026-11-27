# crates/jcode-sdk/src/errors.rs

- ErrorKind · enum · L11-L52 — pub enum ErrorKind
- code · function · L57-L86 — pub fn code(&self) -> &'static str
- Error · struct · L91-L94 — pub struct Error
- new · function · L97-L102 — pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self
- code · function · L105-L107 — pub fn code(&self) -> &'static str
- fmt · function · L111-L113 — fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
- from · function · L119-L131 — fn from(error: jcode_harness_api::FrameError) -> Self
- Result · type · L135-L135 — pub type Result<T> = std::result::Result<T, Error>;
