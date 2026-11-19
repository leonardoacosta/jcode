# Config composition

`~/.jcode/config.toml` can list extra TOML files to merge. It is opt-in: a
config without an `include` key **loads** exactly as before (the file is parsed
as plain TOML and is never rewritten by a load). **Saving** changed in this
release for every install, composed or not: see "Saving: root overrides". jcode
never scans a directory, never creates fragments, and never migrates an existing
file.

## Syntax

Put a top-level `include` array of explicit relative paths in the root file:

```toml
include = ["config/ui.toml", "config/providers.toml"]

[display]
centered = true
```

Each entry must be:

- a relative path to a `.toml` file, resolved from the directory that holds
  `config.toml` (not from the current directory);
- inside that directory after symlinks resolve (a symlink that stays inside is
  fine, one that escapes is rejected).

Rejected: absolute paths, `..`, `~`, `$VAR`, globs (`*`, `?`, `[`, `{`),
duplicates, the root file itself, and nested `include` keys inside a fragment.
Only one include level exists. The `include` key is the manifest, not a setting,
and is never part of the typed config.

## Precedence

Lowest to highest:

1. Built-in defaults.
2. Fragments, in `include` order (later fragment wins).
3. The root file's own settings.
4. Environment variable overrides (applied last, as for a single file).

Tables merge recursively. Scalars and arrays replace: an array in a later source
replaces the earlier array whole (an empty array clears it). Unknown keys are
kept, as in a single file.

If two sources give one key different shapes (a table in one, a scalar in the
other) loading fails and names the key and both files.

## Errors

A missing, unreadable, unparsable, escaping or conflicting source fails the whole
composition. The message names the file and `line:col` or key, never a value or
excerpt. There is no partial merge:

- UI and display reads fall back to whole-config defaults (as for a broken single
  file) and log the cause.
- Authorization-sensitive reads **deny** instead: trusted external auth sources,
  hook execution (a `pre_tool` gate blocks the tool call), and unattended
  task policy. Fixing the file restores them on the next check.

## Saving: root overrides

jcode setters (`/config`, effort and theme pickers, migrations) write **only the
root file**. A save compares the new settings with a fresh read of the same
sources and writes just the changed leaves into the root, so:

- a value a fragment also sets is overridden in the root, and the fragment is
  untouched (byte-identical);
- comments, the `include` manifest, unknown keys and tables are preserved;
- a save that changes nothing writes nothing;
- environment overrides are never written into the file;
- only the requested leaf changes in the root text: comments, key spacing,
  dotted keys, inline tables, literal strings, unknown keys and a consistent
  CRLF or LF line style stay as written. Earlier versions re-serialized the whole
  file with every default filled in. Now defaults are not written, so a root saved
  by this version is shorter than one saved by an older version.

Two save outcomes to know:

- **Arrays are whole values.** Removing an array a fragment provides (a model
  list, `trusted_external_sources`) cannot make the root "inherit" again. The
  save writes an explicit empty array in the root, which overrides the
  fragment's list. Delete that root line by hand to get the fragment's list back.
- **Unsetting an optional value a fragment provides fails** with
  `Cannot remove config key ...: its value is inherited from <file>`, since TOML
  has no null. Edit that fragment.

Override removal is manual. To drop a root override so the fragment value applies
again, delete that line from the root.

### Atomic saves

The root is written to a temporary file in the same directory, fsynced, and
renamed over the target. Permissions are kept, and a symlinked `config.toml` is
replaced at its target so the link survives. If the root or any selected
fragment changed between the read and the rename, nothing is written and the
error asks you to retry. On any write or rename failure the original bytes and
permissions remain and the temp file is removed.

Limit: this detects edits that landed before the rename. An editor that writes
the root during the final rename itself can still win or lose the race, as with
any file replacement. Edits made by hand while jcode is saving are not locked.

### Notices never echo secrets

Change notices and error messages avoid values. Notices print `<redacted>` for
the before/after value of any key whose name contains `key`, `token`, `secret`,
`password`, `auth` or `credential` (and for arrays holding such keys). Parse and
type errors mask quoted, literal, multi-line and numeric values. `jcode policy`
uses the same typed, redacted loader and names the failing source file. Masking
is by key name, so a secret under an unrelated name (for example `cookie` or
`bearer`) can still print in a notice.

## Known limits: readers that ignore `include`

These read only the root `config.toml` and do not follow `include`. A setting
that lives only in a fragment is invisible to them until it is also in the root:

