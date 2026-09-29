# Jev + Jcode: Feature-by-Feature Integration Breakdown

## ADOPT — Priority 1-3 (build now)

### #1 evaluate tool (Adopt)

**What makes it worth it:** Jev gives Jcode a _probabilistic common-sense primitive_. Every agent currently uses LLM calls for simple yes/no judgments, parsing prose, and hoping the format holds. The `evaluate` tool replaces all `"ask LLM → parse text → hope"` workflows with `POST /v1/systemone → typed probability → branch`. Cost per call: ~$0.0002 vs ~$0.003 for LLM equivalent. Latency: ~300ms vs ~2s.

**How to incorporate:** Two mutually reinforcing paths:

- **Path A — native Rust tool** (~200 lines): Define Serde types for the TypeSafe API schema. Add `tool/evaluate.rs` with a single `evaluate(state, questions)` function. Agents call it like any other tool. Fallback: return structured errors if Jev is unavailable.
- **Path B — bundle `evaluate` MCP** (Go binary): Copy `itsmostafa/typesafe-mcp` into `jcode-bundled-servers/`. Register it as a built-in MCP server so every agent session has it automatically. `evaluate setup mcp` runs on first launch.

```rust
// Proposed signature
pub struct EvaluateInput {
    pub state: serde_json::Value,   // plain text or structured JSON
    pub questions: HashMap<String, Question>,
}
pub enum Question {
    Noul { instructions: String },
    Choice { instructions: String, criteria: HashMap<String, String> },
    Score { instructions: String, criteria: Vec<String> },
}
```

**Concrete use cases in Jcode today:**
- "Is this session going off-track?" (Noul)
- "Which MCP server should handle this request?" (Choice)
- "How urgent is this error?" (Score)
- "Does this diff hunk contain a security concern?" (Noul)

---

### #2 Jev-driven context compaction (Adopt)

**What makes it worth it:** Current LLM summarization loses critical details — file paths, exact error messages, constraints, command outputs. Jev compaction _never rewrites anything_. It scores each tool call for continued relevance and only drops or truncates calls the model is confident are obsolete. Cost: ~$0.002 per compaction (50-100 tool calls) vs ~$0.02 for an LLM summary. Speed: ~500ms vs ~3s. The fast-jev-compaction paper shows a staged fitting algorithm that handles Jev's 32k token limit gracefully.

**How to incorporate:** Add `JevCompactor` alongside the existing LLM compactor in `agent/compaction.rs`:
1. Pair `tool_use` ↔ `tool_result` by ID; pin recent N messages
2. Build state from conversation history (tool results replaced with `ok, N chars`)
3. Stage-fit into 25k tokens using fast-jev-compaction's progressive truncation
4. For each call: two Noul questions — "should the call stay?" and "should the result stay?"
5. Apply decisions: keep both / keep call + truncate / drop
6. Gate on `keepThreshold` (default 0.5)

**The algorithm to steal from fast-jev-compaction:**
```
1. collectToolCalls(transcript) → paired calls+results
2. fitState(state, 25k tokens) → staged truncation
3. batchCalls(calls, 30k tokens/request) → batched Noul questions
4. decideCall(result, threshold) → keep/truncate/drop
5. applyDecisions(messages, decisions) → rebuilt message list
6. reductionRatio(result) → gate on value (skip if < 25%)
```

---

### #3 Jev browser element selection (Steal Architecture)

**What makes it worth it:** jev-ultrafast proves: DOM snapshot → Jev picks operation + element → execute. **7.1s for a real Google Flights search.** 101 browser protocol calls instead of 1,092. ~25% faster than prior approach. The core insight: Jev makes _two decisions in one network round trip_ (operation + target) using speculative fan-out. No screenshots needed — pure structured state.

**How to incorporate into Jcode's `tool/browser.rs`:**

Add `action="jev_select"` with a natural-language `goal` parameter:

