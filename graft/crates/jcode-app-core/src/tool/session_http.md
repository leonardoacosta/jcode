# crates/jcode-app-core/src/tool/session_http.rs

- CAPACITY · constant · L10-L10 — const CAPACITY: usize = 32;
- KEYS · constant · L11-L20 — const KEYS: [&str; 8] = [
- ProxyConfig · struct · L23-L27 — struct ProxyConfig
- fmt · function · L29-L31 — fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
- resolve · function · L34-L65 — fn resolve(
- apply · function · L66-L97 — fn apply(&self, mut builder: reqwest::ClientBuilder) -> Result<reqwest::ClientBuilder>
- Profile · enum · L101-L105 — pub(crate) enum Profile
- Entry · struct · L106-L110 — struct Entry
- CACHE · constant · L111-L111 — static CACHE: LazyLock<Mutex<VecDeque<Entry>>> = LazyLock::new(|| Mutex::new(VecDeque::new()));
- builder · function · L113-L132 — fn builder(profile: Profile) -> reqwest::ClientBuilder
- client · function · L134-L136 — pub(crate) fn client(session: &str, profile: Profile) -> Result<reqwest::Client>
- clients · function · L138-L145 — pub(crate) fn clients(session: &str, profiles: &[Profile]) -> Result<Vec<reqwest::Client>>
- daemon_env · function · L147-L158 — fn daemon_env(key: &str) -> Option<String>
- select · function · L160-L196 — fn select(env: &HashMap<String, String>, profile: Profile) -> Result<reqwest::Client>
- prune · function · L200-L209 — pub(crate) fn prune(envs: &HashMap<String, HashMap<String, String>>)
- transport_error · function · L211-L214 — pub(crate) fn transport_error(_: reqwest::Error) -> anyhow::Error
- tests · module · L218-L218 — mod tests;
