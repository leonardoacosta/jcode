# crates/jcode-build-meta/build.rs

- build_inputs · module · L5-L5 — mod build_inputs;
- main · function · L18-L206 — fn main()
- repo_root · function · L209-L219 — fn repo_root() -> PathBuf
- root_package_version · function · L222-L242 — fn root_package_version(repo_root: &Path) -> Option<String>
- parse_semver · function · L244-L246 — fn parse_semver(value: &str) -> Option<(u32, u32, u32)>
- explicit_build_semver_override · function · L248-L252 — fn explicit_build_semver_override() -> Option<String>
- resolve_build_semver · function · L254-L287 — fn resolve_build_semver(base_version: (u32, u32, u32)) -> Result<String, String>
- commits_since_base_tag · function · L292-L298 — fn commits_since_base_tag(base_version: (u32, u32, u32)) -> Option<u32>
- env_or_metadata_or_git · function · L300-L311 — fn env_or_metadata_or_git<const N: usize>(
- git_output · function · L313-L323 — fn git_output<const N: usize>(repo_root: &Path, args: [&str; N]) -> Option<String>
- metadata_value · function · L325-L354 — fn metadata_value(key: &str) -> Option<String>
