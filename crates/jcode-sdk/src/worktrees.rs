//! Blocking local Git worktree operations. Run these on a background thread in a UI.
//!
//! Creation requires user-scoped policy and explicit task authority. No staged,
//! unstaged or untracked changes are copied. Matching clean owned checkouts may
//! be reused. Git must be available on PATH.

use anyhow::{Context, Result, bail, ensure};
use std::path::Path;
use std::process::Command;

/// A worktree reported by Git. Branches are full refs (e.g. `refs/heads/main`).
/// Bare repositories have an empty `head`. Flag-only lock/prune reasons are
/// represented by `Some(String::new())`. Paths must be valid UTF-8.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    pub path: String,
    pub branch: Option<String>,
    pub head: String,
    pub bare: bool,
    pub detached: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
}

fn git(working_dir: &str, args: &[&str]) -> Result<Vec<u8>> {
    // Checkout and even status can run clean/smudge/process filter commands.
    // Read effective config without invoking those drivers, then fail closed.
    let filters = crate::process::no_console_window(&mut Command::new("git"))
        .current_dir(working_dir)
        .args(["-c", "core.fsmonitor=false"])
        .args([
            "config",
            "--null",
            "--name-only",
            "--get-regexp",
            "^filter\\.",
        ])
        .output()
        .context("Cannot verify Git filter execution policy")?;
    ensure!(
        filters.status.code() == Some(1),
        "Git worktree operations refused: configured filters or unverifiable filter policy require separate host-execution authority"
    );
    let output = crate::process::no_console_window(&mut Command::new("git"))
        .current_dir(working_dir)
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .output()
        .context("Failed to run Git")?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        args.first().unwrap_or(&""),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

/// List all worktrees, including bare, detached, locked, and prunable entries.
/// Accepts the repository root or any directory within a worktree.
pub fn list(working_dir: &str) -> Result<Vec<Worktree>> {
    parse(&git(
        working_dir,
        &["worktree", "list", "--porcelain", "-z"],
    )?)
}

fn parse(bytes: &[u8]) -> Result<Vec<Worktree>> {
    let text = std::str::from_utf8(bytes).context("Git worktree output is not UTF-8")?;
    let mut entries = Vec::new();
    let mut current: Option<Worktree> = None;
    for field in text.split('\0') {
        if field.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }
        let (key, value) = field.split_once(' ').unwrap_or((field, ""));
        if key == "worktree" {
            ensure!(current.is_none(), "Missing worktree record separator");
            current = Some(Worktree {
                path: value.to_owned(),
                branch: None,
                head: String::new(),
                bare: false,
                detached: false,
                locked: None,
                prunable: None,
            });
            continue;
        }
        let entry = current.as_mut().context("Missing worktree path")?;
        match key {
            "HEAD" => entry.head = value.to_owned(),
            "branch" => entry.branch = Some(value.to_owned()),
            "bare" => entry.bare = true,
            "detached" => entry.detached = true,
            "locked" => entry.locked = Some(value.to_owned()),
            "prunable" => entry.prunable = Some(value.to_owned()),
            _ => {} // Forward compatible with additional Git attributes.
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    Ok(entries)
}

// Encode rather than replace separators, so feature/a and feature-a cannot collide.
fn safe_name(branch: &str) -> String {
    let mut name = String::new();
    for byte in branch.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            name.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(name, "%{byte:02X}").expect("writing to String");
        }
    }
    name
}

/// Legacy API cannot establish user policy or task authority. Use `create_with_policy`.
pub fn create(_working_dir: &str, _branch: &str) -> Result<Worktree> {
    bail!("Worktree creation requires explicit policy and task authority; use create_with_policy")
}

/// Trusted caller inputs, not repository declarations. Consumers must be checked
/// by the caller's runtime immediately before invoking this blocking operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorktreeRequest {
    pub repository: String,
    pub creator: String,
    pub task: String,
    pub unique_id: String,
    pub branch: String,
    pub base: String,
    pub owner: String,
    pub creation_authorized: bool,
    pub consumers_verified: bool,
    pub live_owner: Option<String>,
}