- `jcode-setup-hints` (`launch_hotkeys` and the macOS `[terminal] preferred`),
  which cannot depend on the composition code in `jcode-base`.
- `jcode-sdk` launch credential links, which copy only the root `config.toml`
  into an instance home. The instance gets no fragments, so a copied root with
  `include` lists files that are absent there and fails to load in that instance.

- `jcode provider add` (`src/cli/commands/provider_setup.rs`) edits the root
  `config.toml` as raw text with a plain write: it does not follow `include`,
  does not check for source drift and checks "profile already exists" against the
  root only. A profile defined only in a fragment is not seen as existing; the
  root entry wins on load. It never modifies a fragment.
- The SSH host `/config view` and `/config set` commands
  (`server/runtime_ssh_host.rs`) read and re-render the root only
  (`toml::to_string_pretty`, so root comments are dropped) and show no
  fragment-only values. Root overrides win on load, so `set` stays safe.

The router bundle export (`auth/router_transfer.rs`) does follow `include`, and
fails the export rather than dropping keys when a source does not compose.

## Broken composition and tool calls

If the sources stop loading, hooks and external-auth trust fail closed (a UI
defaults fallback never grants permission or runs hooks). For tool dispatch this
means calls are blocked only when hook policy could be hiding in the broken
config: the root has an `include` or a `[hooks]` table. A root with neither is
unaffected by an unrelated error elsewhere. Even then, `read`, `agentgrep`,
`conversation_search` and `session_search` run, and `edit`/`write` of the root
or a listed fragment run so the agent can repair the file. `bash` and every
other mutating tool stay blocked until the config loads. `JCODE_HOOKS_DISABLED=1`
(set inside hook children) suppresses hooks and this denial entirely.

The check rides the normal reload throttle, so a source broken by something other
than a tool edit can keep running hooks for up to 500 ms.

## Reload and edit feedback

The config cache re-checks the root and every listed fragment (content, not only
size or mtime, including added, removed, deleted and restored files). Equal-length
edits, changing the `include` list, and deleting then restoring a fragment are all
noticed. The trusted-auth check re-reads on each call, so revoking trust in a
fragment applies immediately.

When an agent writes the root or a listed fragment, the tool result reports the
changed keys and whether each is live. If a source stops loading, the notice says
jcode is falling back to defaults and names the file. Settings under `gateway`,
`acp` and `launch_hotkeys` are read once at startup and need a restart; the notice
says so and does not claim they reload live.

## Adopting and downgrading

Adopt by hand: move sections into files under `~/.jcode/config/`, then add the
`include` list. Run jcode once and check `/config`. A split layout and the same
settings in one file produce identical typed results.

Downgrade by pasting the fragment contents back into `config.toml` and deleting
`include`. Older jcode versions ignore an `include` key and so would not read the
fragments, which is why composition never happens implicitly.

## Workspace rules

`[[workspaces.rules]]` maps a git `origin` to a named workspace. Rules are read
only from the global config (the root file and its includes). A project
checkout can never add or change one.

```toml
[[workspaces.rules]]
name = "brown"                       # header badge: ws:brown
origin = "dev.azure.com/brown/**"    # glob over host/path, case-insensitive
tracker = "ado"                      # linear | ado | none
branch_template = "feat/<id>-<slug>"
skills = ["triage-ado"]
expected_email = "me@brown.example"  # checked, never written to git config
swarm_model = "claude-sonnet-5.5"    # worker model, or "inherit"
[workspaces.rules.roles]
research = "claude-haiku-5.5"
[[workspaces.rules.hooks.PreToolUse]]  # runs after global [hooks]
command = "~/bin/brown-gate"
```

- `origin` is matched against the credential-free `host/path` of `origin`
  (`user:token@`, port, `.git` and trailing `/` removed). `*` and `?` stay in
  one path segment, `**` crosses segments. The first matching rule wins. Write
  the glob as `host/path`, not as a URL.
- A rule applies to the directory a session or worker actually runs in. A
  spawned worker never inherits another workspace's hooks, hints, identity
  check or model from its coordinator.
- Spawn model precedence: explicit `model` > workspace role slot > workspace
  `swarm_model` > `agents.roles` > `agents.swarm_model` > inherit.
- Workspace hooks use lifecycle tables only and append to the global hooks for
  the same event (an entry with the same `command` replaces the global one).
- Invalid values are reported as `Workspace config warning:` log lines at load.
