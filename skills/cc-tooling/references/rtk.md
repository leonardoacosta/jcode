# RTK (Rust Token Killer)

Token-compression CLI proxy, integrated through rtk's own surface: a PreToolUse hook installed by
`rtk init -g`, plus project-local filters in a trusted `.rtk/filters.toml`. cc previously ran a
hand-rolled rewrite path inside `gate.sh`; that fork is retired
(`docs/adr/0007-retire-gate-rtk-rewrite.md`).

**rtk is on trial, not adopted.** Scope, window, and the promote-or-uninstall bar:
`docs/rtk-upstream-trial.md`. Revert is one command: `rtk init -g --uninstall`.

## What the hook does and does not rewrite

The hook rewrites commands rtk ships built-in support for (`git`, `rg`, `ls`, file reads, …). Two
carve-outs matter:

- `~/.config/rtk/config.toml` `[hooks] exclude_commands = ["cat","head","tail"]` exempts bare file
  reads, which upstream issue #582 closed as cost-negative. The exclusion **leaks** on
  argument-bearing shapes — `head -20 f`, `tail -n 5 f`, `tail -5 f` are still rewritten to
  `rtk read`.
- **Project-local `.rtk/filters.toml` filters do NOT fire through the hook.** They apply only on
  the explicit `rtk <cmd>` path. `bd list` runs uncompressed; `rtk bd list` is filtered. Reach for
  the `rtk` prefix deliberately when you want a project filter.

**Live footgun — `find` with compound predicates.** The hook rewrites a bare `find ...` to
`rtk find ...`, and `rtk find` rejects `-o`, `-not`, and `-exec` with
`rtk: rtk find does not support compound predicates or actions`, exit 1. cc's own bypass for this
was deleted with `gate.sh`'s rewrite path and upstream ships no equivalent. Workaround: put the
`find` behind another command in the same call (`cd . && find ... -exec ...`) — the hook skips
compound shell strings — or invoke `/usr/bin/find`.

Check any command before assuming: `rtk hook check "<cmd>"` prints the rewrite, or
`No rewrite for: <cmd>` and exits 1. It is a verdict, not a probe — a non-zero exit means "no
rewrite", not "broken".

## Project filters (`.rtk/filters.toml`)

Line-level actions only — `strip_ansi`, `strip_lines_matching`, `keep_lines_matching`, `replace`,
`max_lines`, `truncate_lines_at`, `on_empty`. There is no cross-line reformat action, so rules
that need to collapse a multi-line record into one line cannot be expressed here.

Two gotchas that fail silently:

1. **`schema_version = 1` is mandatory and undocumented.** Upstream's `src/filters/README.md`
   never mentions it. Without it, `rtk verify` prints `No inline tests found.` and **exits 0** —
   it fails open. Only the `N/N tests passed` line is evidence; the exit code is not.
2. **Trust is SHA-256-bound.** Every edit silently disables the file until it is re-trusted, with
   no warning on the command path — compression just stops. A fresh clone starts untrusted;
   `rtk trust` is per-machine, not per-repo.

```bash
rtk verify --filter <name>          # must print "N/N tests passed"
printf 'y\n' | rtk trust            # re-trust after ANY edit — `--yes` does not exist (exit 2)
rtk trust --list                    # confirm the repo path + sha256
```

## Measuring whether it pays

Use ccusage billing over the trial window — method, commands, and the numeric bar live in
`docs/rtk-upstream-trial.md`. rtk's own savings ledger (`~/.local/share/rtk/history.db`, and every
report reading it) overstates savings badly: bead `cc-w83ov.217` traced 98.5% of its reported
30-day savings to 148 outlier rows.

Do not use `rtk discover` for anything. Claude transcripts store the command *before* the
PreToolUse hook rewrites it, so `discover` reports misleading missed-savings/fake-zero results.

## Verification

```bash
rtk --version         # Should show: rtk X.Y.Z
which rtk             # Verify correct binary
rtk init --show       # Hook / RTK.md / settings.json install state
```

Name collision: if rtk's subcommands are unrecognized, check for reachingforthejack/rtk
(Rust Type Kit).
