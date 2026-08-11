# Query Cookbook

> Copy-ready extraction recipes per store. All read-only. Paths assume this machine
> (`~/.claude` = `~/dev/claude`). Adjust windows per SKILL.md § Standard Procedure.

## 1. Command invocations (`~/.claude/telemetry/command-invocations.jsonl`)

Frequency by command, last 90 days:

```bash
python3 - <<'EOF'
import json, datetime, collections, pathlib
cutoff = (datetime.datetime.now() - datetime.timedelta(days=90)).timestamp()
counts = collections.Counter()
p = pathlib.Path.home() / '.claude/telemetry/command-invocations.jsonl'
for line in p.open():
    try: d = json.loads(line)
    except Exception: continue
    ts = d.get('ts') or d.get('timestamp') or 0
    if isinstance(ts, str):
        try: ts = datetime.datetime.fromisoformat(ts.replace('Z','+00:00')).timestamp()
        except Exception: continue
    if ts >= cutoff: counts[d.get('command') or d.get('name','?')] += 1
for cmd, n in counts.most_common(40): print(f'{n:6d}  {cmd}')
EOF
```

Zero-invocation commands = live registry minus the counter's keys (compare against
`find ~/.claude/commands -name '*.md' | grep -viE '/references/|readme|/archive/|/rules/'`).

## 2. Agent dispatches (`projects/*/*/subagents/agent-*.meta.json`)

Prefer the harness: `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/agent-dispatch-census --json` — emits
`zero_dispatch` + `orphans` over a rolling 90d window. Raw fallback (per-type counts, 90d by
mtime):

```bash
find ~/.claude/projects -path '*/subagents/agent-*.meta.json' -mtime -90 -print0 |
  xargs -0 -I{} python3 -c "import json,sys;print(json.load(open('{}')).get('agentType','?'))" 2>/dev/null |
  sort | uniq -c | sort -rn | head -30
```

Caveat: `agentType` is the dispatch name; built-ins (`general-purpose`, `claude`, `Explore`,
`Plan`, `claude-code-guide`, `statusline-setup`) have no `agents/*.md` and are NOT orphans.

## 3. Hook liveness (transcripts + telemetry)

Did event/hook X fire in the last 30 days?

```bash
grep -l '"hookEvent"' ~/.claude/projects/*/*.jsonl 2>/dev/null | head  # sessions with hook entries
# count fires of one event across recent sessions:
find ~/.claude/projects -name '*.jsonl' -mtime -30 -not -path '*/subagents/*' -print0 |
  xargs -0 grep -ho '"hookEvent":"[^"]*"' 2>/dev/null | sort | uniq -c | sort -rn
```

Denominator (matched opportunities): count the dispatches/tool-calls the matcher would have
matched in the SAME window. The canonical always-on signal for SubagentStart volume is the
empty-matcher telemetry hook's own fire count. Independent cross-check example:
`~/.claude/telemetry/agents-active.json` — entries that never deregister mean the
deregistration-hosting event never fires.

## 4. MCP usage (`~/.claude.json` toolUsage)

```bash
python3 - <<'EOF'
import json, datetime, pathlib, collections
cutoff = (datetime.datetime.now() - datetime.timedelta(days=90)).timestamp() * 1000
d = json.load((pathlib.Path.home() / '.claude.json').open())
per_server = collections.defaultdict(lambda: [0, 0.0])   # [count, last_used_ms]
for tool, u in (d.get('toolUsage') or {}).items():
    if not tool.startswith('mcp__'): continue
    server = tool.split('__')[1]
    per_server[server][0] += u.get('usageCount', 0)
    per_server[server][1] = max(per_server[server][1], u.get('lastUsedAt', 0) or 0)
for s, (n, last) in sorted(per_server.items(), key=lambda kv: -kv[1][0]):
    when = datetime.datetime.fromtimestamp(last/1000).date() if last else 'never'
    live = 'LIVE' if last >= cutoff else 'quiet>90d'
    print(f'{n:6d}  last={when}  [{live}]  {s}')
EOF
```

A server with history but `lastUsedAt` outside the window is "gone quiet," a distinct state
from "never called" — report which.

## 5. RTK (`~/.local/share/rtk/history.db`)

```bash
# Execution counts are trustworthy. The savings column is NOT — see the warning below.
# The time column is `timestamp` (TEXT, ISO-8601) — NOT an epoch `ts`. There is an index on it.
sqlite3 ~/.local/share/rtk/history.db \
  "SELECT count(*) FROM commands WHERE timestamp > datetime('now','-30 days');"
# per-command breakdown (same caveat applies to its Saved column):
rtk gain
```

This db is rtk's rewrite ledger only. It proves rewrite **adoption** — how often a rewrite fired.

