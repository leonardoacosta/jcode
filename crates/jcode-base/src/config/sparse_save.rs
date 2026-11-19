//! Sparse, comment-preserving root-file persistence for [`Config::save`].
//!
//! A save never flattens the effective configuration. It diffs the requested
//! typed config against a fresh file-only baseline (root plus included
//! fragments, no environment overrides) and patches only the changed leaves
//! into the root document with `toml_edit`. Fragments are never rewritten.
//! Arrays are indivisible leaves. The root is replaced atomically through a
//! same-directory temporary file.

use super::composition::{SourceFingerprint, compose, fragment_tables};
use super::*;
use std::path::Path;
use toml_edit::{Document, Item, Table, TableLike};

#[derive(Debug, PartialEq)]
enum Op {
    Set(Vec<String>, toml::Value),
    Remove(Vec<String>),
}

fn dotted(path: &[String]) -> String {
    path.join(".")
}

fn diff_tables(prefix: &[String], base: &toml::Table, req: &toml::Table, ops: &mut Vec<Op>) {
    let empty = toml::Table::new();
    for (key, wanted) in req {
        let mut path = prefix.to_vec();
        path.push(key.clone());
        match (base.get(key), wanted) {
            (Some(have), wanted) if have == wanted => {}
            (Some(toml::Value::Table(have)), toml::Value::Table(wanted)) => {
                diff_tables(&path, have, wanted, ops);
            }
            (_, toml::Value::Table(wanted)) => diff_tables(&path, &empty, wanted, ops),
            (_, wanted) => ops.push(Op::Set(path, wanted.clone())),
        }
    }
    for (key, have) in base {
        if req.contains_key(key) {
            continue;
        }
        let mut path = prefix.to_vec();
        path.push(key.clone());
        match have {
            toml::Value::Table(have) => diff_tables(&path, have, &empty, ops),
            // An empty list is representable, so it stays an explicit override.
            toml::Value::Array(_) => ops.push(Op::Set(path, toml::Value::Array(Vec::new()))),
            _ => ops.push(Op::Remove(path)),
        }
    }
}

fn lookup<'a>(table: &'a toml::Table, path: &[String]) -> Option<&'a toml::Value> {
    let (last, parents) = path.split_last()?;
    let mut current = table;
    for segment in parents {
        current = current.get(segment)?.as_table()?;
    }
    current.get(last)
}

fn to_edit_value(value: &toml::Value) -> anyhow::Result<toml_edit::Value> {
    value
        .to_string()
        .parse::<toml_edit::Value>()
        .map_err(|error| anyhow::anyhow!("Cannot encode config value: {error}"))
}

fn table_at<'a>(
    root: &'a mut Document,
    path: &[String],
    create: bool,
    root_path: &Path,
) -> anyhow::Result<Option<&'a mut dyn TableLike>> {
    let mut current: &mut dyn TableLike = root.as_table_mut();
    for segment in path {
        if !current.contains_key(segment) {
            if !create {
                return Ok(None);
            }
            let mut table = Table::new();
            table.set_implicit(true);
            current.insert(segment, Item::Table(table));
        }
        let Some(item) = current.get_mut(segment) else {
            return Ok(None);
        };
        current = item.as_table_like_mut().ok_or_else(|| {
            anyhow::anyhow!(
                "{}: config key `{}` is not a table, so it cannot be overridden here",
                root_path.display(),
                dotted(path)
            )
        })?;
    }
    Ok(Some(current))
}

fn apply_set(
    doc: &mut Document,
    path: &[String],
    value: &toml::Value,
    root_path: &Path,
) -> anyhow::Result<()> {
    let (leaf, parents) = path.split_last().expect("non-empty config key path");
    let mut new_value = to_edit_value(value)?;
    let table = table_at(doc, parents, true, root_path)?.expect("created parents");
    if let Some(Item::Value(old)) = table.get(leaf) {
        *new_value.decor_mut() = old.decor().clone();
    }
    table.insert(leaf, Item::Value(new_value));
    Ok(())
}

