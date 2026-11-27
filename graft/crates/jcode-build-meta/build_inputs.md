# crates/jcode-build-meta/build_inputs.rs

- immutable_source_verified · function · L8-L33 — pub fn immutable_source_verified(repo_root: &Path, fingerprint: &str) -> bool
- base_tag_offset · function · L37-L55 — pub fn base_tag_offset(description: &str, base: (u32, u32, u32)) -> Option<u32>
- release_semver · function · L58-L74 — pub fn release_semver(value: &str) -> Option<String>
- prerelease_identifier · function · L76-L81 — fn prerelease_identifier(value: &str) -> bool
- semver_core · function · L85-L98 — pub fn semver_core(value: &str) -> Option<(u32, u32, u32)>
- numeric_component · function · L100-L105 — fn numeric_component(value: &str) -> bool
- metadata_rerun_inputs · function · L109-L127 — pub fn metadata_rerun_inputs(path: Option<&OsStr>) -> Vec<String>
- tests · module · L130-L227 — mod tests
- hosted_base_tag_distance · function · L134-L149 — fn hosted_base_tag_distance()
- release_identity_accepts_stable_and_prereleases · function · L152-L188 — fn release_identity_accepts_stable_and_prereleases()
- package_and_build_prereleases_preserve_numeric_core · function · L191-L199 — fn package_and_build_prereleases_preserve_numeric_core()
- no_snapshot_still_tracks_metadata_environment · function · L202-L211 — fn no_snapshot_still_tracks_metadata_environment()
- snapshot_contents_are_tracked_without_git · function · L214-L220 — fn snapshot_contents_are_tracked_without_git()
- caller_fingerprint_without_namespace_is_not_attestation · function · L223-L226 — fn caller_fingerprint_without_namespace_is_not_attestation()
