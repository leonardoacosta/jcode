## Context

### Current correction, 2026-11-03

Compression ownership is now the gateway combo/model route, not Jcode session roles. Remove the compatible provider's profile state, role setter implementation, retry-builder selector/cache-bypass headers, and receipt notice. Remove agent turn-entry selection/notices. Retain the shared provider trait for compatibility with other callers and retain review prompt semantics. Legacy named-provider compression headers no longer opt into automatic selection. The transport forwards the selected model unchanged on both protocols.

Read-only local runtime inspection of `/app/.build/next/server/chunks/16904.js` confirms the resolver prioritizes request overrides before routing overrides, active profile, auto-trigger, and default. The Jcode correction removes its first-priority override. Native SQLite readback found the saved conservative/optimized profiles but no rows in `compression_combo_assignments`. This is not proof that each combo has its intended profile. No live settings were changed and no per-combo assignment or applied-plan success is claimed.

Preflight `preflight-check-v2-shadow`, JIT at source `1d9b92579` on the existing `main` checkout: local source/test correction is eligible under the user's explicit correction. Related tracked targets started clean; preserve the five unrelated untracked runtime paths. Publication is not authorized and branch mismatch remains preserved. Existing broader gateway acceptance/archive prerequisites remain blocked. Tests use real provider and agent public paths with local synthetic transports. The prior notes below describe historical role-header selection, not the corrected implementation.

User approval covers role-based profiles on the two existing gateways plus real TypeSafe Jev verification. Existing config uses Pi5 as the default, manual cross-provider failover, and inherited memory models. Request model aliases already exercise gateway routing combos. The missing contract is role-aware compression selection and observed resolver enforcement, not replacement upstream routing.

## Decisions

- Reuse OmniRoute's supported `x-omniroute-compression` request selector. Equivalent gateway profile names are `jcode-conservative` and `jcode-optimized`; exact requests use `off`. Do not use process-global environment changes to select per-session policy.
- Keep policy state isolated per provider fork or pass a captured per-request value through background transport tasks. A task-local value alone is insufficient if request construction runs in a spawned task.
- Limit gateway headers to explicitly configured supported gateway routes. Generic OpenAI-compatible endpoints are not evidence of OmniRoute support.
- Root and unknown roles are conservative. Use durable bounded-coding identity for the optimized trial, never model-name or prompt inference. Memory always overrides to conservative on an isolated request/fork. Final review and Jev decisions are exact.
- Fix or prove the gateway's resolver behavior before treating a combo card's stored mode as effective. Named profile assignments and current runtime receipts are distinct evidence.
- Keep output-style selection separate from input compression. Exact final review must not inherit forced ultra-terse output constraints.
- Do not change provider accounts, credentials, priorities, quotas, permissions, deployment mechanisms, or automatic fallback policy.

## Runtime bindings and rollback

Observed 2026-11-02 19:49 UTC through authenticated native APIs. Local container `omniroute` runs package 3.8.51, image `homelab/omniroute-routing-observability:3.8.51-20261027`. Pi5 container `omniroute-pi-upstream` runs package 3.8.52, image `homelab/omniroute-global-features:20261029-r5`. No image upgrade, daemon restart, provider-account change, or database migration is needed to create native compression profiles.

Before state (compression fields only, no credentials):

- Local: `enabled=true`, `defaultMode=off`, `activeComboId=default-caveman`; API-default stacked pipeline RTK standard then Caveman full. The active Standard Savings profile is session-dedup then lite.
- Pi5: `enabled=true`, `defaultMode=stacked`, `activeComboId=null`, stacked pipeline RTK aggressive then Caveman ultra.
- Both had only profile `default-caveman`; no named-profile assignments and no effective combo overrides.

The approved `jcode-conservative` and `jcode-optimized` native profiles were created and read back. Both gateways now have `enabled=true`, `activeComboId=jcode-conservative`, `defaultMode=stacked`, and a session-dedup then lite fallback pipeline. Only those approved fallback fields changed. Output styles, engine flags, credentials, accounts, quotas, and routing priorities were preserved. Rollback uses authenticated `PUT /api/settings/compression` to restore the corresponding before values; newly created unused profiles need not be deleted. Gateway source inspection confirms explicit `x-omniroute-compression: off` bypasses output-style injection as well as input compression.

A real non-inference `/api/compression/preview` check found that local 3.8.51 Caveman ultra changed unfenced synthetic JavaScript (72 estimated tokens saved), while Pi5 preserved it. The RTK stage did not change that fixture. Native Caveman `preservePatterns` fixed the discrepancy without replacing either gateway image. Both optimized profiles now use:

