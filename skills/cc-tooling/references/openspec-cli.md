# OpenSpec CLI Patterns (non-interactive)

Agents running `openspec` CLI must use non-interactive flags — the CLI reads confirmation prompts from `/dev/tty`, so `echo "y" |` and `printf` piping DO NOT work and produce loops.

| Operation | Correct invocation | Why |
|-----------|-------------------|-----|
| Validate spec | `openspec validate <name> --strict --no-interactive` | Hard gate before Phase 4 / handoff |
| Archive spec | `openspec archive <name> --yes` | `--yes` (long form, not `-y` in OR-chains) skips confirmation |
| Archive ADDED-only spec (new capability, no parent in `openspec/specs/`) | `openspec archive <name> --yes --skip-specs` | `--skip-specs` bypasses the "update main specs" write that fails for fresh capabilities |
| Archive when the CLI rejects the spec | Leave in `openspec/changes/<name>/`. Do NOT `mv` manually — the project `gate.sh` hook blocks direct archive moves and surfaces `BLOCKED: Use /archive SPEC_NAME` |

**Anti-patterns that burn loops:**
- `openspec archive <name>` (no flag) → hangs on confirmation
- `echo "y" \| openspec archive <name>` → prompts use /dev/tty, stdin is ignored
- `mkdir archive && mv openspec/changes/<name> openspec/changes/archive/` → blocked by gate.sh
- `openspec archive <name> -y ... \|\| openspec archive -y ... \|\| openspec archive --help` → OR-chains swallow the first success

**Rule of thumb for any interactive CLI in agent context:** before piping stdin or looping, run `<cmd> --help` ONCE and look for `--yes`, `--no-interactive`, or `--force` flags.