> **Never cite `saved_tokens` (or `rtk gain`'s Saved column) as a benefit figure.** It is rtk's own
> estimate of what output *would* have cost, and it cannot report a loss. Bead `cc-w83ov.217`
> traced 98.5% of a 30-day total to 148 outlier rows — one `rg -n` claimed 1.2 billion tokens
> saved. For cost questions use ccusage billing instead; the method is written up in
> `docs/rtk-upstream-trial.md` § How to measure.

## 6. Session pattern mining (`projects/<proj>/*.jsonl`)

Prefer `/workflow:retrospect` (wraps `sequence-mine` for 2–4-step command windows +
`state/failures/*.jsonl`). Raw per-session tool-frequency:

```bash
python3 - <<'EOF'
import json, glob, collections, os
counts = collections.Counter()
_proj_slug = os.path.join(os.path.expanduser('~'), 'dev', 'cc').replace('/', '-')
files = sorted(glob.glob(os.path.expanduser(f'~/.claude/projects/{_proj_slug}/*.jsonl')),
               key=os.path.getmtime)[-20:]          # last 20 sessions
for f in files:
    for line in open(f):
        if '"tool_use"' not in line: continue
        try: d = json.loads(line)
        except Exception: continue
        for c in (d.get('message') or {}).get('content') or []:
            if isinstance(c, dict) and c.get('type') == 'tool_use':
                counts[c.get('name','?')] += 1
for t, n in counts.most_common(25): print(f'{n:6d}  {t}')
EOF
```

Rule: stream + filter; never Read a session jsonl whole, never follow `.output` symlinks.

## 7. Subagent return values (`subagents/agent-*.jsonl`)

Last text block of an agent's transcript = its deliverable:

```bash
python3 - AGENT_JSONL_PATH <<'EOF'
import json, sys
last = None
for line in open(sys.argv[1]):
    try: d = json.loads(line)
    except Exception: continue
    if d.get('type') == 'assistant':
        for c in (d.get('message') or {}).get('content') or []:
            if isinstance(c, dict) and c.get('type') == 'text' and c['text'].strip():
                last = c['text']
print(last or 'NO TEXT')
EOF
```

Workflow runs: read `journal.jsonl` in the run's transcript dir before diagnosing an empty
result — it records each agent() call's actual return.

## 8. Cost attribution

Model IDs seen recently, then rate-resolve via the shared lib (never hand-copy rates):

```bash
find ~/.claude/projects -name '*.jsonl' -mtime -30 -print0 |
  xargs -0 grep -ho '"model":"claude-[^"]*"' 2>/dev/null | sort | uniq -c | sort -rn
# resolution check: source scripts/lib/cost-rates.sh; _compute_cost <model> <in> <out>
# strip -20[0-9]{6} date suffixes; <...> sentinels are non-billable placeholders.
```

## 9. Failure patterns (`$HOME/.claude/scripts/state/failures/*.jsonl`)

```bash
cat $HOME/.claude/scripts/state/failures/*.jsonl 2>/dev/null |
  python3 -c "import json,sys,collections;c=collections.Counter(json.loads(l).get('category','?') for l in sys.stdin if l.strip());[print(f'{n:5d}  {k}') for k,n in c.most_common()]"
```

## 10. Remote-machine stores (auditing a machine the current session isn't running on)

The recipes above assume the session runs ON the machine whose stores you're reading (SKILL.md
rule 4: homelab is the primary machine). When a task needs evidence FROM homelab but the current
session is running elsewhere (or vice versa), data does not need to leave the target machine —
the 2026-07-21 homelab session audit established a **staged server-side extractor** pattern:
write a small stdlib-only Python script per query phase, run it over `ssh` on the target machine,
and have it return only the aggregated counts/metadata (never raw transcript content) as JSON.

Reference implementation: `hl-stage-a.py` / `hl-stage-b.py`, kept in a gitignored
`projects-archive/` directory (alongside their JSON outputs) beside the requesting repo — never
committed, since they're one-off extraction scripts scoped to a single audit's stores, not a
reusable tool. Stage split mirrors the two query classes: stage A does the census sweep
(`~/.claude/projects/`, `~/.claude/telemetry/`, `~/.claude/history.jsonl` — counts, sessions,
dispatch tallies), stage B does the failure/narrative mining (`$HOME/.claude/scripts/state/failures/`
— category breakdowns, transcript tail reads for root-cause isolation). Both run over `ssh
<target-host> python3 <script>`, stdout captured locally as the evidence to cite — the "asked
homelab a question over ssh, got JSON back" shape keeps the underlying transcripts on the
machine that owns them while still producing citable `N events in <window> per <store>` claims
per the Evidence Standards above.

## Reporting Template

```
CLAIM: /open invoked 0 times in 56d (window bounded by supersession date)
STORE: telemetry/command-invocations.jsonl
QUERY: <recipe 1, filtered command=="open">
CROSS-CHECK: roadmap-pulse statusline renders daily (recipe 4 N/A) — replacement absorbed the use case
=> RECOMMEND: retire tombstone per /next precedent
```