fn apply_remove(
    doc: &mut Document,
    path: &[String],
    fragments: &[(std::path::PathBuf, toml::Table)],
    root_path: &Path,
) -> anyhow::Result<()> {
    if let Some((source, _)) = fragments
        .iter()
        .rev()
        .find(|(_, table)| lookup(table, path).is_some())
    {
        anyhow::bail!(
            "Cannot remove config key `{}`: its value is inherited from {} and TOML has no \
             neutral value to override it. Edit that file to change it.",
            dotted(path),
            source.display()
        );
    }
    let (leaf, parents) = path.split_last().expect("non-empty config key path");
    for depth in (0..=parents.len()).rev() {
        let (target, owner) = if depth == parents.len() {
            (leaf.as_str(), parents)
        } else {
            (parents[depth].as_str(), &parents[..depth])
        };
        let Some(table) = table_at(doc, owner, false, root_path)? else {
            return Ok(());
        };
        if depth < parents.len() {
            let emptied = table
                .get(target)
                .and_then(|item| item.as_table_like())
                .is_some_and(|child| child.is_empty());
            if !emptied {
                return Ok(());
            }
        }
        table.remove(target);
    }
    Ok(())
}

#[cfg(test)]
pub(super) enum TestFault {
    /// Run once after the baseline is read, before the replace-time drift check.
    BeforeReplace(Box<dyn FnOnce() + Send>),
    /// Run once right after the temporary file is created, before any content.
    AfterCreate(Box<dyn FnOnce() + Send>),
    /// Fail the final rename.
    FailReplace,
    /// Fail writing the temporary file.
    FailWrite,
}

#[cfg(test)]
pub(super) static TEST_FAULT: std::sync::Mutex<Option<TestFault>> = std::sync::Mutex::new(None);

#[cfg(test)]
fn take_fault() -> Option<TestFault> {
    TEST_FAULT.lock().ok().and_then(|mut guard| guard.take())
}

static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TempFile(std::path::PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Replace `target` atomically with `content`, keeping its permissions.
fn replace_atomically(
    target: &Path,
    content: &str,
    expected: &SourceFingerprint,
    root_path: &Path,
) -> anyhow::Result<()> {
    use std::io::Write;
    let dir = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("config path {} has no directory", target.display()))?;
    std::fs::create_dir_all(dir)?;
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".to_string());
    let temp_path = dir.join(format!(
        ".{name}.jcode-tmp-{}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let temp = TempFile(temp_path);
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        // An existing file may hold inline API keys: never create its replacement
        // wider than 0600 (its exact mode is copied once content is written). A
        // brand-new root keeps the previous writer's umask-default mode.
        #[cfg(unix)]
        if target.exists() {
            std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        }
        let mut file = options
            .open(&temp.0)
            .map_err(|e| anyhow::anyhow!("Cannot create temporary config file: {e}"))?;
        #[cfg(test)]
        let mut fault = take_fault();
        #[cfg(test)]
        if let Some(TestFault::AfterCreate(hook)) =
            fault.take_if(|f| matches!(f, TestFault::AfterCreate(_)))
        {
            hook();
        }
        #[cfg(test)]
        if matches!(fault, Some(TestFault::FailWrite)) {
            anyhow::bail!("Cannot write temporary config file: injected failure");
        }
        file.write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Cannot write temporary config file: {e}"))?;
        if let Ok(meta) = std::fs::metadata(target) {
            file.set_permissions(meta.permissions())?;
        }
        file.sync_all()?;
        #[cfg(test)]
        match fault.take() {
            Some(TestFault::BeforeReplace(hook)) => hook(),
            Some(TestFault::FailReplace) => {
                anyhow::bail!("Cannot replace config file: injected failure")
            }
            _ => {}
        }
    }
    if SourceFingerprint::read(Some(root_path.to_path_buf())) != *expected {
        anyhow::bail!(
            "Config sources changed while saving ({}); nothing was written. Retry the change.",
            root_path.display()
        );
    }
    std::fs::rename(&temp.0, target)
        .map_err(|e| anyhow::anyhow!("Cannot replace config file {}: {e}", target.display()))?;
    std::mem::forget(temp);
    if let Ok(dir_handle) = std::fs::File::open(dir) {
        let _ = dir_handle.sync_all();
    }
    Ok(())
}

/// Give `rendered` the line endings `original` used.
///
/// `toml_edit` normalizes CRLF to LF when re-rendering, which would rewrite
/// every line of a Windows-style file for a one-leaf change. Only a file that
/// is CRLF throughout (no bare LF) is converted back, so mixed files keep the
/// renderer's output rather than being guessed at.
fn keep_line_endings(original: &str, rendered: String) -> String {
    let crlf = original.matches("\r\n").count();
    let lf = original.matches('\n').count();
    if crlf == 0 || crlf != lf {
        return rendered;
    }
    rendered.replace("\r\n", "\n").replace('\n', "\r\n")
}

/// A symlinked root keeps its link: replace the resolved target instead.
fn resolve_root_target(root_path: &Path) -> anyhow::Result<std::path::PathBuf> {
    match std::fs::symlink_metadata(root_path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let resolved = std::fs::canonicalize(root_path)?;
            if !std::fs::metadata(&resolved).is_ok_and(|m| m.is_file()) {
                anyhow::bail!(
                    "config symlink {} does not point to a regular file",
                    root_path.display()
                );
            }
            Ok(resolved)
        }
        _ => Ok(root_path.to_path_buf()),
    }
}

