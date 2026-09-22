# FW-GLM-5.3 tool-definition validation failure

Date: 2026-09-22. Mode: exploration only, no implementation or service changes.

## Intent
Explain the 400 response blocking the user's apply-all run and identify the correct repair boundary. The pasted apply-all invocation is the interrupted workflow, not authorization to execute a proposal queue in this repository.

## Evidence and current state
- Endpoint: local OpenAI-compatible chat completions on port 8317. A listening CLIProxyAPI process exists. This is not an endpoint-unavailable error.
- Runtime error log identifies proxy version 7.3.12-4.
- The proxy error log `error-v1-chat-completions-2026-09-22T122919-d7905da0.log`, accessed through `/proc/2995461/root/CLIProxyAPI/runtime/logs/`, contains both REQUEST BODY and API REQUEST 1.
- Parsed only safe structural metadata from each body. Incoming request: model FW-GLM-5.3, 32 tools, function keys description/name/parameters, zero tools with function.arguments. Upstream API request: same model and 32 tools, function keys arguments/description/name/parameters, all 32 with function.arguments.
- Upstream validation rejects all 32 extra arguments fields. This establishes mutation inside the proxy request path, not Jcode's outgoing tool definitions. Exact mutating helper remains unlocated.
- Jcode source agrees: `crates/jcode-provider-openrouter-runtime/src/openrouter_provider_impl.rs:105-122` constructs definitions without arguments. Lines 244-253 permit extra_body overrides, but observed incoming payload rules out an override as this incident's cause. `openrouter_sse_stream.rs` sends JSON without further tool rewriting.
- Local proxy configuration has no literal arguments rule. This does not rule out more general transforms or deployed patches.
- Local matching-version source snapshot exists under scratch/cliproxy-plus-7.3.12-4. Its exact equivalence to the deployed binary has not been verified. Do not copy or integrate unrelated branches or scratch implementations.

## Prior art and research
Bundled Jcode documentation describes the shared OpenAI-compatible streaming transport and extra_body overrides. No relevant proposal or memory was found with the targeted searches. Session search found no useful prior diagnosis. Local source and runtime boundary logs provide stronger incident-specific evidence than external research. No new library or API is being adopted, so Context7 was not required. No external service or live generation was invoked.

## Adversarial findings
- Never recursively strip arguments: assistant message tool_calls[].function.arguments is legitimate and must be preserved. A user tool schema may also legitimately declare a property named arguments.
- Tool declarations and tool invocations must have distinct normalization paths.
- Retrying, /poke, changing credentials, or correcting the base URL cannot remove fields injected deterministically by the proxy.
- Do not remove tools or set tools=[] as a workaround for an agent executing apply-all.
- Other models accepting extra fields would mask this defect rather than prove the request valid.
- Raw proxy logs contain conversation and potentially credential material. Do not publish or commit them. This record contains structural summaries only.
- Existing unrelated working-tree changes were left untouched. No proxy restart or shared daemon reload was performed.

## Alternatives and recommendation
1. Recommended: trace the deployed proxy's inbound-to-upstream transformation, add a regression reproducing this exact key addition, and ensure definition serialization never synthesizes invocation arguments. Repair the proxy in its own authorized development context.
2. Temporary continuity: use an already configured alternative model, then resend. The UI offers Ctrl+Y. Its success has not been tested and it may share the same proxy.
3. Not recommended: modify Jcode's tool definitions, since the actual incoming payload already conforms on the rejected field.
4. Bypass proxy only if a suitable provider route and credentials already exist and the user accepts routing changes. Do not provision or expose credentials for this investigation.

## Proposed scope and verification
- Establish deployed binary provenance and the specific mutating helper before implementation.
- Regression fixtures: one tool definition, 32 definitions, empty tools, prior assistant tool calls with real JSON-string arguments, and a schema property named arguments.
- Assert upstream tools[].function contains no synthesized arguments and legitimate invocation arguments remain unchanged.
- Exercise the real compatible executor against a strict loopback upstream validator.
- Finally run a small streaming tool-call round trip through the deployed proxy to FW-GLM-5.3 and verify a tool result continuation succeeds. A compile or unit-only test is not sufficient acceptance evidence.

## User actions and open decisions
Implementation and deployment are outside active explore scope. A shared proxy restart can disrupt other sessions and needs coordination. No credentials or user-only actions are required to finish this diagnosis. After an approved repair is deployed and verified, resume the original session using /poke or resend the interrupted command. Route next to a bounded proxy-fix feature, not a Jcode serialization change.

## Follow-up: cross-route investigation at 18:05 UTC

### Provenance and exact root cause
The deployed executable reports CLIProxyAPIPlus 7.3.12-4, commit 5cceedbef, built 2026-09-22T07:00:53Z. Go build metadata agrees. SHA-256: `7e8bac067da4eb9d9b184553cc7922e6b646f6077b2635a130e441f5ca6c6836`.

