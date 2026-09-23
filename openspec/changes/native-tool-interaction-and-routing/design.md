# Design: native tool interaction and routing

## Evidence and current surfaces

- `crates/jcode-tool-core/src/lib.rs`: async Tool::execute already permits awaiting; ToolContext has only a string-based stdin request channel.
- `crates/jcode-app-core/src/tool/mod.rs`: registry, conditional macOS registration, ambient/selfdev gates, session allow/deny policy.
- `crates/jcode-protocol/src/wire.rs`: stdin request/reply only, not question payloads.
- `crates/jcode-app-core/src/server/client_lifecycle.rs`: stdin forwarder is connection-local and emits an empty tool_call_id. Do not reuse it as question lifecycle ownership.
- `crates/jcode-tui/src/tui/app/remote/server_events.rs`: stdin events only show a timeout notice.
- `crates/jcode-app-core/src/tool/webfetch.rs`: direct reqwest GET, 5 MiB body cap, byte-based approximately 40 KB output cap, regex conversion, 30s default/120s max timeout.
- `crates/jcode-base/src/prompt.rs`: project/global system-prompt.md replaces bundled default. Current global override exists. `crates/jcode-base/src/prompt/system_prompt.md` is not the sole effective prompt.
- `crates/jcode-app-core/src/tool/browser.rs`: jev_select dispatch exists but action enum omits it. Current helper drops tab/window/frame scope, passes an index as selector and does not carry text values. Archived browser proposal is intent, not proof of implemented acceptance.
- `crates/jcode-app-core/src/tool/computer/mod.rs`: macos_computer_use offers discover, AX/background actions, permission checks, point coordinates and dry_run.
- `crates/jcode-app-core/src/tool/evaluate.rs`: typed probabilistic judgments, not human consent.

Live recon: native fetch and Firecrawl CLI 1.23.3 both read example.com and docs. Native JS-page output lacked quotes, Firecrawl returned ten. Native 404 raised an error; Firecrawl CLI returned success with page status 404. A max-age=0 follow-up returned quotes but omitted cacheState. Cache disposition was not independently established. Sources: https://code.claude.com/docs/en/agent-sdk/user-input and https://docs.firecrawl.dev/features/scrape . Local evidence: `.firecrawl/webfetch-recon/observations.md` (not required for implementation).

## Question contract

Canonical `ask_user_question`, not a Claude SDK dependency. Input: questions (1-4), each with stable id, question text, header (<=12 characters), options (2-4, stable id/label/description), multi_select default false. Enforce unique question ids and per-question option ids, bounded strings and total payload. Proposed maxima: question 1000 chars, label 100, description 500, answer 4000, payload 64 KiB. User-provided answers cannot be set by the model input.

Result: status answered|cancelled|unavailable plus answers keyed by question id, each with selected option ids and optional free_text. Include displayed labels/text for transcript readability; never infer consent from defaults. Other is a UI affordance, not a model-supplied option. Submission is atomic across questions, requires an answer to each, and validates single/multi-select cardinality server-side.

Server owns pending state keyed by session and tool-call id with unpredictable request id. Typed request/resolution events and response request carry correlation. Response handlers must not await the agent mutex held by a suspended turn. One active question per session; concurrent/batched requests return busy/unavailable rather than overwrite. Reattachment to the same live session replays the pending question. First valid answer wins, duplicates receive already-resolved acknowledgement; wrong session is rejected. A disconnect does not fabricate cancellation. Explicit Esc cancellation resolves the tool, while turn cancel/session clear/server shutdown releases pending waits. Process restart does not resurrect an answer channel; interruption is recorded, and the agent may ask anew. No secret/password collection.

Capability negotiation gates exposure: interactive attached root with question-capable client only. Unsupported CLI/direct calls and workers return unavailable immediately. Tool-call/output history remains balanced in direct API and SDK-native paths. Waiting is visible, not billed as model generation or mislabeled as hung shell execution.

## Fetch contract and transport

Preserve existing url/format/timeout. Add backend direct|firecrawl (default direct), optional freshness bound max_age_ms for Firecrawl only. Do not implement auto fallback. This initial Firecrawl path accepts markdown only; reject html/text explicitly rather than silently returning cleaned/raw content with different semantics. Direct preserves current formats and behavior.