/// Credential-free remote identity, or a stable identity of the canonical common
/// Git directory when no origin exists. A supplied request must match this value.
pub fn repository_identity(working_dir: &str) -> Result<String> {
    let common = git(
        working_dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let common = Path::new(std::str::from_utf8(&common)?.trim()).canonicalize()?;
    if let Ok(remote) = git(working_dir, &["remote", "get-url", "origin"]) {
        let remote = std::str::from_utf8(&remote)?.trim();
        let parsed = url::Url::parse(remote)
            .ok()
            .filter(|u| u.host_str().is_some());
        let pair = parsed
            .as_ref()
            .map(|u| (u.host_str().unwrap(), u.path()))
            .or_else(|| {
                remote.split_once(':').and_then(|(host, path)| {
                    (!host.contains('/')).then_some((host.rsplit('@').next().unwrap_or(host), path))
                })
            });
        if let Some((host, path)) = pair {
            ensure!(
                !host.is_empty() && !path.trim_matches('/').is_empty(),
                "Unknown repository identity"
            );
            return Ok(format!(
                "{}--{}",
                safe_name(&host.to_ascii_lowercase()),
                path.trim_matches('/')
                    .trim_end_matches(".git")
                    .split('/')
                    .map(safe_name)
                    .collect::<Vec<_>>()
                    .join("--")
            ));
        }
    }
    // FNV-1a is deterministic across processes. Ownership metadata also binds
    // the exact canonical common directory, so a key collision cannot grant reuse.
    let hash = common
        .to_string_lossy()
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    Ok(format!("local-{hash:016x}"))
}

fn component(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value != "."
            && value != ".."
            && value.len() <= 120
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')),
        "Unsafe or missing worktree name/identity component"
    );
    Ok(())
}