Version-matched source in the existing scratch snapshot identifies the defect in `internal/runtime/executor/openai_compat_executor.go`, `sanitizeEmptyToolDeclarations`, lines 1679-1688. It backfills `function.arguments` on tool declarations. This supersedes the earlier unresolved-helper note. `sanitizeEmptyToolFunctionNames` calls it unconditionally from both Execute (line 185) and ExecuteStream (line 603). The existing `openai_compat_executor_empty_tool_names_test.go` explicitly expects the incorrect behavior for missing, empty, and whitespace arguments. This is a semantic confusion between declarations and invocations, not a GLM-specific feature.

### Runtime experiment and coverage
Launched a separate instance of the exact deployed executable with isolated HOME, auth directory, ephemeral loopback listener, dummy API keys, and a loopback capture upstream. No production config, service, or credentials were modified. The capture upstream intentionally returns HTTP 400 after recording request structure. These experiments prove emitted payload shape, not successful remote model responses.

Probe: `scratch/proxy_scope_probe.py`. Results: `scratch/proxy-scope-probe/results.json`.

32 cases passed their reproduction assertions: four model names (FW-GLM-5.3, DeepSeek-V4-Pro, FW-Kimi-K3, generic gpt-probe), two streaming settings, and four fixtures (ordinary definition, prior assistant invocation, empty tools, schema property named arguments). All 24 nonempty-catalog cases gained the erroneous arguments field. Empty catalogs gained no tool. Legitimate invocation arguments and the nested schema property were preserved.

A second 32-case experiment (`scratch/proxy_scope_filter_probe.py`, results in `scratch/proxy-scope-probe-filter/results.json`) added `payload.filter` for `tools.*.function.arguments` on all OpenAI models. All 24 nonempty cases still gained arguments. Source ordering explains why: payload config is applied before the unconditional sanitizer. A normal payload filter cannot solve this bug.

### Impact boundaries
- The active azure-foundry OpenAI-compatible provider lists FW-GLM-5.3, DeepSeek-V4-Pro, and FW-Kimi-K3. All three model names are confirmed to receive the mutation using the real executable and equivalent compatible routing against the local capture upstream.
- Whether the remote DeepSeek and Kimi deployments reject or tolerate it remains untested. Only GLM rejection is proven by production evidence.
- A generic gpt-probe name on the compatible route also mutates. Model-family naming does not protect this path.
- The local auth inventory contains two native Codex credentials. Source `sdk/cliproxy/service_executors.go:289-300` registers native Codex via CodexAutoExecutor and returns before compatible registration. The defective helper has only the two compatible-executor call sites. Native Codex is therefore outside this specific defect's source path. No live native Codex round trip was performed, and this is not a blanket health claim for every OpenAI model.
- Other providers/routes were not exhaustively smoke-tested. Distinguish the Jcode profile label from the proxy's actual executor routing.

### External evidence
GitHub's latest-release endpoint, fetched 2026-09-22 around 18:04 UTC, reports v7.3.12-4 as the latest published release, the same affected version. Therefore “upgrade to latest” is not currently a verified remedy. Public release: https://github.com/jc01rho/CLIProxyAPIPlus/releases/tag/v7.3.12-4 . No new dependency or API adoption is proposed. Context7 was not needed for this source-level bug investigation.

### Recommended update, not configuration
Recommend a minimal patch to the same deployed proxy release: remove argument backfilling from tool declaration sanitation, retain empty-name handling, and preserve actual assistant invocation arguments. Update the tests that currently enshrine the invalid definition shape. Do not implement a GLM-only exception, change Jcode, disable tools, or recursively delete all arguments fields. Whether explicitly client-supplied invalid declaration arguments should be stripped is a separate compatibility decision; stopping synthesis is the minimal incident fix.

Alternatives: payload-filter workaround is experimentally disproven; model switching can mask strict validation without fixing the corruption; a downgrade requires its own route/regression matrix and may lose unrelated fixes; a post-proxy rewriting shim adds another service and is inferior to correcting the responsible function. Latest published release is not a fix. No upstream patch was integrated or copied from another branch.

### Deployment gates and remaining decisions
1. Author a bounded proxy repair in its authorized repository/branch. No production code was changed during this exploration.
2. Run strict shape regressions for both executor modes and all three configured compatible model aliases. Include 32-tool catalogs, named required tool choice, empty/invalid names, existing invocation arguments, malformed arguments, and nested properties named arguments. Verify legitimate tools survive.
3. Run complete synthetic streaming tool-call/result continuation against the candidate binary, plus native Codex regression coverage. The current capture tests intentionally stop at upstream 400 and do not establish response-stream correctness.
4. Canary the candidate on a separate listener with a small real request/tool continuation for each Azure model and native Codex before changing the shared listener. This requires existing authorized provider credentials but no new provisioning.
5. Pin the candidate artifact and checksum. Preserve the current binary/image and config for rollback. Coordinate the shared-service restart with active sessions. Check deployment packaging and successful model listing, then smoke-test the production listener before resuming /poke.

Outcome: cross-model request mutation and the failure of the config-only workaround are established by 64 isolated runtime cases. A narrow proxy patch is the recommended path. Implementation, successful upstream model round trips, candidate packaging, and deployment remain intentionally unperformed under explore scope.