Proposed initial transport: optional installed Firecrawl CLI, spawned directly with literal argv, no shell; no bundled Node requirement, install/login or private CLI-config parsing. Probe supported flags/version once and fail clearly when unavailable. Credentials remain in its supported inherited environment/private store and never argv. Native Jcode owns validation, timeout, bounded output and normalized results. Unsupported future CLI schemas fail closed. A different transport requires a design amendment, not a silent REST fallback.

Hosted requests require explicit config enablement and a normalized exact-host allowlist. Approval authorizes this configuration mechanism, not individual private destinations. Require HTTPS, no URL credentials or secret-bearing links, public DNS only, all address answers checked; reject local/IP literals and mixed public/private answers. Remote public redirects rely on the documented public-research trust boundary; validate exposed final URL, and reject outside scope before returning content. No claim of per-hop SSRF enforcement; sensitive/customer/internal use is not supported. A missing allowlist, auth, CLI, unsupported flag, DNS failure or service error causes a bounded actionable error without local fallback or retries that multiply spend.

Use direct-process streaming with 5 MiB response cap, child cleanup on cancel/timeout, current output cap and truncation metadata. Return backend, original/final URL, target status, cache status (unknown if omitted), requested freshness, and reported credits without invented values. Treat API success and target status separately. Reject non-2xx target responses except documented valid 304 cached content. Mark retrieved text as untrusted data. Do not add hosted headers/profiles/actions or private-file parsing.

## Prompt/tool inventory

Audit registry plus runtime policy, not memory of tool names. Generate/filter routing guidance from effective exposed capabilities; do not advertise platform-disabled or allowlist-hidden tools. Tool descriptions carry parameter details, prompt carries task routing. Families:

- Files/code: agentgrep, read, ls, edit, multiedit, apply_patch, patch, write, bash.
- Interaction/web: ask_user_question, webfetch, websearch, browser, macos_computer_use, open.
- Orchestration: batch, bg, swarm, todo, initiative, schedule.
- Context/discovery: jcode_docs, skill_manage, memory, session_search, conversation_search, integration_tools, mcp and dynamically registered MCP tools.
- Specialized: evaluate, gmail, side_panel, maintainer_feedback, invalid.
- Conditional: selfdev, debug_socket; ambient end_ambient_cycle, schedule_ambient, request_permission, send_message.

Inventory records platform/feature gates and coverage reason for every registry entry. No need to repeat every dynamic MCP name in the prompt. Core routing: ask only genuinely blocking human choices; evaluate is machine judgment and cannot answer for the human. Fetch for reading, browser for scoped live-page interaction, computer tool for user-directed native desktop tasks. Browser status before setup; honor actual supported provider rather than enum alone. Prefer background AX over visible cursor/focus changes; missing permissions remain a user action. Respect explicit skill/policy requirements, destructive-action consent and secret handling.

Compose a deterministic tool-routing module after the selected base prompt and before user overlays, passing capability names down rather than importing registry into base crate. Preserve overrides/overlay order and caching semantics. Do not edit global system-prompt.md or skill policies automatically. Test conflicting custom guidance: module describes capabilities, never overrides stricter safety or explicit user instructions.

## Browser parity boundary

Advertising jev_select requires schema/dispatch parity AND scoped, validated execution. Preserve tab/window/frame through observation and action; resolve a selected element only against the same scoped snapshot. Unknown/stale targets, missing required values, unsupported operations or no credentials return actionable failure without mutation. No generated text values, no autonomous retries, no full browser agent expansion. If the existing bridge cannot meet these requirements, block this task and dependent prompt advertisement rather than implement the entire archived proposal without approval. Hosted Jev page-text egress and mutating behavior must be explicit in description. Computer behavior is documented/tested, not expanded.

## Verification and rollout

Add protocol/validation/lifecycle tests and real TUI tester frames for selections, cancellation and live reconnect. Test old client capability negotiation and unsupported headless callers. Live Firecrawl smoke is opt-in with bounded credits/public allowlist; no keys in fixtures. Run isolated built daemon, not shared binary. Verify macOS computer guidance and harmless observe/permission checks on macOS before claiming that branch validated. Default-direct offline installation works without Firecrawl/Node. No shared daemon replacement without separate rollout approval.