impl Config {
    /// Patch only the root document, without composing or typing any source.
    ///
    /// For one-time migrations that must still run when an unrelated part of
    /// the composition does not load. Returns whether `patch` changed anything;
    /// an unparseable root is left untouched and reported as unchanged.
    pub(super) fn patch_root_document(
        patch: impl FnOnce(&mut Document) -> bool,
    ) -> anyhow::Result<bool> {
        let root_path = Self::path().ok_or_else(|| anyhow::anyhow!("No config path"))?;
        let before = SourceFingerprint::read(Some(root_path.clone()));
        let text = std::fs::read_to_string(&root_path)?;
        let Ok(mut doc) = text.parse::<Document>() else {
            return Ok(false);
        };
        if !patch(&mut doc) {
            return Ok(false);
        }
        let target = resolve_root_target(&root_path)?;
        replace_atomically(
            &target,
            &keep_line_endings(&text, doc.to_string()),
            &before,
            &root_path,
        )?;
        Self::invalidate_cache();
        Ok(true)
    }

    /// Persist `self` as root-level overrides without rewriting fragments.
    pub(super) fn save_sparse(&self) -> anyhow::Result<()> {
        let root_path = Self::path().ok_or_else(|| anyhow::anyhow!("No config path"))?;
        let before = SourceFingerprint::read(Some(root_path.clone()));
        let existing = match std::fs::read_to_string(&root_path) {
            Ok(text) => Some(text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => anyhow::bail!("Failed to read config file {}: {e}", root_path.display()),
        };
        let (baseline, fragments) = match &existing {
            Some(text) => {
                let composed = compose(&root_path, text)?;
                (composed.table, fragment_tables(&root_path, text)?)
            }
            None => (toml::Table::new(), Vec::new()),
        };
        let baseline_typed = Self::typed_from_table(&root_path, baseline)?;
        let to_table = |config: &Self| -> anyhow::Result<toml::Table> {
            match toml::Value::try_from(config)? {
                toml::Value::Table(table) => Ok(table),
                _ => anyhow::bail!("config did not serialize to a table"),
            }
        };
        let mut ops = Vec::new();
        diff_tables(&[], &to_table(&baseline_typed)?, &to_table(self)?, &mut ops);

        if ops.is_empty() {
            Self::invalidate_cache();
            return Ok(());
        }
        let mut doc: Document = match &existing {
            Some(text) => text.parse().map_err(|e: toml_edit::TomlError| {
                anyhow::anyhow!(
                    "Failed to parse config file {}: {}",
                    root_path.display(),
                    e.message()
                )
            })?,
            None => Document::new(),
        };
        for op in &ops {
            match op {
                Op::Set(path, value) => apply_set(&mut doc, path, value, &root_path)?,
                Op::Remove(path) => apply_remove(&mut doc, path, &fragments, &root_path)?,
            }
        }
        let rendered = keep_line_endings(existing.as_deref().unwrap_or(""), doc.to_string());
        // Never write a document that no longer composes to the requested config.
        Self::typed_from_table(&root_path, compose(&root_path, &rendered)?.table)?;

        let target = resolve_root_target(&root_path)?;
        replace_atomically(&target, &rendered, &before, &root_path)?;
        Self::invalidate_cache();
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn diff_sets_changed_leaves_and_keeps_arrays_whole() {
        let base: toml::Table = "[a]\nx = 1\ny = [1, 2]\n[b]\nz = true\n".parse().unwrap();
        let req: toml::Table = "[a]\nx = 2\ny = [1, 2, 3]\n[c]\nn = \"v\"\n"
            .parse()
            .unwrap();
        let mut ops = Vec::new();
        diff_tables(&[], &base, &req, &mut ops);
        assert!(ops.contains(&Op::Set(vec!["a".into(), "x".into()], 2.into())));
        assert!(ops.contains(&Op::Set(
            vec!["a".into(), "y".into()],
            toml::Value::Array(vec![1.into(), 2.into(), 3.into()])
        )));
        assert!(ops.contains(&Op::Remove(vec!["b".into(), "z".into()])));
        assert!(ops.contains(&Op::Set(vec!["c".into(), "n".into()], "v".into())));
        assert_eq!(ops.len(), 4);
    }
}
