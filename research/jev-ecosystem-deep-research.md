# Deep Research: Jev Ecosystem — Integration Opportunities for Jcode

## Executive Summary

TypeSafe AI's Jev — a System One model that returns typed, probabilistic decisions instead of free text — has exploded into a 30+ repo ecosystem within days. These repos cluster around three Jcode-relevant themes: **browser automation** (jev-ultrafast, 7.9k stars), **agent tooling** (fast-jev-compaction, 4k stars; foreman, 353 stars; jev-router, 191 stars), and **MCP/server integration** (typesafe-mcp/evaluate, 99 stars). The critical finding: Jev's architectural model — small, fast, parallel probabilistic judgments — maps directly onto Jcode's existing needs for browser element selection, context compaction, model routing, code review, and multi-source decision fusion.

TypeSafe provides a Python SDK (`typesafe-sdk`), JavaScript SDK, HTTP API (`POST /v1/systemone`), an MCP server (the `evaluate` CLI), and an official agent skill (`typesafe-ai/skills`). Jev is also available via OpenRouter (`~typesafe/jev-latest`). The API key you have in the `jev` repo DB works with both TypeSafe direct and OpenRouter.

**Recommendations priority-ordered:**

| Rank | Integration | Effort | Risk | Impact |
|------|-------------|--------|------|--------|
| 1 | **typesafe-mcp (evaluate) as built-in MCP tool** | Low | Low | High |
| 2 | **Jev-driven context compaction** | Medium | Medium | High |
| 3 | **Jev-driven browser element selection** | High | Medium | High |
| 4 | **jev-router-style model tier selection** | Medium | Medium | Medium |
| 5 | **foreman-style worker supervision** | High | High | Medium |
| 6 | **jev-review code review pipeline** | Medium | Low | Medium |
| 7 | **jev-shell-history integration** | Low | Low | Low |

## Key Findings

### 1. Jev Ultrafast — Browser Automation That Jcode Should Steal

