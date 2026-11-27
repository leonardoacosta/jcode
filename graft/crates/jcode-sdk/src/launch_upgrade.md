# crates/jcode-sdk/src/launch_upgrade.rs

- NEGOTIATION_TIMEOUT · constant · L9-L9 — const NEGOTIATION_TIMEOUT: Duration = Duration::from_secs(3);
- MAX_FRAME · constant · L10-L10 — const MAX_FRAME: usize = 64 * 1024;
- is_local_shared · function · L12-L17 — pub(super) fn is_local_shared(api: &Path, native: &Path, candidate: &Path) -> bool
- connect · function · L19-L23 — fn connect(path: &Path, timeout: Duration) -> io::Result<UnixStream>
- read_frame · function · L25-L47 — fn read_frame(stream: &mut UnixStream, deadline: Instant) -> io::Result<Option<Value>>
- Outcome · enum · L50-L55 — enum Outcome
- request · function · L57-L91 — fn request(socket: &Path, candidate: &Path, timeout: Duration) -> io::Result<Outcome>
- offer · function · L93-L127 — pub(super) fn offer(
- confirm_reconnect · function · L129-L144 — fn confirm_reconnect(socket: &Path, candidate: &Path, timeout: Duration) -> io::Result<()>
- retry_until_idle · function · L146-L160 — fn retry_until_idle(
- schedule_retry · function · L162-L200 — fn schedule_retry(socket: &Path, candidate: &Path)
- Key · type · L165-L165 — type Key = (std::path::PathBuf, std::path::PathBuf);
- PENDING · constant · L166-L166 — static PENDING: LazyLock<Mutex<HashSet<Key>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
- tests · module · L204-L204 — mod tests;