```python
def choose(state, goal, history):
    elements, targets, controls = action_space(state["actions"])
    # Build dynamic action space for this page
    operations = {"CLICK": "...", "TYPE_TEXT": "...", "SELECT": "..."}
    questions = {
        "operation": {"type": "choice", "criteria": operations},
    }
    # Speculative fan-out: include target heads for every possible operation
    for operation, candidates in targets.items():
        questions[operation.lower() + "_target"] = {
            "type": "choice", "criteria": {
                index: {"element": el["label"], "role": el["role"]}
                for index, el in candidates.items()
            }
        }
    # ONE TypeSafe request returns operation + matching target
    result = post_json("https://api.typesafe.ai/v1/systemone", key, body)
    # Only the selected operation's target head executes
    operation = result["answers"]["operation"]["choice"]
    target = result["answers"][operation + "_target"]["choice"]
    execute(operation, target)
```

**Key integration points:**
- Replace/extend `snapshot` action to produce an indexed element table
- Add `jev_select` action wrapping the choose→execute loop
- For TYPE_TEXT: use the main LLM (already available) to generate text
- Validate DOM freshness before executing (jevs-ultrafast's `StalePage` pattern)
- Fall back to existing `click`/`type`/`snapshot` actions if Jev is unavailable

**Required DOM snapshot format (adapt from jev-ultrafast's `snapshot.js`):**
```json
[
  {"id": "e1", "kind": "click", "node": "n1", "label": "Change ticket type · Round trip", "role": "button"},
  {"id": "e2", "kind": "fill", "node": "n2", "label": "Where from?", "value": "San Francisco", "role": "combobox"},
  {"id": "e3", "kind": "select", "node": "n3", "label": "Sort by", "current_value": "Price", "options": [...]}
]
```

---

## CONSIDER — Priority 4-6 (investigate, prototype)

### #4 Model tier routing (Consider)

**What makes it worth it:** Saves 40-60% on API costs by routing simple tasks ("what does this file do?") to cheap models and complex tasks ("implement auth middleware") to strong models. jev-router proves it works on Claude Code and Codex. The key insight: Jev decides per-turn, not per-session.

**How to incorporate:** Build as provider middleware in Jcode's provider dispatch layer:
1. Intercept the first request of each user turn
2. Send prompt text (only) to Jev with tier classification questions
3. Map result to configured model tiers (Jcode uses many providers, so tier mapping is per-provider)
4. Route to selected model; tool-loop continuations use the same tier

**Adaptation needed:** jev-router uses a proxy pattern. Jcode should use middleware so it works across all providers natively. The tier taxonomy would need to be per-provider-config, not hardcoded.

---

### #5 Worker supervision / Foreman (Consider)

**What makes it worth it:** Independent Jev observer catches stuck workers, off-track work, and AGENTS.md drift _before_ they waste thousands of tokens. Foreman's 10-dimension assessment in one Jev call is the right pattern. But Jcode's swarm is more complex than Foreman's single-worker model.

**How to incorporate (if we build it):**
- Run periodic Jev assessment on each active swarm worker (every 30s by default)
- Send compact observation: original job, recent worker history tail, git status, bounded diff
- 10 Noul questions in one request per assessment cycle
- Deterministic policy decides: continue / steer / stop / verify / escalate
- Start with the simple dimensions: `worker_stuck`, `work_off_track`, `needs_human`

**Risk:** Jcode's swarm already has sophisticated orchestration. Adding Jev might create conflicting control signals. Start with a passive observer that reports but doesn't intervene.

---

### #6 Jev code review (Consider)

**What makes it worth it:** Jev screens diffs and codebases with calibrated probabilities across 5 dimensions (correctness, security, reliability, compatibility, test coverage). jev-review shows a 6-stage pipeline: Noul risk matrix → Choice/Score file profiles → Choice evidence → Score severity → Choice routing. ~10-50× cheaper per review than LLM-based review.

**How to incorporate:** Enhance Jcode's existing `/review` skill with a Jev pre-screen:
1. Run Noul risk matrix on each diff hunk (5 questions: correctness, security, reliability, compatibility, test_gaps)
2. Score any hunk that triggers >0.7 on any dimension
3. Route high-risk hunks to full LLM review; low-risk hunks get Jev-only assessment
4. Present results in the existing review format with confidence scores

---

## BUNDLE/BUILD — Priority 7

### #7 Shell history (Bundle)

**What makes it worth it:** Dead simple Jev integration — on Tab, send current command prefix + recent history → Jev picks best match. ~$0.0001 per completion. The mrnugget/jev-shell-history repo is Go and could be bundled as a Jcode sub-command.

**How to incorporate:** Either bundle the Go binary or re-implement in Rust as a `jcode shell-history` subcommand. Low risk, low effort, immediate quality-of-life improvement for power users.

---

## BONUS: Graft + Jev Augmentation Strategy

### What TrailHQ Graft Is

[Graft](https://github.com/NanoNets/context-graph-engine) (MIT, by NanoNets/Trail) is an open-source **context layer for large codebases**. Key properties:

- **Tree-sitter parsing** across 20+ languages — deterministic, no API key needed
- **Two-pass LLM enrichment** — per-file summaries → grouped into semantic nodes with typed links (uses, produces, configures, validates)
- **Markdown graph stored as files** — no vector DB, no embeddings, no server. Agents read it like any other file.
- **Automatic refresh** — queries rebuild against working tree (~3ms when nothing moved)
- **Real benchmarks** — 46% fewer tool calls, 42% fewer tokens, 60% less time on 162-task sweep; 66% vs 54% correctness on SWE-bench Verified (50 instances)
- **MCP server** with 6 tools: `ask`, `grep`, `callers`, `skeleton`, `map`, `check`
- **Any model/any key** — OpenAI, Anthropic, OpenRouter, local models

**The graph format for "cron.ts":**
```markdown
# cron
→ schedules jobs using node-cron
→ uses logger, config, db
→ configures cron expressions from config.schedules
→ validates job handlers exist before scheduling
→ tested by cron.test.ts
```

### Jev + Graft: The Augmentation

Graft builds the structural graph. What if Jev could make _semantic_ judgments over that graph?

**1. Impact analysis with Jev confidence scores**
```
graft callers cron.ts → [scheduler.ts, api.ts, workers.ts]
Jev: "If cron.ts changes, which caller is most likely to break?" →
      scheduler.ts: 0.78, api.ts: 0.45, workers.ts: 0.12
```
Graft shows you _what depends on what_. Jev tells you _how risky_ each dependency is.

**2. Task → node routing**
```
User: "Add rate limiting to the API"
Graft ask → [api.ts, middleware.ts, config.ts]
Jev: "Which of these nodes actually needs changes for rate limiting?"
  → api.ts: 0.91, middleware.ts: 0.88, config.ts: 0.72
```
Graft finds relevant nodes. Jev prunes false positives.

**3. Code review over graph clusters**
```
git diff → changed files
Graft map → affected graph nodes + dependency clusters
Jev on each cluster: "Does this change break any contracts its dependents rely on?"
  → auth_cluster: 0.12 (safe), payment_cluster: 0.89 (risky)
```

**4. Jev as Graft's enrichment accelerator**
Graft currently uses an LLM (user's key) for the two-pass enrichment: per-file summaries and node grouping. Jev could replace the per-file summary pass with fast, cheap Choice/Noul questions:
```
Jev Choice: "Which system does this file belong to?" → [auth, payment, config, ...]
Jev Noul: "Does this file expose a public API?" → 0.83
```
This would make `graft build --deep` faster and cheaper, especially for large repos.

**5. Graft-as-Jcode-integration**
Graft's MCP server (`graft mcp`) is already an MCP tool. Jcode could:
- Auto-install Graft MCP on `jcode init` for repo-sized projects
- Use Graft's `ask` tool to preload context before each agent turn (like Claude Code does)
- Add Jev-driven relevance scoring on top of Graft's results



**Integration surface for Jcode:**
```
Jcode agent turn
  → Graft: pull relevant nodes from the code graph (structural)
  → Jev: score each node for task relevance (semantic)
  → Inject top-N nodes into context
  → Agent works with pre-loaded structural understanding
```

This combines Graft's deterministic tree-sitter code graph with Jev's probabilistic semantic judgment — structure + meaning, both cheap and fast.

### Prior Art: Context7 vs Graft

**Context7** (Upstash): pulls _library documentation_ from external sources — npm packages, API docs, SDK references. External → internal context flow.

**Graft** (NanoNets/Trail): builds a _codebase-specific graph_ from your own source files. Your code → graph.

They're complementary: Context7 tells you how to use a library; Graft tells you how your own code uses it.