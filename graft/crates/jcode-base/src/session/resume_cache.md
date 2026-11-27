# crates/jcode-base/src/session/resume_cache.rs

- FileFingerprint · struct · L14-L17 — struct FileFingerprint
- fingerprint · function · L19-L28 — fn fingerprint(path: &Path) -> std::io::Result<Option<FileFingerprint>>
- SessionFingerprint · struct · L31-L34 — pub(crate) struct SessionFingerprint
- read · function · L37-L57 — pub(crate) fn read(path: &Path) -> Option<Self>
- Entry · struct · L60-L65 — pub(crate) struct Entry
- Cache · struct · L67-L71 — pub(super) struct Cache
- load · function · L74-L89 — pub fn load() -> Self
- get · function · L91-L131 — pub fn get(&mut self, id: &str, path: &Path) -> Option<&Entry>
- finish · function · L133-L148 — pub fn finish(self)
- backfill · function · L153-L171 — pub(crate) fn backfill() -> anyhow::Result<()>