/// Reject symlinks in every existing ancestor, not only the final root. Never
/// follow a repository-controlled link to a destination outside the allowed root.
fn inspect_path(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "Worktree root must be absolute");
    let mut current = std::path::PathBuf::new();
    for part in path.components() {
        ensure!(
            !matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            ),
            "Worktree path traversal"
        );
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                ensure!(
                    !meta.file_type().is_symlink() && meta.is_dir(),
                    "Worktree path contains symlink or non-directory: {}",
                    current.display()
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Ownership {
    repository: String,
    common_dir: std::path::PathBuf,
    creator: String,
    task: String,
    unique_id: String,
    owner: String,
    branch: String,
    base: String,
}

fn ownership_path(path: &str) -> Result<std::path::PathBuf> {
    let dir = git(path, &["rev-parse", "--absolute-git-dir"])?;
    Ok(Path::new(std::str::from_utf8(&dir)?.trim()).join("jcode-worktree-owner.json"))
}

/// Resolve placement/authority once, then create or conservatively reuse. No
/// reset, clean, stash, switch, branch overwrite or automatic cleanup occurs.
pub fn create_with_policy(
    working_dir: &str,
    policy: &jcode_config_types::WorktreesConfig,
    request: &WorktreeRequest,
) -> Result<Worktree> {
    policy.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        request.creation_authorized || policy.allow_creation,
        "Explicit task worktree creation authority is required"
    );
    ensure!(
        policy.creator.as_deref() == Some(request.creator.as_str()),
        "Configured creator identity is missing or mismatched"
    );
    for value in [
        &request.creator,
        &request.task,
        &request.unique_id,
        &request.owner,
    ] {
        component(value)?;
    }
    ensure!(
        request.consumers_verified,
        "Active worktree consumers are unverified"
    );
    ensure!(
        request
            .live_owner
            .as_ref()
            .is_none_or(|owner| owner == &request.owner),
        "Worktree has an incompatible live owner"
    );
    let repository = repository_identity(working_dir)?;
    ensure!(
        repository == request.repository,
        "Repository identity mismatch"
    );
    ensure!(
        repository.len() <= 240,
        "Repository identity is too long for a worktree component"
    );
    let common = git(
        working_dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let common = Path::new(std::str::from_utf8(&common)?.trim()).canonicalize()?;
    ensure!(
        !request.branch.is_empty() && !request.branch.starts_with('-'),
        "Invalid branch name"
    );
    let checked = git(
        working_dir,
        &["check-ref-format", "--branch", &request.branch],
    )?;
    ensure!(
        checked == format!("{}\n", request.branch).as_bytes(),
        "Branch name must be literal"
    );
    ensure!(
        (request.base.len() == 40 || request.base.len() == 64)
            && request.base.bytes().all(|b| b.is_ascii_hexdigit()),
        "Base must be a verified commit ID"
    );
    let root = if let Some(relative) = policy.root.strip_prefix("~/") {
        std::path::PathBuf::from(
            std::env::var_os("HOME").context("HOME is required for ~/worktrees")?,
        )
        .join(relative)
    } else {
        policy.root.clone().into()
    };
    inspect_path(&root)?;
    let destination = root
        .join(&repository)
        .join(&request.creator)
        .join(format!("{}--{}", request.task, request.unique_id));
    inspect_path(&destination)?;
    let destination_str = destination
        .to_str()
        .context("Worktree destination is not UTF-8")?;
    let entries = list(working_dir)?;
    if let Some(existing) = entries
        .iter()
        .find(|entry| Path::new(&entry.path) == destination)
    {
        ensure!(
            !existing.bare
                && !existing.detached
                && existing.locked.is_none()
                && existing.prunable.is_none(),
            "Existing worktree is not reusable"
        );
        ensure!(
            existing.branch.as_deref() == Some(format!("refs/heads/{}", request.branch).as_str())
                && existing.head == request.base,
            "Worktree branch/base freshness mismatch"
        );
        let metadata_path = ownership_path(destination_str)?;
        let metadata = std::fs::symlink_metadata(&metadata_path)
            .context("Worktree ownership is unverified")?;
        ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "Worktree ownership metadata must be a real file"
        );
        let owner: Ownership = serde_json::from_slice(
            &std::fs::read(metadata_path).context("Worktree ownership is unverified")?,
        )?;
        ensure!(
            owner.repository == repository
                && owner.common_dir == common
                && owner.creator == request.creator
                && owner.task == request.task
                && owner.unique_id == request.unique_id
                && owner.owner == request.owner
                && owner.branch == request.branch
                && owner.base == request.base,
            "Worktree ownership mismatch"
        );
        ensure!(
            git(
                destination_str,
                &["status", "--porcelain", "--untracked-files=all"]
            )?
            .is_empty(),
            "Dirty worktree preserved; resolve conflict manually"
        );
        return Ok(existing.clone());
    }
    ensure!(
        !destination.try_exists()? && std::fs::symlink_metadata(&destination).is_err(),
        "Worktree destination already exists"
    );
    ensure!(
        git(working_dir, &["rev-parse", "HEAD"])? == format!("{}\n", request.base).as_bytes(),
        "Invoking HEAD no longer matches authorized base"
    );
    // Git atomically rejects an existing branch. Precheck before creating directories.
    ensure!(
        git(
            working_dir,
            &[
                "show-ref",
                "--verify",
                &format!("refs/heads/{}", request.branch)
            ]
        )
        .is_err(),
        "Branch already exists"
    );
    std::fs::create_dir_all(destination.parent().context("No worktree parent")?)?;
    inspect_path(&destination)?;
    git(
        working_dir,
        &[
            "-c",
            "core.hooksPath=",
            "worktree",
            "add",
            "-b",
            &request.branch,
            "--",
            destination_str,
            &request.base,
        ],
    )?;
    let owner = Ownership {
        repository,
        common_dir: common,
        creator: request.creator.clone(),
        task: request.task.clone(),
        unique_id: request.unique_id.clone(),
        owner: request.owner.clone(),
        branch: request.branch.clone(),
        base: request.base.clone(),
    };
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(ownership_path(destination_str)?)?;
    file.write_all(&serde_json::to_vec(&owner)?)?;
    list(working_dir)?
        .into_iter()
        .find(|entry| entry.path == destination_str)
        .context("Created worktree missing from Git list")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn repo() -> (TempDir, String) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("repo with spaces");
        std::fs::create_dir(&path).unwrap();
        let path = path.to_str().unwrap().to_owned();
        git(&path, &["init", "-b", "main"]).unwrap();
        // Fixture-only identity, never used for repository development commits.
        git(&path, &["config", "user.name", "SDK Test"]).unwrap();
        git(&path, &["config", "user.email", "sdk-test@example.invalid"]).unwrap();
        std::fs::write(Path::new(&path).join("tracked"), "original").unwrap();
        git(&path, &["add", "tracked"]).unwrap();
        git(&path, &["commit", "-m", "initial"]).unwrap();
        (temp, path)
    }

    fn policy(temp: &TempDir) -> jcode_config_types::WorktreesConfig {
        jcode_config_types::WorktreesConfig {
            root: temp.path().join("allowed").to_str().unwrap().into(),
            creator: Some("fixture-owner".into()),
            ..Default::default()
        }
    }

    fn request(path: &str, branch: &str) -> WorktreeRequest {
        WorktreeRequest {
            branch: branch.into(),
            task: "task".into(),
            unique_id: branch.replace('/', "-"),
            creator: "fixture-owner".into(),
            owner: "session-a".into(),
            repository: repository_identity(path).unwrap(),
            base: String::from_utf8(git(path, &["rev-parse", "HEAD"]).unwrap())
                .unwrap()
                .trim()
                .into(),
            creation_authorized: true,
            consumers_verified: true,
            live_owner: None,
        }
    }

    #[test]
    fn worktrees_policy_requires_authority_identity_and_safe_names() {
        let (temp, path) = repo();
        assert!(
            create(&path, "legacy").is_err(),
            "legacy API must not infer authority"
        );
        let mut policy = policy(&temp);
        let mut req = request(&path, "new");
        req.creation_authorized = false;
        assert!(create_with_policy(&path, &policy, &req).is_err());
        assert!(!Path::new(&policy.root).exists());
        req.creation_authorized = true;
        policy.creator = None;
        assert!(create_with_policy(&path, &policy, &req).is_err());
        policy.creator = Some(req.creator.clone());
        req.unique_id = "../escape".into();
        assert!(create_with_policy(&path, &policy, &req).is_err());
        req.unique_id = "safe".into();
        req.repository = "wrong-repository".into();
        assert!(create_with_policy(&path, &policy, &req).is_err());
        assert!(git(&path, &["show-ref", "--verify", "refs/heads/new"]).is_err());
    }

    #[test]
    fn worktrees_policy_reuse_checks_dirty_base_and_live_owner() {
        let (temp, path) = repo();
        let policy = policy(&temp);
        let mut req = request(&path, "reusable");
        let created = create_with_policy(&path, &policy, &req).unwrap();
        assert!(Path::new(&created.path).starts_with(&policy.root));
        assert!(created.path.contains("/fixture-owner/task--reusable"));
        assert_eq!(create_with_policy(&path, &policy, &req).unwrap(), created);
        req.live_owner = Some("session-b".into());
        assert!(create_with_policy(&path, &policy, &req).is_err());
        req.live_owner = None;
        req.consumers_verified = false;
        assert!(create_with_policy(&path, &policy, &req).is_err());
        req.consumers_verified = true;
        std::fs::write(Path::new(&created.path).join("untracked"), "preserve").unwrap();
        assert!(create_with_policy(&path, &policy, &req).is_err());
        assert_eq!(
            std::fs::read_to_string(Path::new(&created.path).join("untracked")).unwrap(),
            "preserve"
        );
        std::fs::remove_file(Path::new(&created.path).join("untracked")).unwrap();
        git(
            &created.path,
            &["commit", "--allow-empty", "-m", "advanced"],
        )
        .unwrap();
        assert!(create_with_policy(&path, &policy, &req).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn worktrees_policy_rejects_ancestor_symlink_escape() {
        let (temp, path) = repo();
        let mut policy = policy(&temp);
        let outside = temp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        let link = temp.path().join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        policy.root = link.join("nested").to_str().unwrap().into();
        assert!(create_with_policy(&path, &policy, &request(&path, "escape")).is_err());
        assert_eq!(std::fs::read_dir(outside).unwrap().count(), 0);
    }

    #[test]
    fn worktrees_policy_preserves_existing_unowned_destination_and_remote_credentials() {
        let (temp, path) = repo();
        git(
            &path,
            &[
                "remote",
                "add",
                "origin",
                "https://user:secret@example.invalid/team/repo.git?token=private",
            ],
        )
        .unwrap();
        let identity = repository_identity(&path).unwrap();
        assert_eq!(identity, "example%2Einvalid--team--repo");
        assert!(!identity.contains("secret") && !identity.contains("token"));
        let policy = policy(&temp);
        let req = request(&path, "occupied");
        let destination = Path::new(&policy.root)
            .join(identity)
            .join("fixture-owner/task--occupied");
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(destination.join("private"), "preserve").unwrap();
        assert!(create_with_policy(&path, &policy, &req).is_err());
        assert_eq!(
            std::fs::read_to_string(destination.join("private")).unwrap(),
            "preserve"
        );
        assert!(git(&path, &["show-ref", "--verify", "refs/heads/occupied"]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn worktrees_policy_rejects_symlink_owner_metadata() {
        let (temp, path) = repo();
        let policy = policy(&temp);
        let req = request(&path, "metadata");
        let created = create_with_policy(&path, &policy, &req).unwrap();
        let metadata = ownership_path(&created.path).unwrap();
        let outside = temp.path().join("owner-copy.json");
        std::fs::rename(&metadata, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &metadata).unwrap();
        assert!(
            create_with_policy(&path, &policy, &req).is_err(),
            "must not follow ownership symlink"
        );
    }

    #[test]
    fn worktrees_policy_does_not_execute_repository_checkout_hooks() {
        let (temp, path) = repo();
        let hooks = temp.path().join("hooks");
        std::fs::create_dir(&hooks).unwrap();
        let sentinel = temp.path().join("hook-ran");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let hook = hooks.join("post-checkout");
            std::fs::write(
                &hook,
                format!("#!/bin/sh\ntouch '{}'\n", sentinel.display()),
            )
            .unwrap();
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
            git(
                &path,
                &["config", "core.hooksPath", hooks.to_str().unwrap()],
            )
            .unwrap();
            create_with_policy(&path, &policy(&temp), &request(&path, "no-hooks")).unwrap();
            assert!(
                !sentinel.exists(),
                "repository checkout hook executed without authority"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn worktrees_policy_refuses_configured_filters_before_creation_and_reuse() {
        let (temp, path) = repo();
        std::fs::write(
            Path::new(&path).join(".gitattributes"),
            "tracked filter=sentinel\n",
        )
        .unwrap();
        git(&path, &["add", ".gitattributes"]).unwrap();
        git(&path, &["commit", "-m", "filter attributes fixture"]).unwrap();
        let policy = policy(&temp);
        let req = request(&path, "filtered");
        let created = create_with_policy(&path, &policy, &req).unwrap();
        let sentinel = temp.path().join("filter-ran");
        // Deliberately configure a disposable fixture through raw Git. Production
        // SDK Git must refuse these drivers before checkout or status can run.
        let command = format!("touch '{}'; cat", sentinel.display());
        for driver in ["smudge", "clean", "process"] {
            let configured = Command::new("git")
                .current_dir(&path)
                .args(["config", &format!("filter.sentinel.{driver}"), &command])
                .output()
                .unwrap();
            assert!(configured.status.success());
            assert!(
                create_with_policy(&path, &policy, &request_without_git(&req, "filtered-new"))
                    .is_err(),
                "configured {driver} filter must be rejected before checkout"
            );
            assert!(
                list(&path).is_err(),
                "configured {driver} filter must be rejected before Git worktree operations"
            );
            assert!(create_with_policy(&path, &policy, &req).is_err());
            assert!(
                !sentinel.exists(),
                "configured {driver} filter executed without authority"
            );
            assert!(Path::new(&created.path).exists());
            let cleared = Command::new("git")
                .current_dir(&path)
                .args(["config", "--unset", &format!("filter.sentinel.{driver}")])
                .output()
                .unwrap();
            assert!(cleared.status.success());
        }
    }

    #[cfg(unix)]
    fn request_without_git(existing: &WorktreeRequest, branch: &str) -> WorktreeRequest {
        WorktreeRequest {
            branch: branch.into(),
            unique_id: branch.into(),
            ..existing.clone()
        }
    }

    #[cfg(unix)]
    #[test]
    fn worktrees_policy_reuse_does_not_execute_fsmonitor() {
        use std::os::unix::fs::PermissionsExt;
        let (temp, path) = repo();
        let policy = policy(&temp);
        let req = request(&path, "monitor");
        let created = create_with_policy(&path, &policy, &req).unwrap();
        let sentinel = temp.path().join("fsmonitor-ran");
        let monitor = temp.path().join("fsmonitor");
        std::fs::write(
            &monitor,
            format!("#!/bin/sh\ntouch '{}'\nprintf '\\0'\n", sentinel.display()),
        )
        .unwrap();
        std::fs::set_permissions(&monitor, std::fs::Permissions::from_mode(0o755)).unwrap();
        let configured = Command::new("git")
            .current_dir(&path)
            .args(["config", "core.fsmonitor", monitor.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(configured.status.success());
        assert_eq!(create_with_policy(&path, &policy, &req).unwrap(), created);
        assert!(
            !sentinel.exists(),
            "reuse status executed fsmonitor without host authority"
        );
    }

    #[test]
    fn parses_nul_records_without_trimming_paths_or_reasons() {
        let entries = parse(b"worktree /repo with\nnewline \0HEAD abc\0branch refs/heads/main\0locked reason\nwith newline \0\0worktree /bare\0bare\0\0worktree /detached\0HEAD def\0detached\0locked\0prunable missing path\0\0").unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, "/repo with\nnewline ");
        assert_eq!(entries[0].locked.as_deref(), Some("reason\nwith newline "));
        assert!(entries[1].bare);
        assert_eq!(entries[1].head, "");
        assert!(entries[2].detached);
        assert_eq!(entries[2].branch, None);
        assert_eq!(entries[2].locked.as_deref(), Some(""));
        assert_eq!(entries[2].prunable.as_deref(), Some("missing path"));
        assert!(parse(b"worktree /\xff\0\0").is_err());
    }

    fn authorized_create(path: &str, branch: &str) -> Result<Worktree> {
        let entries = list(path)?;
        let root = Path::new(&entries[0].path)
            .parent()
            .unwrap()
            .join("allowed");
        let policy = jcode_config_types::WorktreesConfig {
            root: root.to_str().unwrap().into(),
            creator: Some("fixture-owner".into()),
            ..Default::default()
        };
        let base =
            String::from_utf8(git(path, &["rev-parse", "HEAD"])?).context("HEAD is not UTF-8")?;
        let mut req = WorktreeRequest {
            repository: repository_identity(path)?,
            creator: "fixture-owner".into(),
            task: "task".into(),
            unique_id: String::new(),
            branch: branch.into(),
            base: base.trim().into(),
            owner: "session-a".into(),
            creation_authorized: true,
            consumers_verified: true,
            live_owner: None,
        };
        req.unique_id = safe_name(branch).replace('%', "_");
        create_with_policy(path, &policy, &req)
    }

    #[test]
    fn creation_isolates_dirty_edits_and_nested_invocation() {
        let (temp, path) = repo();
        let before = list(&path).unwrap().remove(0);
        std::fs::write(Path::new(&path).join("tracked"), "staged").unwrap();
        git(&path, &["add", "tracked"]).unwrap();
        std::fs::write(Path::new(&path).join("tracked"), "unstaged").unwrap();
        std::fs::write(Path::new(&path).join("untracked"), "private").unwrap();
        let nested = Path::new(&path).join("nested");
        std::fs::create_dir(&nested).unwrap();
        let created = authorized_create(nested.to_str().unwrap(), "feature/one").unwrap();
        assert_eq!(created.head, before.head);
        assert_eq!(created.branch.as_deref(), Some("refs/heads/feature/one"));
        assert_eq!(
            Path::new(&created.path),
            temp.path()
                .join("allowed")
                .join(repository_identity(&path).unwrap())
                .join("fixture-owner/task--feature_2Fone")
        );
        assert_eq!(
            std::fs::read_to_string(Path::new(&created.path).join("tracked")).unwrap(),
            "original"
        );
        assert!(!Path::new(&created.path).join("untracked").exists());
        assert_eq!(
            std::fs::read_to_string(Path::new(&path).join("tracked")).unwrap(),
            "unstaged"
        );
        assert_eq!(git(&path, &["show", ":tracked"]).unwrap(), b"staged");
        git(
            &created.path,
            &["commit", "--allow-empty", "-m", "linked HEAD"],
        )
        .unwrap();
        let linked_head = git(&created.path, &["rev-parse", "HEAD"]).unwrap();
        let second = authorized_create(&created.path, "feature-two").unwrap();
        assert_eq!(format!("{}\n", second.head).as_bytes(), linked_head);
        assert_ne!(second.head, before.head);
        assert_eq!(
            Path::new(&second.path).parent(),
            Path::new(&created.path).parent()
        );
        assert_eq!(list(&path).unwrap().len(), 3);
    }

    #[test]
    fn rejects_invalid_branches_collisions_and_non_repositories() {
        let (temp, path) = repo();
        for branch in [
            "",
            "-oops",
            "--detach",
            "a b",
            "../escape",
            "a..b",
            "a\nb",
            "@{-1}",
        ] {
            assert!(
                authorized_create(&path, branch).is_err(),
                "accepted {branch:?}"
            );
        }
        assert!(authorized_create(&path, "main").is_err());
        let created = authorized_create(&path, "new").unwrap();
        assert_eq!(authorized_create(&path, "new").unwrap(), created);
        assert!(Path::new(&created.path).exists());
        let occupied = temp
            .path()
            .join("allowed")
            .join(repository_identity(&path).unwrap())
            .join("fixture-owner/task--occupied");
        std::fs::create_dir(&occupied).unwrap();
        assert!(authorized_create(&path, "occupied").is_err());
        assert!(git(&path, &["show-ref", "--verify", "refs/heads/occupied"]).is_err());
        // TMPDIR may be inside the source checkout under the dev wrapper.
        // Stop Git discovery explicitly so this fixture cannot find its parent.
        std::fs::write(temp.path().join(".git"), "not a repository").unwrap();
        assert!(list(temp.path().to_str().unwrap()).is_err());
        assert!(authorized_create(temp.path().to_str().unwrap(), "valid").is_err());
        assert_ne!(safe_name("feature/a"), safe_name("feature-a"));
        assert_ne!(safe_name("feature/a"), safe_name("feature%2Fa"));
    }

    #[test]
    fn lists_existing_detached_locked_prunable_and_bare_worktrees() {
        let (temp, path) = repo();
        let detached = temp.path().join("detached\nwith space");
        let detached = detached.to_str().unwrap();
        git(&path, &["worktree", "add", "--detach", detached, "HEAD"]).unwrap();
        git(
            &path,
            &["worktree", "lock", "--reason", "keep\nthis", detached],
        )
        .unwrap();
        let entry = list(&path)
            .unwrap()
            .into_iter()
            .find(|w| w.path == detached)
            .unwrap();
        assert!(entry.detached);
        assert_eq!(entry.locked.as_deref(), Some("keep\nthis"));
        git(&path, &["worktree", "unlock", detached]).unwrap();
        std::fs::remove_dir_all(detached).unwrap();
        assert!(
            list(&path)
                .unwrap()
                .iter()
                .any(|w| w.path == detached && w.prunable.is_some())
        );
        let bare = temp.path().join("bare.git");
        git(&path, &["clone", "--bare", &path, bare.to_str().unwrap()]).unwrap();
        let entries = list(bare.to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].bare);
        let created = authorized_create(bare.to_str().unwrap(), "from-bare").unwrap();
        assert_eq!(created.branch.as_deref(), Some("refs/heads/from-bare"));
    }

    #[test]
    fn unborn_head_does_not_create_a_branch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_str().unwrap();
        git(path, &["init", "-b", "main"]).unwrap();
        assert!(authorized_create(path, "new").is_err());
        assert!(git(path, &["show-ref", "--verify", "refs/heads/new"]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlink_destination_root() {
        let (temp, path) = repo();
        let elsewhere = temp.path().join("elsewhere");
        std::fs::create_dir(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, temp.path().join("allowed")).unwrap();
        assert!(authorized_create(&path, "new").is_err());
        assert_eq!(std::fs::read_dir(elsewhere).unwrap().count(), 0);
    }
}