**Repo:** [browser-use/jev-ultrafast](https://github.com/browser-use/jev-ultrafast) · 7.9k stars · Python · MIT

**Architecture:** A browser agent with a **dynamic, indexed action space**. The core insight: Jev selects both an **operation** and a **target element** in a single TypeSafe request, eliminating multi-step LLM reasoning loops.

```
page → DOM snapshot → element table ([1] button "Click me", [2] input "Search"...)
→ One TypeSafe request: operation (CLICK/TYPE_TEXT/SELECT/...) + specific target
→ Browser executes the chosen operation on the chosen element
→ Only TYPE_TEXT sends to a small LLM for text generation
→ Repeat
```

**Key metrics:** Zürich→London on Google Flights in **7.1 seconds**. 25% faster than the prior version, from **1,092→101 browser protocol calls**. Wikipedia article open in **2.8s**.

**How it works:**
- `snapshot.js` reads all visible controls (buttons, comboboxes, text inputs) atomically from the DOM, producing an indexed element table
- `model.py` constructs **speculative fan-out questions**: the operation question returns a choice across [CLICK, TYPE_TEXT, SELECT, SCROLL_UP, SCROLL_DOWN, WAIT, DONE, BLOCKED], and multiple target questions return different elements — only the target matching the selected operation executes
- No screenshots in the agent loop; Jev consumes structured state (element type, label, value, nearby text)
- One browser call per snapshot; execution validates DOM freshness and click occlusion before performing
- Text generation uses a small LLM (mercury-2.5 or similar) **only** for TYPE_TEXT operations

**Jcode relevance:** Jcode's `browser` tool (`source/jcode/crates/jcode-app-core/src/tool/browser.rs`) currently uses a Firefox/Chrome bridge provider with action-based commands (click, type, screenshot, snapshot, etc.). It does not do AI-driven element selection. Integrating Jev-driven selection would **radically accelerate Jcode's browser tool** by eliminating multi-turn "find the right element" loops. The DOM snapshot approach could be adapted to Jcode's existing Firefox bridge.

**Integration approach:**
1. Add a `jev_select` browser action that takes a natural-language goal
2. Run the DOM snapshot (Jcode already snapshots DOM as accessibility tree)
3. Send structured element table + goal to Jev via the evaluate tool
4. Execute the selected operation on the chosen element
5. Generate text via the main LLM (already available) for TYPE_TEXT operations

### 2. fast-jev-compaction — Context Management Jcode Should Adopt

**Repo:** [tamaratran/fast-jev-compaction](https://github.com/tamaratran/fast-jev-compaction) · 4k stars · TypeScript · MIT

**Architecture:** Replaces LLM-based context summarization with Jev-scored tool call pruning. Never rewrites text; only deletes or truncates tool calls/results that Jev says are obsolete.

**How it works:**
1. Every `tool_use` paired with `tool_result`; recent N messages pinned (never touched)
2. Full conversation state sent to Jev with tool results replaced by short notes (`ok, 4213 chars (omitted)`)
3. State fitted into 25k tokens via progressive truncation stages
4. For each non-pinned tool call, two Noul questions: "should the call stay?" and "should the result stay?"
5. Decisions: keep both, keep call + truncate result, or remove both
6. User and assistant text stays verbatim

**Jcode relevance:** Jcode already has context compaction (`source/jcode/crates/jcode-app-core/src/agent/compaction.rs`) that tracks messages, token budgets, and cache invalidation. The current approach appears to use summary-based compaction. Jev-driven compaction would:
- Never lose critical file paths, error messages, or constraints
- Be significantly cheaper than an LLM summary call
- Be faster (one Jev call vs an LLM round trip)
- Produce deterministic retention decisions with confidence scores

**Integration approach:**
1. Implement a `JevCompactor` alongside the existing LLM compactor
2. Use the TypeSafe API directly (Jcode is Rust; could use HTTP directly or add a thin Rust→TypeSafe bridge)
3. Gate on confidence: only drop calls where Jev is confident they're obsolete
4. Fall back to LLM summarization if Jev is unavailable

### 3. Foreman — Worker Supervision Architecture

**Repo:** [thruwire/foreman](https://github.com/thruwire/foreman) · 353 stars · Python · MIT

**Architecture:** A "software factory" with two concurrent asyncio loops:
- **Coding agent loop:** Codex worker does software engineering (reason→tool→observe→edit→test)
- **Foreman loop:** independently watches factory events, sends them to Jev for assessment, then a deterministic Python policy decides what to do

**Ten Jev assessment dimensions** (all Noul questions, one request):
| Dimension | What it measures |
|-----------|-----------------|
| `implementation_complete` | Is the required implementation done? |
| `tests_sufficient` | Are tests adequate? |
| `requirements_satisfied` | Does the repo satisfy the request? |
| `needs_verification` | Should an independent verifier run? |
| `ready_to_finish` | Should the factory declare completion? |
| `meaningful_progress` | Is the current worker advancing? |
| `worker_stuck` | Is the worker looping/failing? |
| `work_off_track` | Is work drifting from the job? |
| `agents_md_drift` | Is worker behavior inconsistent with AGENTS.md? |
| `needs_human` | Does this need human attention? |

**Policy actions:** CONTINUE, START_WORKER, START_VERIFIER, STEER_WORKER, STOP_WORKER, RETRY_WORKER, FINISH, ESCALATE

**Why it fits:** Foreman uses deterministic policy with Jev probability inputs. Jcode's swarm system already manages worker agents. Adding Jev-based supervision would:
- Detect stuck workers before they waste tokens
- Validate completion claims independently
- Route to human when needed

### 4. jev-router — Per-Turn Model Routing

**Repo:** [gargpratyush/jev-router](https://github.com/gargpratyush/jev-router) · 191 stars · TypeScript · MIT

**Architecture:** Launches Claude Code or Codex behind a loopback proxy. On each fresh user turn, sends the prompt to Jev to decide: Fast/Balanced/Strong/Long tier. The proxy rewrites the model header before forwarding to Anthropic/OpenAI.

**Routing tiers:**
| Tier | Claude Code default | Codex default |
|------|--------------------|---------------|
| Fast | Haiku | gpt-5.6-luna |
| Balanced | Sonnet | gpt-5.6-terra |
| Strong | Opus | gpt-5.6-sol |
| Long | Fable | gpt-6-astra |

**Policy rules in code:**
- Explicit user requests win (`use opus`)
- Failure/timeout keeps current model
- Low confidence never downgrades
- Large conversations refuse downgrades
- Unavailable tiers step upward

**Jcode relevance:** Jcode supports multiple models/providers already. A Jev-driven routing layer could:
- Auto-select model tier based on task complexity
- Save costs by routing simple tasks to cheaper models
- Integrate as a provider middleware rather than a proxy

### 5. jev-review — Structured Code Review

**Repo:** [devagrawal09/jev-review](https://github.com/devagrawal09/jev-review) · 331 stars · TypeScript · MIT

**Architecture:** Staged Jev judgments for code review:
1. **Noul risk matrix** — screen for correctness, security, reliability, compatibility, test coverage issues
2. **Choice + Score file profiles** — classify and rank files
3. **Choice evidence selection** — select relevant diff hunks/source regions
4. **Choice mechanism classification** — categorize the type of issue
5. **Score severity** — rate impact
6. **Conditional Choice reviewer routing** — route to appropriate reviewer

**Key features:**
- Change review mode (git diff) and codebase scan mode
- Uses changed/related tests as context for test gap judgments
- Local dashboard at `127.0.0.1:4317`
- Thresholds and workflow policy in code, not in prompts
- Structured hints, counterexamples, and explicit decision boundaries

**Jcode relevance:** Jcode's review skill (`/review`) currently uses LLM-based review. Jev-driven review would:
- Be 10-50x cheaper per review
- Provide calibrated confidence scores
- Enable systematic risk screening across multiple dimensions

### 6. typesafe-mcp (evaluate) — The Critical MCP Bridge

**Repo:** [itsmostafa/typesafe-mcp](https://github.com/itsmostafa/typesafe-mcp) · 99 stars · Go · MIT

**Architecture:** A single Go binary (`evaluate`) that exposes one MCP tool: `evaluate(state, questions)`. Sets up with one command: `evaluate setup mcp`.

**Tool schema:**
```json
{
  "state": "string | object | array",  // content to judge
  "questions": {
    "<id>": {
      "type": "noul | choice | score",
      "instructions": "string | object | array",
      "criteria": { ... }  // depends on type
    }
  }
}
```

**Routes:** Direct TypeSafe API (`TYPESAFE_API_KEY`) or OpenRouter (`OPENROUTER_API_KEY`). Auto-retries 429/529 with exponential backoff.

**Jcode relevance: CRITICAL.** This is the most straightforward integration path. Jcode's MCP module (`source/jcode/crates/jcode-app-core/src/tool/mcp.rs`) already manages MCP servers. Adding `evaluate` as a built-in MCP server would give every Jcode agent access to:
- Typed yes/no judgments with probabilities
- Choice selection from defined option sets
- Scored ratings on ordered scales

### 7. Jev Shell History

**Repo:** [mrnugget/jev-shell-history](https://github.com/mrnugget/jev-shell-history) · 63 stars · Go

**Architecture:** Zsh widget that on Tab, sends the current command prefix + recent history to Jev, which selects the best match. Returns the matching command for completion.

**Jcode relevance:** Jcode's `bash` tool doesn't have shell history integration. This is a nice-to-have, lower priority but dead simple to implement.

### 8. pg-jev — SQL-embedded AI Judgments

**Repo:** [realZachi/pg-jev](https://github.com/realZachi/pg-jev) · 207 stars · Python/C · MIT

**Architecture:** PostgreSQL extension that lets you call Jev from SQL queries:
```sql
SELECT * FROM tickets WHERE jev_noul(description, 'Is this ticket urgent?') > 0.7;
```

**Jcode relevance:** Medium. Jcode uses SQLite, not PostgreSQL. But the pattern of embedding judgments in query pipelines is powerful. Could inspire a SQLite extension or a local judgment cache.

## TypeSafe AI / Context7 Architecture

### What Jev Is

TypeSafe's Jev is a **System One model**: small, fast, returns typed judgments with probabilities instead of generating text. Three question primitives:

| Primitive | Returns | Use case |
|-----------|---------|----------|
| Noul | Probability (0-1) the statement is true | Yes/no judgments |
| Choice | Selected option + per-option probabilities | Routing, classification |
| Score | Probability-weighted position on ordered levels | Severity, quality ratings |

**Key properties:**
- All questions in one request evaluated independently and in parallel
- Adding more questions barely changes response time
- Probabilities are calibrated (not raw LLM logits)
- Confidence score available for Choice/Score (distribution concentration)
- ~32k token request limit
- Default timeout: 10 seconds

### API Access

**Direct TypeSafe API:**
```
POST https://api.typesafe.ai/v1/systemone
Authorization: Bearer <TYPESAFE_API_KEY>
```

**OpenRouter proxy:**
```
POST https://openrouter.ai/api/alpha/decisions
Model: ~typesafe/jev-latest
```

### TypeSafe Agent Skill

**Official skill repo:** [typesafe-ai/skills](https://github.com/typesafe-ai/skills) — contains `skills/typesafe-ai/SKILL.md`

**Installation:**
```bash
# Claude Code plugin
claude plugin marketplace add typesafe-ai/skills
claude plugin install typesafe@typesafe-ai

# Generic (npx skills)
npx skills add typesafe-ai/skills --skill typesafe-ai
```

The skill provides guidance on when to use each primitive, how to decompose judgments, confidence handling, and pattern selection. It's designed to be read by the agent, not executed as code.

### Context7 vs TypeSafe

**Important distinction:** Context7 ([upstash/context7](https://github.com/upstash/context7)) is a **separate product from Upstash**, not TypeSafe. Context7 pulls up-to-date library documentation into AI coding agents. It has its own MCP server (`@upstash/context7-mcp`) and SDK. It is NOT the "TypeSafe skill platform" — that's the `typesafe-ai/skills` repo on GitHub, which distributes TypeSafe's own agent skill.

### SDKs

- **Python:** `pip install typesafe-sdk` — `AsyncTypeSafeClient`, `system_one(state, questions)` method
- **JavaScript/TypeScript:** `npm install @typesafe/sdk` — similar client pattern
- **Rust:** No official Rust SDK. Would need to call HTTP API directly.

## Jcode Architecture Fit Assessment

### Current Jcode Architecture

Jcode is a Rust application with three crates:
- `jcode-base` — foundation
- `jcode-app-core` — application logic (agents, tools, sessions, compaction)
- `jcode-tui` — terminal UI layer

**Key modules:**
| Module | Path | Current capability |
|--------|------|--------------------|
| Browser tool | `tool/browser.rs` | Firefox bridge with action-based commands; no AI element selection |
| Agent compaction | `agent/compaction.rs` | LLM-summary based compaction with token tracking |
| Agent orchestration | `agent/turn_loops.rs`, `agent/streaming.rs` | Turn-based agent loop |
| MCP server | `tool/mcp.rs` | MCP server management |
| Skill system | `tool/skill.rs` | Skill loading/invocation |
| Computer use | `tool/computer/` | macOS/Windows/Linux screen/input automation |

### Integration Points

1. **Browser tool** (`tool/browser.rs`): Add `action="jev_select"` that takes a goal, snapshots DOM, sends to Jev for element+operation selection, executes result. This would give Jcode "Jev Ultrafast" capabilities.

2. **Compaction** (`agent/compaction.rs`): Add JevCompactor as alternative to LLM compactor. Could be gated behind a config flag initially.

3. **New tool: `evaluate`**: Create a native Jcode tool wrapping TypeSafe API calls. This is the most impactful and lowest-effort integration. Agents could call `evaluate(state, questions)` directly without MCP overhead.

4. **Provider middleware**: Implement jev-router-style model selection as a provider middleware that inserts Jev-based routing decisions before dispatching to the actual model.

5. **Session supervision**: Implement foreman-style periodic assessment as a background task that watches agent sessions.

6. **Review skill enhancement**: Add Jev-driven risk screening to the existing review workflow.

### Recommended Rust Implementation Strategy

Since Jcode is Rust and TypeSafe has no Rust SDK:

1. **Add `reqwest`-based HTTP client** for `POST /v1/systemone`
2. **Define Serde types** for the request/response schema
3. **Create `tool/evaluate.rs`** as a new built-in tool
4. **Optionally bundle `evaluate` (Go binary)** as an MCP server for agent access
5. **Add `JEV_API_KEY` / `TYPESAFE_API_KEY`** config support

The HTTP API is simple enough that a native Rust implementation would be ~200 lines.

```rust
// Proposed tool signature
pub struct EvaluateInput {
    pub state: serde_json::Value,
    pub questions: HashMap<String, Question>,
}

pub enum Question {
    Noul { instructions: String },
    Choice { instructions: String, criteria: HashMap<String, String> },
    Score { instructions: String, criteria: Vec<String> },
}
```

### 9. Mobile Jev — Android Automation Pattern

**Repo:** [droidrun/mobile-jev](https://github.com/droidrun/mobile-jev) · ~103 stars · TypeScript · MIT

**Architecture:** Directly mirrors jev-ultrafast but for Android phones via Mobilerun API (no ADB). Same pattern: one Jev request selects operation + target element from indexed controls and installed apps. Text comes from exact spans in the goal, not generated.

**Key points:**
- 21 seconds for 9 actions (Uber booking demo)
- Live React studio with device stream, action timeline, latency measurements
- CLI for headless runs with traces
- Installed-app inventory (up to 200 apps) for OPEN_APP decisions
- Operations: OPEN_APP, TAP, TYPE_TEXT, scrolling, navigation, WAIT, DONE, BLOCKED
- No ADB required; uses Mobilerun's cloud Android devices

**Jcode relevance:** Low-moderate. Jcode is primarily a desktop/server agent. Mobile use-cases are peripheral. The architecture pattern (indexed controls → Jev selection) is the same as jev-ultrafast. If Jcode ever adds mobile device control, this is the template.

### 10. Jev Voice Browser — Speech-Driven Browser Control

**Repo:** [moritzkremb/jev-voice-browser](https://github.com/moritzkremb/jev-voice-browser) · ~40 stars · TypeScript · MIT

**Architecture:** Chrome Web Speech API → WebSocket → Node server → Jev (9-11 typed questions per partial transcript) → Playwright action. ~250-350ms per Jev call.

**How it works:**
- Microphone streams partial transcripts word-by-word
- 200ms debounce
- Page snapshot (≤100 elements, e01..eNN labels)
- One Jev request with questions: intent, target element, site, "is command complete?", "is this addressed to me?", "is it destructive?"
- Policy code decides: act, wait, ask, or ignore
- Destructive actions require spoken "confirm"
- ~$0.0002 per Jev call

**Jcode relevance:** Low. Voice is not currently in Jcode's scope. But the **architecture pattern** — continuous streaming input → Jev gating → action — is interesting for any real-time agent interaction. The "is command complete?" gating pattern could apply to partial user inputs in Jcode.

### 11. Jev Search — Multi-Source Search with Jev Ranking

**Repo:** [superagents-lab/jev-search](https://github.com/superagents-lab/jev-search) · ~48 stars · TypeScript · MIT (Cloudflare Workers)

**Architecture:** A search pipeline where Jev:
1. Chooses sources, time ranges, and search terms based on natural-language query
2. Ranks results for relevance after multi-engine search

**How it works:**
- Jev interprets user request → query, source selection, time range
- Search across Google, DuckDuckGo, Yandex, Hacker News, Reddit, GitHub, X, arXiv, YouTube, Wikipedia, IMDb, WeChat (concurrent)
- Jev scores each result for relevance
- Results merged by relevance, engine agreement, original rank
- Streamed as each engine completes
- Deployed on Cloudflare Workers; supports TypeSafe, Cloudflare Workers AI, and Vercel AI Gateway for Jev

**Jcode relevance:** Moderate. Jcode's `websearch` tool currently uses DuckDuckGo/Bing without AI-driven ranking. Jev Search shows a pattern for: (1) smart query construction from natural language, (2) relevance ranking of search results, (3) multi-source merging. Jcode could adopt the ranking pattern to improve web search result quality.

### 12. Jevmeter — Video Analysis Pattern

**Repo:** [ChetasLua/jevmeter](https://github.com/ChetasLua/jevmeter) · ~52 stars · Python · MIT

**Architecture:** Scores every sentence in videos using Jev Noul questions, then renders a 16:9 overlay. Full debate costs ~$0.05.

**How it works:**
- Speech-to-text transcription
- Jev scores each sentence against preset dimensions (evasive, emotional appeal, vague guidance, hype, etc.)
- Renders video with meter overlay + scoreboard
- 99% accuracy on held-out preset eval

**Jcode relevance:** Low. Video processing is outside Jcode's scope. The pattern — batch scoring of sequential items with Jev — could apply to any structured review workflow (log analysis, transcript review, etc.).

### 13. Additional Repos (Brief Coverage)

| # | Repo | Stars | What it does | Jcode relevance |
|---|------|-------|-------------|-----------------|
| 2 | typesafe-computer-use | ~203 | Mac-only OCR/accessibility automation | Moderate: Jcode has computer/ module for macOS automation |
| 9 | jev-rules | ~29 | Rule selection for Claude Code | Moderate: could enhance Jcode's skill routing |
| 10 | skillbox | ~190 | MCP-distributed agent skills with Jev recommendations | Moderate: Jcode has a skill system |
| 15 | neo4jev | ~17 | Jev-driven graph traversal in Neo4j | Low: no graph DB in Jcode |
| 18 | youtube-sponsor-detection | ~46 | Auto-skip sponsor segments in YouTube | Low: video processing |

## Contrarian Views And Risks

### 1. Jev is new and unproven at scale

TypeSafe launched Jev publicly mere days ago. Most of these repos are **2-3 days old**. Stars are reflective of hype, not production usage. The API could change, rate limits are unpublished, and the business behind TypeSafe is a startup.

### 2. Latency for real-time browser interaction

While 7.1 seconds for a flight search is impressive, Jev still adds network latency per decision loop. For Jcode's browser tool, which may do 5-20 operations per task, the total Jev latency could be noticeable. The `evaluate` approach from typesafe-mcp batches all questions into one call, which helps.

### 3. Jev may not handle complex reasoning

System One models make fast, shallow judgments. They are NOT designed for multi-step reasoning. The foreman docs explicitly note that Jev assesses, but deterministic code decides. If Jcode needs deep analysis of code diffs (for review) or complex task decomposition, Jev alone won't suffice — you'd need Jev for screening + a reasoning model for depth.

### 4. TypeSafe API dependency

Adding a hard dependency on the TypeSafe API creates a single point of failure. Every Jev integration should have a graceful fallback: the evaluate tool returns errors, the compactor falls back to LLM summarization, the router passes through to the default model.

### 5. OpenRouter vs Direct tradeoffs

OpenRouter provides a fallback path but its Decisions endpoint is on an `/api/alpha/` path and "may move." Direct TypeSafe API is the canonical path but means yet another API key to manage.

### 6. The right primitive for the right job

Not every decision needs Jev. Pattern matching (regex, AST matching), exact lookups, and deterministic calculations should stay in code. Jev is for semantic judgments that would otherwise require LLM parsing.

## Open Questions

1. **Jev rate limits:** TypeSafe's docs mention 429 handling but don't publish numeric limits. What's the sustained QPS limit? How does this affect compaction (bursty) vs review (batch)?

2. **Jev context window vs Jcode session size:** Jev's ~32k token limit may not fit large Jcode sessions. fast-jev-compaction's staged fitting approach would be necessary.

3. **Browser DOM snapshot format:** Does Jcode's Firefox bridge produce structured-enough DOM data to feed Jev? The jev-ultrafast approach uses a custom snapshot.js — Jcode would need equivalent element indexing.

4. **Model tier selection for non-Anthropic/OpenAI:** jev-router is designed for Claude Code and Codex. Jcode uses many more providers. How does tier selection generalize?

5. **Rust HTTP client ecosystem:** Should Jcode use `reqwest` (async, well-maintained) or a lighter client? Should calls be blocking or async in Jcode's architecture?

6. **Confidence calibration for Jcode use cases:** Jev's confidence scores are model-calibrated. Are they calibrated for code review decisions? For browser element selection? Validation needed.

7. **Foreman-style supervision for Jcode swarm:** Jcode's swarm management is more complex than Foreman's single-worker model. Would a supervision layer add value or just noise?

8. **pg-jev for Jcode's data layer:** Jcode uses SQLite. Should it adopt a similar pattern for local judgment storage, or keep judgments ephemeral?

## Sources

| URL | Content |
|-----|---------|
| https://github.com/browser-use/jev-ultrafast | Jev Ultrafast repo (7.9k stars) — browser agent with Jev-driven element selection |
| https://raw.githubusercontent.com/browser-use/jev-ultrafast/main/README.md | Full README: 7.1s flight search, DOM snapshot architecture, speculative fan-out |
| https://github.com/thruwire/foreman | Foreman repo (353 stars) — Jev-supervised Codex worker with 10-dimension assessment |
| https://raw.githubusercontent.com/thruwire/foreman/main/README.md | Full README: dual-loop architecture, assessment dimensions, policy actions |
| https://github.com/gargpratyush/jev-router | jev-router repo (191 stars) — per-turn model routing for Claude Code and Codex |
| https://raw.githubusercontent.com/gargpratyush/jev-router/master/README.md | Full README: loopback proxy, 4-tier routing, policy rules |
| https://github.com/itsmostafa/typesafe-mcp | typesafe-mcp/evaluate repo (99 stars) — MCP server exposing Jev as one tool |
| https://raw.githubusercontent.com/itsmostafa/typesafe-mcp/main/README.md | Full README: one-command setup, three question types, dual API routing |
| https://github.com/tamaratran/fast-jev-compaction | fast-jev-compaction repo (4k stars) — Jev-driven context compaction |
| https://raw.githubusercontent.com/tamaratran/fast-jev-compaction/main/README.md | Full README: 7-step compaction pipeline, tool-call scoring |
| https://github.com/devagrawal09/jev-review | jev-review repo (331 stars) — staged Jev code review pipeline |
| https://raw.githubusercontent.com/devagrawal09/jev-review/main/README.md | Full README: 6-stage review workflow, change/codebase modes |
| https://github.com/mrnugget/jev-shell-history | jev-shell-history repo (63 stars) — zsh history completion via Jev |
| https://github.com/realZachi/pg-jev | pg-jev repo (207 stars) — PostgreSQL Jev extension |
| https://github.com/kitze/unclutter | unclutter repo (~73 stars) — browser extension for Jev-powered ad removal |
| https://github.com/EliaAlberti/jev-rules | jev-rules repo (29 stars) — rule selection for Claude Code via Jev |
| https://raw.githubusercontent.com/droidrun/mobile-jev/main/README.md | mobile-jev README: Android automation via Mobilerun + Jev, same indexed control pattern |
| https://raw.githubusercontent.com/moritzkremb/jev-voice-browser/main/README.md | jev-voice-browser README: Web Speech API → Jev gating → Playwright actions |
| https://raw.githubusercontent.com/superagents-lab/jev-search/main/README.md | jev-search README: Jev-driven query construction + relevance ranking across 12 engines |
| https://raw.githubusercontent.com/ChetasLua/jevmeter/main/README.md | jevmeter README: Jev Noul scoring of video transcripts with overlay rendering |
| https://docs.typesafe.ai/introduction | TypeSafe introduction: Jev, System One, three primitives |
| https://docs.typesafe.ai/api | TypeSafe API reference: POST /v1/systemone, request/response schema |
| https://docs.typesafe.ai/sdk | TypeSafe SDKs: Python and JavaScript |
| https://docs.typesafe.ai/agent-skill | TypeSafe agent skill: installation, usage patterns |
| https://raw.githubusercontent.com/typesafe-ai/skills/main/skills/typesafe-ai/SKILL.md | Official TypeSafe skill definition: patterns, primitives, guidance |
| https://context7.com | Context7 (Upstash) — separate product, not TypeSafe; documentation lookup for agents |
| source/jcode/crates/jcode-app-core/src/tool/browser.rs | Jcode browser tool implementation |
| source/jcode/crates/jcode-app-core/src/agent/compaction.rs | Jcode context compaction module |
| source/jcode/crates/jcode-app-core/src/tool/mcp.rs | Jcode MCP integration module |
| source/jcode/crates/jcode-app-core/src/tool/skill.rs | Jcode skill system implementation |

## Rerun Inputs

```
workflow: firecrawl-deep-research
topic: Jev ecosystem repos 1-17 + TypeSafe Context7 integration for Jcode adoption
depth: exhaustive
output: markdown
```