```json
[
  {"engine":"rtk","intensity":"aggressive","config":{"applyToToolResults":false,"applyToCodeBlocks":false}},
  {"engine":"caveman","intensity":"ultra","config":{"preservePatterns":["[\\s\\S]*[{}\\[\\]\\x60][\\s\\S]*","[\\s\\S]*(?:^|\\n)[ \\t]*[A-Za-z_][A-Za-z0-9_.-]*[ \\t]*[:=][\\s\\S]*"]}}
]
```

Profiles were updated through authenticated `PUT /api/context/combos/jcode-optimized` with strict before-state drift checks, then read back. Both runtime versions now preserve the four synthetic fixtures byte-for-byte: system prose, fenced code, JSON tool output, and unfenced code. These are representative engine checks, not a claim that every possible input or complete inference request is covered. Actual role-selected gateway receipts remain a separate acceptance task.

## Verification and rollout

Use actual project transport request capture for both Chat Completions and Responses, negative direct-endpoint tests, fork/concurrency/model-switch tests, role classification tests, and memory/review override tests. Verify the maintained version-checked gateway patch against its actual base image, with source-specific tests and unchanged protected-content behavior. Follow Homelab and Pi5 repository deployment rules, verify exact targets and prior state for rollback, and inspect sanitized runtime receipts using synthetic inputs. No transcript or credential capture is needed.

TypeSafe verification must use production `JevClient` and explicit provider selection. A mock proves protocol handling only; it does not satisfy live acceptance. Record missing credential/auth/quota/schema failures without fallback.

After fresh final-diff checks, run independent review, repair and repeat affected checks/review, update tasks, commit scoped verified changes using the configured identity, and archive only after required gates pass. Preserve the existing `main` checkout despite remote HEAD naming `dev`. Do not publish unrelated history or bypass repository delivery gates.

## Verification snapshot

Observed 2026-11-02 20:28 UTC. Explicit opt-in belongs only on supported named gateway tables: `headers = { x-omniroute-compression = "jcode-conservative" }`. The presence of this marker enables role selection. Other named/direct providers remain unsupported and receive no policy header. The local config currently enables only `providers.omnipi`; remove that single entry to roll back the opt-in. All other parsed config fields, including credentials, model choices, protocols, and priorities, were verified unchanged.

Fresh commands and results:

- `scripts/dev_cargo.sh test -p jcode-provider-openrouter-runtime --lib`: 200 passed, 2 ignored. Coverage includes both wire protocols, isolated forks, captured request policy, missing/mismatched/legacy response receipts, and direct endpoints.
- `scripts/dev_cargo.sh test -p jcode-base --lib provider::tests -- --test-threads=1`: 181 passed, including actual registered-runtime fork/rebinding paths.
- `scripts/dev_cargo.sh test -p jcode-base --lib input_compression -- --test-threads=1`: 3 passed for durable roles, caller-isolated memory override, and provider rebinding.
- `scripts/dev_cargo.sh test -p jcode-base --lib jev:: -- --test-threads=1`: 61 passed, 2 ignored.
- `scripts/dev_cargo.sh test -p jcode-app-core --lib gateway_input_policy_tracks_durable_roles_and_reports_direct_routes -- --test-threads=1`: 1 passed.
- `openspec validate enforce-role-based-compression-routing --strict`: passed.

The ignored live gateway test requires `JCODE_LIVE_GATEWAY_POLICY_TEST=1`; `JCODE_LIVE_GATEWAY_ROUTE` scopes one existing gateway. `JCODE_LIVE_GATEWAY_MODEL` is allowed only with one explicit target, and `JCODE_LIVE_GATEWAY_PROFILE` can independently select one approved policy. The test uses the native, source-verified `x-omniroute-no-cache: true` control. Never equate a cached or HTTP-200 response with compression enforcement.

Pi5's configured Chat endpoint completed all three policies with matching request-header receipts. The configured local endpoint is Responses. Its default bare combo was rejected by the current upstream account, so diagnostics explicitly selected the already-known-working `codex/gpt-6.1-sol`; no stored model, combo, account, quota, or priority was changed. Qualified local conservative/optimized/exact completions return the expected response but no compression response header. A cache-bypassed repetition has the same limitation. Read-only native SQLite analytics, restricted to small requests in the synthetic time window, show conservative 736->736 and optimized 732->619 tokens; exact creates no compression row. All six native API-key policies have compression enabled. These observations rule out a blanket Responses-compression bypass, but do not provide complete exact/profile attestation. Retain the failed public receipt gate and disabled local opt-in until the integration is verified. Do not fabricate a success receipt by reflecting a request header.

`JCODE_MEMORY_JEV_PROVIDER=typesafe scripts/dev_cargo.sh test -p jcode-base --lib live_typesafe_jev_returns_expected_choice -- --ignored --nocapture --test-threads=1` uses production `JevClient`, enforces the TypeSafe endpoint, and expects schema-valid `green`. It is blocked before HTTP by the missing TypeSafe credential. Other provider keys are not substitutes. No live TypeSafe success, archive, remote publication, or gateway image deployment is claimed.

