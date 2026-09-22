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
