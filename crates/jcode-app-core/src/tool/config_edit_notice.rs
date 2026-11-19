//! Tell the agent (and through it, the user) what a config.toml edit did.
//!
//! When a user asks jcode to change a setting, the agent writes
//! `~/.jcode/config.toml` and then has to guess whether the change took
//! effect. That guess is where the confusion comes from. Instead, every file
//! write that lands on the active config file appends an explicit report:
//! which keys changed, and whether each one is live in running sessions or
//! needs a restart.

use std::path::Path;

/// Resolve a path for comparison, falling back to the path as given.
///
/// A config file that does not exist yet cannot be canonicalized, and that is
/// a normal case here (the very first write creates it), so the unresolved
/// path is the correct answer rather than an error to report.
fn comparable(path: &Path) -> std::path::PathBuf {
    match std::fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(_) => path.to_path_buf(),
    }
}

/// Whether `path` is the config file the running process actually reads.
///
/// Compares resolved paths so `~/.jcode/config.toml`, a relative path, and a
/// symlinked jcode home all resolve to the same file.
fn is_active_config_file(path: &Path) -> bool {
    let Some(config_path) = crate::config::Config::path() else {
        return false;
    };
    comparable(path) == comparable(&config_path)
}

/// The fragment listed by the active root manifest that `path` names, if any.
fn is_active_fragment(path: &Path) -> bool {
    fragment_position(path).is_some()
}

/// Index of `path` among the listed fragments (root excluded), if it is one.
fn fragment_position(path: &Path) -> Option<usize> {
    let config_path = crate::config::Config::path()?;
    let target = comparable(path);
    crate::config::Config::source_paths(&config_path)
        .iter()
        .skip(1)
        .position(|source| comparable(source) == target)
}

/// Exact dotted keys set by a source that wins over fragment `index`.
///
/// Later fragments and the root replace earlier values leaf by leaf, so an
/// edit to a key they also set changes nothing in the effective config. Only an
/// exact key match masks: a related key (a parent or child path) does not.
/// Each winning source is read once, not once per changed key.
fn winning_keys(index: usize) -> std::collections::HashSet<String> {
    let Some(config_path) = crate::config::Config::path() else {
        return Default::default();
    };
    crate::config::Config::source_paths(&config_path)
        .iter()
        .enumerate()
        .skip(index + 2) // index 0 is the root, fragment `index` is at `index + 1`
        .map(|(_, source)| source)
        .chain(std::iter::once(&config_path))
        .flat_map(|source| {
            crate::config::change_report::diff_toml("", &read_config_text(source))
                .into_iter()
                .map(|change| change.key)
        })
        .collect()
}

/// Report appended to a tool result after a write to the active config file.
///
/// Returns `None` for non-config files and for edits that changed no settings
/// (comments or formatting), so ordinary writes stay untouched.
pub fn config_edit_notice(path: &Path, before: &str, after: &str) -> Option<String> {
    let fragment = !is_active_config_file(path);
    if fragment && !is_active_fragment(path) {
        return None;
    }
    // Force the next config() call to re-read instead of waiting out the
    // staleness throttle, so "live now" is true the moment it is claimed.
    crate::config::Config::invalidate_cache();

    // A config file that no longer parses is silently ignored by
    // `Config::load`, which falls back to defaults. That is the worst possible
    // outcome to leave unreported: the write "succeeded" while every setting
    // in the file quietly stopped applying. Surface it instead.
    if let Err(error) = crate::config::Config::load_strict() {
        return Some(format!(
            "\n\nWARNING: the jcode config sources no longer load (after editing {}), so \
             jcode is falling back to default settings and every setting in the root \
             file and its fragments is being ignored. Fix the error: {error:#}",
            path.display()
        ));
    }

    if let Some(index) = fragment_position(path) {
        let winning = winning_keys(index);
        let changes: Vec<_> = crate::config::change_report::diff_toml(before, after)
            .into_iter()
            .filter(|change| !winning.contains(&change.key))
            .collect();
        if changes.is_empty() {
            return Some(format!(
                "\n\nNote: {} changed, but every changed key is overridden by a later \
                 fragment or the root config, so the effective settings are unchanged.",
                path.display()
            ));
        }
        let summary = crate::config::change_report::summarize_changes(&changes);
        return Some(format!("\n\n{summary}"));
    }
    let summary = crate::config::change_report::summarize_toml_change(before, after)?;
    Some(format!("\n\n{summary}"))
}

/// Append [`config_edit_notice`] to a tool output body when applicable.
pub fn append_config_edit_notice(body: &mut String, path: &Path, before: &str, after: &str) {
    if let Some(notice) = config_edit_notice(path, before, after) {
        body.push_str(&notice);
    }
}

/// Read the config file, treating "absent or unreadable" as empty.
///
/// An absent config file is the normal pre-state for the write that creates
/// it, and an unreadable one is reported by the change summary itself, so
/// there is no error here worth propagating: empty is the meaningful value.
fn read_config_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Watches the active config file across a whole tool invocation.
///
/// Tools that touch several files, or write through several code paths (patch
/// application, moves, deletes), cannot easily thread before/after content to
/// the place that builds the result string. This captures the config file
/// content up front and re-reads it at the end, so a config edit is reported
/// no matter which path produced it.
pub struct ConfigEditWatch {
    /// Root file followed by every fragment its manifest listed at `begin`.
    sources: Vec<(std::path::PathBuf, String)>,
}

impl ConfigEditWatch {
    /// Snapshot the active config file and its listed fragments before a tool runs.
    pub fn begin() -> Self {
        let sources = match crate::config::Config::path() {
            Some(root) => crate::config::Config::source_paths(&root)
                .into_iter()
                .map(|path| {
                    let text = read_config_text(&path);
                    (path, text)
                })
                .collect(),
            None => Vec::new(),
        };
        Self { sources }
    }

    /// Append a change report for each watched source that changed while the tool ran.
    ///
    /// A fragment first listed by an edit to the root is picked up too: the
    /// root's report covers the manifest change, and newly listed fragments
    /// are compared against empty.
    pub fn finish(self, body: &mut String) {
        let mut sources = self.sources;
        if let Some(root) = crate::config::Config::path() {
            for path in crate::config::Config::source_paths(&root) {
                if !sources.iter().any(|(known, _)| *known == path) {
                    sources.push((path, String::new()));
                }
            }
        }
        for (path, before) in sources {
            let after = read_config_text(&path);
            if after != before {
                append_config_edit_notice(body, &path, &before, &after);
            }
        }
    }
}

#[cfg(test)]
#[path = "config_edit_notice_tests.rs"]
mod tests;
