# Probe Recipes

> Concrete rung-2/rung-3 probes per surface. All designed to be rerunnable and residue-free.
> Stdin payload shapes: `docs/research/hook-stdin-reference.md`. Run from the primary
> `~/dev/claude` checkout, never a `/apply` worktree.

## 1. Hook script — direct execution with synthetic stdin

```bash
# PostToolUse example (validate-file-hook.sh expects tool_name + tool_input.file_path):
printf '%s' '{"session_id":"probe","transcript_path":"/dev/null","hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"/tmp/probe-target.sh"}}' |
  ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/validate-file-hook.sh; echo "exit=$?"
```

Build the payload from the hook's own header comments + the stdin reference doc. Paste stdout
AND exit code. A hook that emits JSON decisions (`{"suppress":true}`, `{"decision":"block"}`)
must be probed on BOTH branches (matching and non-matching input).

## 2. Hook wiring — real-event fire evidence

Trigger the event once, then look for the fire:

| Event | Cheap deliberate trigger | Fire evidence |
| --- | --- | --- |
| PostToolUse (Write\|Edit) | Edit a scratch file in `/tmp` via the Edit tool | Newest telemetry entry for the hook / its side-effect (e.g. design-gate-nudge output in transcript) |
| SubagentStart | Dispatch one trivial `Explore` agent | `~/.claude/state/skill-list-injected.<session>.<turn>` marker mtime; telemetry entry |
| SessionStart | Open a throwaway session (or rely on THIS session's transcript) | Grep current transcript for the primer's emitted line |
| Stop | End a scratch session | `session-closer` effects: `bd dolt push` log lines, state file mtimes |
| PreToolUse (Bash) | Run any Bash command | gate.sh side-channel: rtk rewrite visible in executed command, gate logs |

```bash
# generic transcript fire-grep (current project, newest session):
T=$(ls -t ~/.claude/projects/"$(echo "$HOME/dev/claude" | tr '/' '-')"/*.jsonl | head -1)
grep -c '"hookEvent":"PostToolUse"' "$T"
```

**Dead-event differential**: if your hook shows 0 fires, check the always-on `telemetry.sh`
count on the SAME event over the same window. telemetry>0 + yours=0 -> your matcher/entry is
wrong. Both 0 -> the event itself is dead on this CC version (SubagentStop precedent) —
migrate events, do not fiddle the matcher.

## 3. settings.json key

```bash
# rung 1 — the key exists on the right hook OBJECT (not just anywhere in the file):
jq '.hooks.PostToolUse[] | select(.hooks[].command | test("validate-file-hook")) ' ~/.claude/settings.json
# rung 3 — exercise the gated path: e.g. for continueOnBlock, write a scratch file that fails
# validation, confirm the turn receives the block reason as context and CONTINUES.
```

After ANY bulk settings.json rewrite, re-run the probe for every hook with a
`# requires-settings:` header (or just `validate-cc --tier 3` — the hook-contract check walks
them).

## 4. session-primer / SessionStart

```bash
# rung 2 — direct run, paste emitted lines:
CLAUDE_PROJECT_DIR=~/.claude ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/session-primer 2>&1 | head -20
# rung 3 — wiring proof: the line landed in a real session:
T=$(ls -t ~/.claude/projects/"$(echo "$HOME/dev/claude" | tr '/' '-')"/*.jsonl | head -1)
grep -o '\[Ratchet\][^"]*' "$T" | head -3
```

Timing claims (e.g. "removed the second bd ready call") need `time` output before/after.

## 5. Statusline

```bash
jq -r '.statusLine.command // .statusline' ~/.claude/settings.json   # exact invocation
<that command>                                                        # paste rendered output
# pulse-source freshness (stale-while-revalidate caches):
ls -la ~/.claude/state/*pulse* 2>/dev/null                            # cache mtime within TTL?
```

First-ever render legitimately shows nothing (cache cold) — prime it, then probe.

## 6. Preprocessor / detection scripts

The contract is exit-0-under-failure, because a non-zero aborts the whole command render:

```bash
S=${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/<script>
time $S --json | python3 -m json.tool >/dev/null && echo "valid JSON"
# failure-mode execution (the ONLY valid audit — never lexical grep):
( cd /tmp && env -i HOME=$HOME PATH=$PATH $S --json ); echo "exit=$?"   # broken cwd/env precondition
# expect: exit=0, single JSON object with an "error" key
```

GATE scripts (`openspec-status --closure-check`) are the documented exception — probe that
they DO exit non-zero on a planted finding.

## 7. Command markdown

Invoke once with scratch args; then confirm in the transcript: (a) the bang-fenced block's JSON
rendered into the command body, (b) the pinned model served the turn:

```bash
T=$(ls -t ~/.claude/projects/"$(echo "$HOME/dev/claude" | tr '/' '-')"/*.jsonl | head -1)
grep -o '"model":"claude-[^"]*"' "$T" | sort -u
```

## 8. Agent definition

Dispatch on a trivial in-domain task, then:

```bash
M=$(ls -t ~/.claude/projects/*/*/subagents/agent-*.meta.json | head -1)
python3 -c "import json;d=json.load(open('$M'));print(d.get('agentType'), d.get('model',''))"
```

Confirms the name resolves (no ghost-dispatch), the model pin took, and — via the agent's
transcript — that its frontmatter skills injected.

## 9. Skill triggering

Auto-triggered: phrase a natural request containing the skill's trigger keywords in a scratch
turn; evidence = the skill body present in context / skill-load transcript entry.
Explicit-only: evidence = the citing `Skill()` call site resolves (`dangling-skill-ref` ratchet
covers this continuously — cite its pass).

## 10. Pre-commit guards

```bash
cd /tmp && git init probe-repo -q && cd probe-repo
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/guard-install . >/dev/null
echo 'x' > FOO_SUMMARY.md && git add FOO_SUMMARY.md
git commit -m probe; echo "exit=$?"        # paste the rejection + non-zero exit
cd /tmp && rm -rf probe-repo               # residue-free
```

Never probe guards in a real repo with staged work. If the target repo's hook has a beads
managed block, verify your install landed BEFORE the `BEGIN BEADS INTEGRATION` marker.

## 11. systemd user timer (ratchet lane et al.)

```bash
systemctl --user list-timers 'ratchet*' --no-pager
systemctl --user start ratchet-validate.service 2>/dev/null   # one manual run (check unit name first)
ls -la ~/.claude/state/ratchet-last-run.json                  # mtime just updated = fired
```

## Evidence block template

```
ASSERT: validate-file-hook block feeds back as context (continueOnBlock)
PROBE:  recipe 3 rung-3 — scratch file with planted syntax error via Edit
OUTPUT: <pasted stdout / transcript line>
EXIT:   0, turn continued with correction context
RESIDUE: /tmp/probe-target.sh removed
```