## Expanded fidelity gate and pending source delivery

The earlier four small fixtures did not cover truncation thresholds or whitespace-sensitive literals. Larger native previews found that Lite truncated a JSON tool payload from 27,712 to 517 estimated tokens on both versions. Native source confirms a default 2,000-character tool limit. Per-stage `config.compressToolResults` is rejected by the actual API schema, so it was not applied. The supported global setting `lite.compressToolResults=false` was instead applied through `PUT /api/settings/compression` and read back on both targets, preserving the approved pipeline order and every other Lite field. Its prior value was `true`; restore only that field to roll back this repair.

After that repair, conservative previews preserve large JSON and fenced tool output, but both Lite versions still strip trailing spaces and collapse blank lines inside a fenced Python string literal. Local optimized RTK also changes large structured tool results; the existing Pi5 guarded RTK preserves them. These remain failed acceptance cases, not successful optimization.

The maintained Homelab patch now has a protected-content Lite guard in `services/omniroute/patch_global_compression.cjs`; `Dockerfile.global-features` applies and checks it. It fails closed for tool payloads and identifiable code/data/diffs/traces before any Lite transform, while retaining safe-prose normalization. The `--test-lite <native-chunk>` check executes the actual captured module, with only vision/token-stat dependencies stubbed. Unmodified native modules from both runtime versions failed the whitespace fixture; patched scratch copies passed protected payloads and ordinary prose. Patch idempotence, syntax, unknown/duplicate anchors, and corrupt markers were checked. This is representative source-level evidence, not deployed end-to-end acceptance. No running container or gateway image was changed. The Dockerfile's required 3.8.52 base image is unavailable locally, and the current local gateway remains 3.8.51. Do not bypass version checks, silently upgrade it, or claim a successful image build.

Operational eligibility: local implementation, read-only investigation, source checks, and independent review remain eligible. Full TypeSafe acceptance is blocked by its own missing credential. Final gateway fidelity and per-request local attribution remain unverified. Gateway image delivery requires a separately authorized, version-checked owner-repository rollout. Scoped local checkpoint commits may follow review; archive and full completion remain blocked.

Recovery clarification: the original approved recommendation explicitly says conservative is not strictly lossless. Do not convert conservative whitespace normalization into an unapproved universal byte-fidelity requirement. The pending Lite guard hardens that risk; optimized-trial protected-content failures remain acceptance defects. Pi5 does have the maintained arm64 base image. The actual owner runner, however, fails its no-write preflight because `/opt/pi5-infra` is absent and initial activation is a separate change. Its code also refuses recreation/adoption. This concrete owner-path blocker, not mere local image absence, prevents a compliant live replacement. No direct Docker restart or owner-runner bypass is permitted.

## Native repair supersedes optional image hardening

Observed 2026-11-02 21:11 UTC: native RTK stage configuration supports `applyToToolResults=false` and `applyToCodeBlocks=false`. Real local previews with those settings preserve the previously failing large JSON and fenced tool payloads. The same settings were applied to both saved `jcode-optimized` profiles through the native combo API after strict before-state checks, then read back. RTK remains aggressive and Caveman remains ultra with the existing preserve patterns. No provider route, account, quota, priority, image, or process changed. Rollback removes only those two RTK-stage options, retaining the previously established Caveman patterns.

Final real previews passed seven optimized protected-content fixtures on each running gateway, including large JSON, large fenced tool code, and whitespace-sensitive literals. Conservative large tool truncation is disabled on both through the supported global Lite setting. Conservative's documented whitespace normalization is observed and reported, not relabeled byte-preserving. This matches the originally approved non-lossless conservative policy and mandatory optimized protected-content rules.

Consequently, no gateway image rollout is necessary for the required repair. This session's two optional Homelab Lite hardening edits were removed with an exact reverse-diff check, leaving all unrelated changes untouched; the optional patch remains scratch-only. Earlier source-test/deployment paragraphs are historical investigation, not current pending delivery. Pi5 owner activation is outside the selected completion path. Remaining blockers are missing direct TypeSafe credentials, local Responses receipt attribution, and successful independent final review.

Independent-review correction: arbitrary named-provider headers are not forwarded by this transport. Earlier live-fixture insertion of `x-omniroute-no-cache` did not prove a cache bypass. The final production builder now sends this exact native control only when an explicit compression profile is active, next to that captured selector and inside every retry. This prevents a cached result from another policy being mistaken for exact processing without expanding generic header forwarding. The both-protocol capture assertion failed before the fix and passed afterward in the complete provider suite. Direct/unopted routes still receive neither policy nor cache-control header. Live inputs also carry unique synthetic nonces. Fresh post-fix Pi5 completions pass all three profiles; local Responses still returns expected output without the required receipt. Preserve that failed attribution gate rather than reflecting request headers as proof.
