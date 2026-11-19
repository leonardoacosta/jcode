use std::ffi::OsStr;
use std::path::Path;

/// Only the opt-in wrapper's namespace-private, read-only source mount attests
/// compiler inputs. Environment flags alone never establish provenance.
// Shared with build.rs through `#[path]`; the library half never calls it.
#[allow(dead_code)]
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

/// Recover the base-tag distance forwarded by a source-only hosted build.
#[allow(dead_code)]
pub fn base_tag_offset(description: &str, base: (u32, u32, u32)) -> Option<u32> {
    let tag = format!("v{}.{}.{}", base.0, base.1, base.2);
    let description = description.trim();
    if description == tag {
        return Some(0);
    }
    let suffix = description.strip_prefix(&format!("{tag}-"))?;
    let (count, hash) = suffix.split_once("-g")?;
    if count.is_empty()
        || !count.bytes().all(|byte| byte.is_ascii_digit())
        || hash.is_empty()
        || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    count.bytes().try_fold(0u32, |offset, digit| {
        offset.checked_mul(10)?.checked_add(u32::from(digit - b'0'))
    })
}

/// Accept release identities without accepting arbitrary environment text.
pub fn release_semver(value: &str) -> Option<String> {
    let value = value.trim().strip_prefix('v').unwrap_or(value.trim());
    let (core, prerelease) = match value.split_once('-') {
        Some((core, suffix)) => (core, Some(suffix)),
        None => (value, None),
    };
    let components: Vec<_> = core.split('.').collect();
    if components.len() != 3 || components.iter().any(|part| !numeric_component(part)) {
        return None;
    }
    if let Some(suffix) = prerelease
        && !suffix.split('.').all(prerelease_identifier)
    {
        return None;
    }
    Some(value.to_string())
}

fn prerelease_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        // Preserve the release identity policy that numeric identifiers start at 1.
        && (!value.bytes().all(|byte| byte.is_ascii_digit()) || !value.starts_with('0'))
}

/// Extract the numeric core of a package or build version, ignoring its prerelease.
#[allow(dead_code)] // Used by build.rs, not runtime release identity parsing.
pub fn semver_core(value: &str) -> Option<(u32, u32, u32)> {
    let trimmed = value.trim().trim_start_matches('v');
    let core = trimmed.split_once('-').map_or(trimmed, |(core, _)| core);
    let mut parts = core.split('.');
    let (Ok(major), Ok(minor), Ok(patch)) = (
        parts.next()?.parse(),
        parts.next()?.parse(),
        parts.next()?.parse(),
    ) else {
        // Invalid and overflowing components are not semantic versions.
        return None;
    };
    Some((major, minor, patch))
}

fn numeric_component(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
        && value.parse::<u64>().is_ok()
}

/// Snapshot contents are build inputs even when no Git checkout is available.
#[allow(dead_code)] // Used by build.rs, but not by the runtime identity parser.
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
    fn hosted_base_tag_distance() {
        let base = (0, 99, 1);
        assert_eq!(base_tag_offset("v0.99.1-42-gabc1234", base), Some(42));
        assert_eq!(base_tag_offset("v0.99.1-0-gabc1234", base), Some(0));
        assert_eq!(base_tag_offset("v0.99.1", base), Some(0));
        for invalid in [
            "v0.98.1-42-gabc1234",
            "abc1234",
            "v0.99.1-x-gabc",
            "v0.99.1-42-g",
            "v0.99.1-42-gxyz",
            "v0.99.1-4294967296-gabc",
        ] {
            assert_eq!(base_tag_offset(invalid, base), None, "{invalid}");
        }
    }

    #[test]
    fn release_identity_accepts_stable_and_prereleases() {
        for value in [
            "1.2.3",
            "1.0.0-preview.1",
            "1.0.0-preview.123",
            "1.2.3-rc.1",
            "0.99.2-dist.7",
            "1.1.0-preview.1",
            "1.0.0-preview.1-dev",
            "1.2.3-alpha-beta",
            "1.2.3-01a",
            "1.2.3-18446744073709551616",
        ] {
            assert_eq!(release_semver(value).as_deref(), Some(value));
        }
        assert_eq!(
            release_semver(" v1.0.0-preview.2 ").as_deref(),
            Some("1.0.0-preview.2")
        );
        for value in [
            "1.2",
            "1.2.3.4",
            "1.0.0-preview.0",
            "1.0.0-preview.01",
            "1.2.3-rc.0",
            "1.2.3-dist.01",
            "1.2.3-",
            "1.2.3-rc..1",
            "1.2.3-rc.1.",
            "1.2.3-rc_1",
            "1.2.3-rc.1\nINJECT=1",
            "01.0.0",
            "1.0.0\nINJECT=1",
        ] {
            assert_eq!(release_semver(value), None, "{value}");
        }
    }

    #[test]
    fn package_and_build_prereleases_preserve_numeric_core() {
        for value in ["0.95.1", "0.95.1-rc.7", " v0.95.1-preview.2 "] {
            assert_eq!(semver_core(value), Some((0, 95, 1)), "{value}");
        }
        assert_eq!(semver_core("1.2.3-rc.1"), Some((1, 2, 3)));
        for value in ["1.2", "1.2.beta-rc.1", "4294967296.1.2-rc.1"] {
            assert_eq!(semver_core(value), None, "{value}");
        }
    }

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
