# crates/jcode-command-risk/src/paths.rs

- PROTECTED_CREDENTIAL_SUBPATHS · constant · L15-L15 — const PROTECTED_CREDENTIAL_SUBPATHS: &[&str] = &[".ssh", ".gnupg", ".aws", ".kube", ".docker"];
- PROTECTED_HOME_SUBPATHS · constant · L20-L28 — const PROTECTED_HOME_SUBPATHS: &[&str] = &[
- PROTECTED_SYSTEM_PATHS · constant · L31-L52 — const PROTECTED_SYSTEM_PATHS: &[&str] = &[
- SYSTEM_PATHS_PROTECTED_RECURSIVELY · constant · L56-L59 — const SYSTEM_PATHS_PROTECTED_RECURSIVELY: &[&str] = &[
- ProtectedPaths · struct · L62-L62 — pub struct ProtectedPaths;
- home_subpaths · function · L65-L67 — pub fn home_subpaths() -> &'static [&'static str]
- credential_subpaths · function · L68-L70 — pub fn credential_subpaths() -> &'static [&'static str]
- system_paths · function · L71-L73 — pub fn system_paths() -> &'static [&'static str]
- recursive_system_paths · function · L74-L76 — pub fn recursive_system_paths() -> &'static [&'static str]
- expand · function · L83-L98 — pub fn expand(raw: &str, ctx: &RiskContext) -> PathBuf
- expand_known_prefix · function · L102-L125 — fn expand_known_prefix(raw: &str, ctx: &RiskContext) -> String
- normalize · function · L132-L147 — fn normalize(path: &Path) -> PathBuf
- is_catastrophic_target · function · L153-L190 — pub fn is_catastrophic_target(path: &Path, ctx: &RiskContext) -> bool
- classify_target · function · L193-L309 — pub fn classify_target(
- is_temp_path · function · L313-L317 — fn is_temp_path(path: &Path) -> bool
- paths_tests · module · L321-L321 — mod paths_tests;
