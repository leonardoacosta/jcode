---
execution:
  version: 1
  depends_on: []
---
# Compose configuration from explicit TOML files

## Why

The inspected user config is 540 lines, including 235 lines of named provider catalogs, while three alternate snapshots duplicate common settings. Splitting by concern would improve navigation, but the current loader reads only `config.toml`, saves serialize the entire typed config, and reload/auth caches watch only the root file.

## What Changes

- Add an optional root `include = ["config/ui.toml", "config/providers/example.toml"]` manifest. Fragments keep existing fully qualified TOML tables. Keep single-file configuration supported and do not scan directories.
- Resolve explicit, relative, contained paths and load one include level. Merge tables recursively in include order, then apply root settings, typed defaults and existing environment overrides. Replace arrays rather than concatenate them. Reject table/value shape conflicts.
- Preserve comments, unknown keys and fragments during preference saves. Persist only changed settings as root overrides, never flatten the effective configuration or bake environment values into disk.
- Track every source for reload and auth-cache freshness. Extend config-edit feedback to active fragments, distinguishing a file edit from an effective setting change.
- Supply documented split examples, merge/error rules and rollback instructions without migrating the user's live files.

## Capabilities

### New Capabilities

- `config-composition`: deterministic TOML composition, safe preference persistence, source-aware reload and actionable diagnostics.

### Modified Capabilities

None.

## Impact

Primary surfaces: `crates/jcode-base/src/config.rs`, `crates/jcode-base/src/config/{config_file,default_file,change_report}.rs`, `crates/jcode-base/src/config_tests.rs`, and `crates/jcode-app-core/src/tool/config_edit_notice{,_tests}.rs`. Inventory all `Config::save` callers and root-only migrations before implementation. Reuse installed `toml_edit` and existing reload/reporting modules rather than add a configuration framework.

Related but not dependent: archived `configure-task-and-worktree-policy` owns typed `[tasks]` and `[worktrees]` behavior. This feature preserves and deserializes those typed tables, but neither reimplements nor removes them. Serialize edits if both features touch shared configuration files. No implementation prerequisite changes are selected.

This is a standalone user-requested feature, not a new binding or completion claim for any outcome in `docs/plans/PRODUCT_ROADMAP.md`. No roadmap or tracker updates are included.

## Non-goals and boundaries

No automatic migration, live configuration changes, profile activation, table renaming, global unknown-key rejection, recursive includes, globs, remote includes, environment interpolation in paths, new project config, MCP JSON changes, Desktop work, daemon restart, deployment or remote publication. A later explicit migration request can split the user's actual files after this contract is implemented and verified.

Feature authoring does not authorize Apply. The proposed defaults below are ordinary design choices, not recorded user decisions. Verification and independent review precede any implementation commit and archive. Authoring evidence and operation eligibility are in `research.md`.
