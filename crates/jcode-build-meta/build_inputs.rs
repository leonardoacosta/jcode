use std::ffi::OsStr;
use std::path::Path;

/// Only the opt-in wrapper's namespace-private, read-only source mount attests
/// compiler inputs. Environment flags alone never establish provenance.
pub fn immutable_source_verified(repo_root: &Path, fingerprint: &str) -> bool {
    if repo_root != Path::new("/src")
        || fingerprint.len() != 64
        || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return false;
    }
    let Ok(mounts) = std::fs::read_to_string("/proc/self/mountinfo") else {
        return false;
    };
    let readonly = |path: &str| {
        mounts.lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.get(4) == Some(&path)
                && fields
                    .get(5)
                    .is_some_and(|options| options.split(',').any(|v| v == "ro"))
        })
    };
    readonly("/src")
        && readonly("/jcode-immutable")
        && std::fs::read_to_string("/jcode-immutable").is_ok_and(|data| {
            data.lines()
                .any(|line| line == format!("fingerprint={fingerprint}"))
        })
}

/// Snapshot contents are build inputs even when no Git checkout is available.
pub fn metadata_rerun_inputs(path: Option<&OsStr>) -> Vec<String> {
    let mut inputs = vec![
        "cargo:rerun-if-env-changed=JCODE_BUILD_METADATA_FILE".to_owned(),
        "cargo:rerun-if-env-changed=JCODE_BUILD_CHANGELOG_RAW".to_owned(),
        "cargo:rerun-if-env-changed=JCODE_BUILD_SOURCE_FINGERPRINT".to_owned(),
        "cargo:rerun-if-env-changed=JCODE_BUILD_SOURCE_VERIFIED".to_owned(),
        "cargo:rerun-if-changed=build_inputs.rs".to_owned(),
    ];
    if Path::new("/jcode-immutable").exists() {
        inputs.push("cargo:rerun-if-changed=/jcode-immutable".to_owned());
    }
    if let Some(path) = path {
        inputs.push(format!(
            "cargo:rerun-if-changed={}",
            Path::new(path).display()
        ));
    }
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_snapshot_still_tracks_metadata_environment() {
        let inputs = metadata_rerun_inputs(None);
        assert!(inputs.contains(&"cargo:rerun-if-env-changed=JCODE_BUILD_METADATA_FILE".into()));
        assert!(inputs.contains(&"cargo:rerun-if-env-changed=JCODE_BUILD_CHANGELOG_RAW".into()));
        assert!(
            inputs.contains(&"cargo:rerun-if-env-changed=JCODE_BUILD_SOURCE_FINGERPRINT".into())
        );
        assert!(inputs.contains(&"cargo:rerun-if-env-changed=JCODE_BUILD_SOURCE_VERIFIED".into()));
        assert_eq!(inputs.len(), 5);
    }

    #[test]
    fn snapshot_contents_are_tracked_without_git() {
        let inputs = metadata_rerun_inputs(Some(OsStr::new(".jcode-release-metadata/build.txt")));
        assert!(
            inputs.contains(&"cargo:rerun-if-changed=.jcode-release-metadata/build.txt".into())
        );
        assert_eq!(inputs.len(), 6);
    }

    #[test]
    fn caller_fingerprint_without_namespace_is_not_attestation() {
        assert!(!immutable_source_verified(Path::new("."), &"a".repeat(64)));
        assert!(!immutable_source_verified(Path::new("/src"), "invalid"));
    }
}
