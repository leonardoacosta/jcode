# OO → TC pilot rollout

Status: **implementation in progress; acceptance not proven**, 2026-10-02. Scoped OO/TC changes and offline checks are complete as recorded below. TesterArmy OAuth and TC project creation succeeded. TC hosted static run `37043479059` reached the production dependency audit and failed that gate. No live Neon lifecycle, hosted required-story acceptance, native Linear defect export, legacy retirement, or deployment has been completed.

Artifact validation: skill frontmatter valid; four synthetic decision regressions passed all 16 assertions; local reference links and OO/TC inventory source paths verified; new skill passed scoped privacy boundary patterns and is discoverable as `/story-driven-ci`. Framework contract review corrected obsolete config/reporting guidance. The full skill-store public-release boundary gate failed on pre-existing unrelated skill content, not this new skill. Hosted static CI evidence is recorded below; the full CI gate, DB lifecycle, and native Linear export remain unvalidated. Main policy: [story-driven-ci](/home/nyaptor/.agents/skills/story-driven-ci/SKILL.md). Contracts: [framework-contracts](/home/nyaptor/.agents/skills/story-driven-ci/references/framework-contracts.md).

## Target architecture

```text
candidate SHA + trust decision
  → cheap existing checks
  → owned attempt-scoped Neon schema-only branch + synthetic seed
  → app built/started once on approved hosted acceptance runner
  → one Chromium e2e execution: narrow agent goals + exact business assertions
  → JSON/JUnit + essential diagnostics + story ledger + resource cleanup
  → stable admission result on the exact candidate SHA

confirmed product defect in hosted TesterArmy Issues
  → authorized native Linear export → existing linked ticket on re-export
```

The second route is not an established automatic consumer of CLI report.json. Keep the CLI-to-platform bridge explicitly unresolved. Do not make report upload or ticket export part of the critical pass/fail path. Keep secrets out of untrusted code and preserve deploy admission.

## Current state and gaps

### Otaku Odyssey

Repository `/home/nyaptor/dev/priceless/otaku-odyssey`, `dev`, inspected HEAD `6d0cecaab29b26239e017be9f690b7009ed115b2`, ahead of `origin/dev` by two commits. Pre-existing modified `.env.test`, `.neon`, and `openspec/changes/add-agentic-e2e-lane/tasks.md`; untracked `.firecrawl/`, `neon.ts`. Preserve these and do not stage secret-bearing files.

- `.github/workflows/ci.yml` supports PRs to `main/dev`, pushes to `main`, and manual scopes. Docs paths are ignored. A manual signed-lane acceptance job does not provision a DB/start the app/run browser acceptance. Existing required status names and release/production admission need inspection before redesign, not assumptions from stale specs.
- `packages/e2e/e2e.config.ts` already imports `e2e` and `@e2e-dev/web`, selects `tests/agent/**/*.e2e.ts`, and uses the configured OmniRoute model path with `OMNI_API_KEY`. `packages/e2e/scripts/run-e2e.mjs` admits only loopback targets and requires prepared-target admission in CI. Treat this as an existing boundary, not a need for arbitrary remote browser origins.
- Pinned source package versions at inspection: `e2e` `0.15.0-canary-20260928184528`, `@e2e-dev/web` `0.11.0-canary-20260928184528`, Playwright packages `1.63.0`. New docs may outpace these types. Verify installed support before updating config.
- `apps/scripts/src/preview-refresh/neon.ts` has `provisionNeonCandidate` and `deleteNeonCandidate`; preparation in `actions/prepare.ts` retains successful preview candidates, and `actions/cleanup.ts` is not a turnkey PR-attempt teardown. Reuse validated identity/ready/URL logic, not preview retention semantics. Unit coverage lives in `preview-refresh/__tests__/neon.test.ts`.
- Native change `openspec/changes/add-agentic-e2e-lane/` still requires staged legacy parity and an owned prepared-target architecture. Tasks have two prior completion marks and a pre-existing duplicate `3`. These are history, not proof of new acceptance.
- **External blocker:** exposed Neon child credential scope/invalidation is unresolved. No branch, database, or credential operation before authoritative account-specific resolution. Generic role-inheritance docs are insufficient.

### Tribal Cities

Repository `/home/nyaptor/dev/priceless/tribal-cities`, `dev`, inspected HEAD `d8b5ec480b396c777071ce563c6f7c6e4d24b606`. Pre-existing `.env.test` modification. Preserve it without reading values.

- `.github/workflows/ci.yml` is `workflow_dispatch` only. Jobs include lint, format, typecheck, unit tests, disposable PostgreSQL integration, security, build, and fallow. Unit job excludes `@tc/e2e`; there is no active E2E job despite comments about a PR-only lane.
- `packages/e2e/e2e.config.ts`, `scripts/run-e2e.mjs`, and `src/agent/auth-login.e2e.ts` are the current tester-style foothold. Package scripts still invoke the legacy `scripts/run-playwright.mjs`; there are legacy config/projects and lint gates. Do not call the current workflow a working e2e standard.
- `openspec/changes/add-agentic-e2e-lane/` still frames replacement as cohorts/parity/second lane. Reconcile with the new single bounded cutover.
- No disposable Neon branch lifecycle was found in the inspected workflow/e2e/db/scripts/change paths. Existing disposable PostgreSQL integration is a useful cheap correctness layer, not proof of Neon branch acceptance and not automatically redundant.
- Both pilots declare `e2e` `0.15.0-canary-20260928184528` and `@e2e-dev/web` `0.11.0-canary-20260928184528`. TC's lockfile records a `playwright >=1.63.0 <2` engine peer. Preserve that browser-engine dependency even when deleting `@playwright/test` architecture. TC's existing agent auth checks staff session email/role, not the participant auth stories below. Do not copy OO's provider/seed/deployment topology by assumption.

Native reconciliation completed: OO and TC proposals/designs/specs/tasks now express the pivot with concrete future implementation/acceptance sequences. Both preserve prior task documents byte-for-byte in `references/legacy-task-history.md`; current acceptance tasks remain unchecked. Strict OpenSpec JSON validation has `valid: true`, no issues, for both changes. Authored Markdown formatting, unique task/delta/link integrity and unchanged environment/Neon diff checks pass; independent planning review found no remaining actionable defect. OO commit `c1917a5d22`, TC commit `bf16c9d7d`, each on local `feat/story-driven-ci-plan`. These are planning-only changes, not runtime implementation, hosted acceptance, deployment or archive.

## Ordered execution plan

Complete the skill and reference validation first, then use it for repository-native planning. Task labels below are unique planning IDs, not claims of completion in either OpenSpec change.

### Phase A: OO plan reconciliation, safe while Neon is blocked

1. **OO-A1: Rewrite native change coherently.** Update `proposal.md`, `design.md`, `specs/ci-cd/spec.md`, `specs/e2e-testing/spec.md`, and `tasks.md` to story-led acceptance, Neon attempt isolation, trust, evidence, and deletion criteria. Inspect/preserve the existing task diff first; record dispositions for prior tasks rather than overwriting their history. Remove duplicate IDs and superseded parity requirements. Preserve release admission and repository verification routing.
2. **OO-A2: Establish story ledger and deletion manifest.** Use the initial inventory below. Identify actual stable required checks, supported dispatch inputs, candidate revision binding, fixture dependencies, and which legacy scripts/config/docs/dependencies become obsolete. Mark unverified terminal payment outcomes as discovery, not requirements already proven.
3. **OO-A3: Verify local contracts without secrets.** Inspect pinned package types/CLI help and existing Neon unit helpers. Use offline/unit tests for target allowlist, branch ownership, partial provisioning, stale-cache behavior, artifact retention, empty selection, and cleanup failure. Do not invoke wrappers that decrypt `.env.test`, pretend to be CI, or touch shared DBs.

Exit A: coherent native artifacts, classified critical stories and legacy deletion map, local guard tests where safe, unresolved external blockers listed separately. No hosted acceptance claim.

### Phase B: OO isolated vertical slice

Depends on authoritative incident resolution, approved pinned project/parent, supported branch creation mode, and authorization for cloud operations/hosted execution.

1. **OO-B1: Own resource lifecycle.** Adapt the smallest existing provision/delete helper path for one PR/SHA/run/attempt. Preserve strict project/parent/branch/endpoint checks and use schema-only synthetic fixtures. Keep the Neon control-plane credential isolated from app/test/model steps. Never reuse/delete a preview branch simply because its name matches.
2. **OO-B2: Start one candidate app.** Keep a test-owned loopback app on the hosted runner with the isolated Neon connection. Choose supported config `app.command` or a tiny existing lifecycle wrapper based on installed types. This need not allow remote origins. Retire prepared-target assumptions that exist only for legacy compatibility, but retain target ownership and synthetic-data enforcement. Do not open the internally isolated local Docker network to the model gateway.
3. **OO-B3: Prove the existing vendor story.** Seed only the fixed submitted application and test-owned `vendorManager` user. Use the actual tester-style test as the first slice, then strengthen exact resource identity/authorization oracles where needed. Run one worker initially, retries `0`, documented model bounds, read-only replay.
4. **OO-B4: Hosted evidence and fault paths.** Through supported GitHub Actions, prove the visible outcome, candidate SHA, branch identity, report, and teardown on pass/fail/partial setup/cancellation. Capture warm and deliberately cold replay plus a deliberate test failure. Expiration/reconciliation protect runner-loss cases. Preserve original evidence when diagnosing a failure.

Exit B: one genuine story proven in authorized hosted CI with isolated state and lifecycle evidence, not list output or a mocked acceptance marker.

### Phase C: OO critical stories, native Linear, and cutover

1. **OO-C1: Add high-value outcomes, not cohorts.** Prioritize auth/RBAC, submitted vendor queue/detail, badge/cart behavior, and exact pricing/checkout only as product risks require. Keep monetary/RBAC assertions deterministic. Use test-owned payment resources only after their scope is approved; no production Stripe side effects. Keep exact terminal assertions/fixture lifecycle visible in the ledger.
2. **OO-C2: Enable native Linear per project.** Select intended TesterArmy project, Linear workspace, and default team, approve project-shared OAuth scope, then connect under Integrations → Linear. Determine how confirmed CI defects legitimately enter hosted Issues before promising automation. With ticket-export authorization, prove one confirmed defect export and repeat-export behavior. If ingestion is unsupported, document a supported hosted-testing/manual triage route rather than add a custom bot.
3. **OO-C3: Make replacement authoritative.** Required selected-story completeness and execution exit govern admission. Keep existing release admission connected to the exact candidate SHA. Do not let path skips, empty selection, untrusted PR restrictions, or cleanup steps manufacture green acceptance.
4. **OO-C4: Delete classified legacy architecture.** After replacement proof, remove obsolete Playwright test-runner configs/scripts/cohorts/duplicate workflow invocation and stale docs/specs. Retain `playwright` or other engine dependencies when `@e2e-dev/web` needs them. Independently review, run fresh relevant gates and supported hosted acceptance on the final cutover revision, then mark tasks complete. Archive only when all native acceptance requirements are satisfied.

Exit C: no permanent legacy second lane, required critical stories proven, native Linear path verified or explicitly still blocked, final candidate admission and cleanup evidence recorded.

### Phase D: TC adapts the proven standard

1. **TC-D1: Reconcile native planning.** Rewrite TC's same-named change around product outcomes and one replacement gate. Classify legacy stories, scripts, configs, E2E lint gates, and dependencies. Preserve useful deterministic API/DB/unit checks, not old browser conventions by inertia.
2. **TC-D2: Make trigger/admission real.** Current CI is manual only. Design supported PR admission plus any manual diagnostics, consistent with repository policy and existing deployment admission. Verify required status names and permissions before changing external settings. Do not just append a permanently optional `test:agentic` job.
3. **TC-D3: Add isolated Neon lifecycle.** Reuse the proven minimal lifecycle policy, not OO's preview binding stack. Approve TC project/parent and synthetic seed independently. Verify target/app readiness, migrations, permissions, per-attempt identity, and teardown. Keep disposable PostgreSQL unit/integration where it cheaply exercises independent invariants.
4. **TC-D4: Prove login slice then real TC outcomes.** Verify current auth test with seeded isolated users, strengthen an authenticated-session oracle, and add selected city/admin/attendee outcomes from the actual ledger. Exact permissions and mutations need deterministic assertions and isolated records.
5. **TC-D5: Connect native Linear and delete legacy.** Confirm TC's own hosted project/team mapping, prove an authorized native export, then cut over and remove classified old runner pieces after fresh hosted proof. Measure the same warm/cold cost, runtime, reliability and cleanup criteria as OO.

Exit D: TC has its own hosted evidence and deletion manifest, not only OO's results or a transplanted config.

## Initial OO story inventory

All rows below are **inventory only**, not passed, migrated, or approved for shared external services.

| Candidate | Source | Observable result / exact oracle to preserve | Planning status |
| --- | --- | --- | --- |
| Submitted vendor application | `packages/e2e/tests/agent/vendor-application-readonly.e2e.ts` | Vendor manager sees seeded `Test Anime Goods Co.` pending/submitted row; UUID detail URL, selected-application region, Submitted status, business/contact/product fields. Verify identity/access against the synthetic record. | Existing tester-style foothold, first hosted slice. |
| Vendor queue/workspace | `packages/e2e/tests/staff/vendor-applications-workspace.spec.ts` | Owned UUID-prefixed records and queue/search behavior. Read terminal assertions before selecting the required result. | Legacy behavior inventory, no parity mandate. |
| Badge/cart reader | `packages/e2e/tests/web/badge/badge-purchase-readers.spec.ts` | Reader/cart behavior isolates browser context and excludes DB/Stripe/email/webhooks. Extract exact cart assertions. | Prefer lower-side-effect story before payment integrations. |
| Payment smoke | `packages/e2e/tests/payments/badge-purchase-smoke.spec.ts` | Attendee adds GA badge then T-shirt, cart counts `1 → 2`; terminal checkout state requires further inspection and approved test payment fixture. | Do not infer payment completion from cart assertions. |
| Promo pricing | `packages/e2e/tests/wave-5/purchase-promo-fee-display.spec.ts` | Product/type/tier scope and correct coupon inclusion/exclusion, precise displayed totals if required. | Conditional on approved test-owned payment resources. |
| Admin RBAC mutation | `packages/e2e/tests/web/admin/admin-rbac-writer-journey.spec.ts` | Exact allow/deny and persisted tenant/department/session boundaries. | High-risk isolated writer story, not part of first read-only slice. |

## Initial TC story inventory

All are assertion-grounded **inventory only**, not passing tester-army acceptance. Existing source setup is not a requirement to preserve brittle seed context or browser mocks.

| Candidate | Source | Persona / observable outcome / exact oracle | Planning status |
| --- | --- | --- | --- |
| Participant valid/invalid login | `packages/e2e/src/tests/auth/login.spec.ts` | Logged-out participant submits credentials, chooses burn, reaches dashboard with nonempty identity; invalid email never authenticates and supported syntax errors show `Please enter a valid email address`. | Strong first product story beyond staff setup. |
| Staff authenticated identity | `packages/e2e/src/agent/auth-login.e2e.ts` | `staff-admin` signs in, reaches dashboard/admin, session endpoint returns normalized email and role `admin`. | Existing tester-style foothold, not participant-story parity. |
| City planning + zone lifecycle | `packages/e2e/src/tests/admin/city-planning.spec.ts` | `placementLead` with `MANAGE_CITY_PLANNING` sees title/cards/description; owned zone create/rename/delete has HTTP `200` and corresponding rendered state, department `PLACEMENT`, readiness `true`. | Distinct writer story with exact ownership/permission checks. Historic map/popover cases are skipped and not current product behavior. |
| Admin roster/commerce controls | `packages/e2e/src/tests/admin/admin-attendees-commerce.spec.ts` | Admin sees `Sell Ticket`, not `Invite Attendee`; refund dialog explains access revocation without deleting record; bulk actions appear on selection; legal name visible under approved permission. | Current source is read-only refund preview, **not completed purchase/refund**. Preserve that distinction. |
| Attendee guidance and tour persistence | `packages/e2e/src/tests/attendee/onboarding-guidance.spec.ts` | No-membership user sees `Join a Burn`/`Create a Burn`; member sees Tickets/Camp/shift cards in row-major order; closing tour persists `skipped: true` and prevents restart after reload. | Replace mutable historic seed selection with deterministic fixture state. Post-purchase toast test is skipped pending a real test payment harness. |

Do not label these as generic trade/social or checkout completion. Select exact requirements with independent scenarios so auth/setup failure does not suppress every result.

## Execution decisions and external dependencies (updated 2026-10-02)

- **Branches:** jcode coordination stays on `main`; OO and TC delivery work belongs on each project's `dev`. Existing local planning feature commits must be fast-forwarded/reconciled to `dev` before implementation, without losing unrelated changes.
- **Approved execution:** scoped pushes, supported test-workflow dispatches, and disposable Neon operations after local checks pass. Deployment is permitted only after the given project is green; verify project-specific admission and tested revision before any deployment. No deployment has occurred.
- **Neon remediation:** the user assigns credential investigation/remediation to another session. Assume that session will handle it for scheduling, not that it is already resolved. Remove incident investigation from this execution team's work; request a sanitized readiness confirmation before actual DB/branch/credential operations. No credential values required. Safe local planning/guard work can proceed in parallel.
- **Destinations:** OO hosted project `priceless-workspace/projects/bKutI8aFf99c` (UUID `6062745f-990d-45e6-9663-707454b0452a`), Linear `otaku-odyssey-b2d8dffd23ac`. TC created in `priceless-workspace`: short ID `EAoJgtTyeskl`, UUID `ac93644a-c788-4303-a77e-6feb67f8ff0c`, Linear `tribal-cities-e93462970209`. TC metadata uses documented `https://dev.tribalcities.com`; TesterArmy labels its default environment Production even though URL is development. No runs/schedules started.
- **Execution started:** OO and TC planning commits are reconciled onto `dev`, unrelated dirty files preserved. Verified/pushed OO `60c785fd55` (bounded config, 14 offline runner/ownership tests, typecheck/format/OpenSpec) and TC `1f3067e2e` (bounded config, 21 offline target/collection tests, native agent collection plus preserved 280-file legacy diagnostic, typecheck/format/OpenSpec). These are local guard preparation, not hosted story acceptance. Next exclusive workers prepare TC credential-free PR/static admission and OO offline attempt ownership/lifecycle guards. No live Neon operations before externally confirmed readiness.
- **Hosted static CI evidence:** TC added credential-free GitHub-hosted PR/static-dispatch job (`1e72759fc`), with all fleet DB/browser jobs restricted to manual full scope; workflow contract tests and actionlint pass. Run `37036201438` on that SHA failed during install because existing E2E manifest dependencies were unsorted and Playwright specifiers disagreed. Minimal manifest/lock repair `a027a3880` passes local workspace/frozen-lock/type checks and was pushed. Rerun `37036944711` passed install then failed existing API/Next source formatting, not product acceptance. Formatter repair `c31a86bb5` was verified across 18 files as exactly pinned Prettier output from HEAD; full local format passes 13/13 workspace tasks. It preserves the other session's Node25 runtime commit `9cc3f4fe0`; Run `37038777813` subsequently reached unit tests and failed 8 suites/19 tests (1184 passed) due Node25 native localStorage shadowing jsdom. Split-step bounded workflow `24f6baecf` exposed the same failure in run `37040933034`; install/lint/workspace/format/typecheck all passed. One-line Vitest worker `--no-webstorage` fix `32af16012` preserves real jsdom storage and passes all eight formerly failing suites (44 tests) locally without assertion changes. Hosted run `37043479059` passed install, lint, workspace policy, format, typecheck, unit tests and security audit contracts, then failed production dependency audit. Narrow existing XML override patch `1da172cee` (local TC dev, not yet pushed) upgrades `@xmldom/xmldom` to 0.8.15; frozen lock/workspace/audit contracts pass and XML advisories disappear. Full audit still reports 28 high/critical advisories, including framework/native/transitive packages and a node-forge advisory with no patched release reported. Broader dependency remediation requires a bounded separate decision; do not bypass audit or deploy. OO offline helper hardening `288943a6c0` passes 12 mocked tests/typecheck and was pushed; existing preview source-name compatibility remains, while a future acceptance caller must supply approved exact parent ID before live activation. No product acceptance or deployment implied.
- **Defect export gate:** authenticated MCP inspected all OO hosted issue states: zero issues. TC is newly created. Native Linear connection/export is not exposed in the 50 MCP tools. Do not fabricate a defect or use direct Linear ticket creation as native-integration proof. Await confirmed hosted issue evidence plus the native UI path.
- **Approved integration actions:** create TC's TesterArmy project and export exactly one confirmed defect through native Linear to verify integration. Do not fabricate a defect or ticket, create duplicates, or infer approval for a new OAuth/access grant. Use existing authorized connection where possible; human approval may still be required by the provider.
- **Current access evidence:** TesterArmy MCP OAuth authenticated and owner membership verified in workspace `bff02096-be15-436e-a7d1-95841a8f75a9`; hosted project creation succeeded. Remote transport was reconnected with keep-alive after idle closure. Earlier browser/sign-in failures no longer block MCP project operations; native Linear UI authentication/connection remains unverified.

### Historical prerequisites, now superseded where approvals above settle them

1. **Neon incident and resource identity:** authoritative confirmation that the exposed credential is invalidated or account-specific scope is safe, plus approved OO/TC project and non-production parent IDs. Until then no DB/branch/credential operations. Recommend schema-only synthetic fixtures and independent attempt ownership.
2. **Hosted acceptance authorization:** permission to push the exact scoped delivery commits and dispatch the supported test workflows when local implementation is ready. Existing authorization for hosted `OMNI_API_KEY` use is not permission to weaken local network isolation or deploy. Model bounds/provider compatibility still need the pinned-type check.
3. **Linear destination and access:** hosted TesterArmy project for OO/TC, intended Linear workspace/team(s), and approval for project-shared OAuth plus a confirmed test defect export. Recommend native connection and triaged export, no custom bot or automatic every-red-run ticket policy. A plan alone does not authorize consequential access changes.

These do not block completing this skill or planning safely. Ask once when executing work reaches these boundaries.

## Rollout to other projects

Only after both pilots, publish a compact adoption checklist with story ledger, resource lifecycle guard checklist, trusted trigger/admission pattern, native Linear mapping, deletion manifest, and measured results. This requires explicit publishing/remote-mutation authorization. Keep project-specific seeds/auth/deployment/provider configuration local. Do not add a third pilot, browser/device matrix, or shared framework to this planning scope.

Required evidence ledger: project, final tested SHA, hosted run URL/result, required story IDs/statuses, branch IDs and sanitized cleanup events, cold/warm model-call counts, runner duration/cost if measurable, known flakes/exclusions, native Linear issue/ticket mapping and re-export result. Numeric cost targets stay unset until baseline evidence exists.

## Offline integration checkpoint, 2026-10-02 18:39 UTC

- OO `0ed3f2fd8b`: explicit `neon:acceptance:preflight` CLI validates pinned IDs, exact SHA length, positive run/attempt, and a user readiness assertion. It does not verify external readiness or invoke providers. Ten real CLI subprocess regressions passed.
- OO `4ec1d469e3`: verified newly created candidates are rolled back after readiness/connection failures. Reused or unverifiable create responses are not deleted; rollback rechecks exact parent alongside project/name/ID/protection. Nineteen fake-boundary tests, scripts typecheck and scoped formatting passed. Ambiguous create failures still need expiry/reconciliation, not unsafe guessed deletion. No live cleanup proof.
- Both commits are local on OO dev, not yet pushed. Existing user working-tree files were preserved.
- Actual acceptance orchestration remains in progress with exclusive script ownership. It must preserve genuine CI context, canonical database ownership marker, nonproduction environment and Stripe test-mode admission. Preflight is not lifecycle completion.
- TC audit investigation used an incomplete cached hosted log. Package groups include Next/sharp, Expo/Metro/RN transitive dependencies, undici, brace-expansion and image-size. Exact remaining advisory scope requires a complete fresh sanitized report. No broader dependency updates or exception changes authorized/performed.

## Paused at decision boundary, 2026-10-02 18:44 UTC

Lifecycle author is stopped with no retained lifecycle edits. Source inspection confirmed a fixture conflict: `seed-test-personas.ts:1125-1129` creates the submitted vendor required by the first story, while `seed-test-con.ts:3301-3340` promotes that persona to approved for legacy invoice tests. The full test-con seed also requires Stripe-backed work. No safe compatible existing submitted-state route was established.

User decision requested: approve a narrow submitted-vendor story fixture without Stripe operations, preserving admission checks, or retain the broad fixture with a separate submitted application. Separately, approve bounded production dependency remediation in TC after a complete audit, excluding new risk exceptions and framework-major changes. No answer received yet. Automated incomplete-todo reminders do not resolve these decisions.

Remaining lifecycle, security remediation, native confirmed-defect export, live Neon activation and hosted proof/retirement/deployment are deferred, not completed. Resume automatically when their explicit prerequisites are supplied. No live operations or additional lifecycle edits occurred.

## Whole-result revalidation, 2026-10-02 18:45 UTC

Fresh checks against committed OO result: 10/10 CLI subprocess regressions, 19/19 Neon adapter fake-boundary tests, `@oo/scripts typecheck`, scoped Prettier and `git diff --check` all passed. Public `pnpm --silent neon:acceptance:preflight` accepted valid synthetic arguments with `mode=preflight-only` and `readinessAssertion=user-asserted`; missing arguments exited 2. No provider calls were made.

Requirement-to-check mapping:
- CLI dispatch/output: actual package script invocation confirms honest preflight-only output, not readiness verification or acceptance.
- Input rejection: subprocess tests cover missing readiness, duplicate readiness/project, unknown/positional args, 41-character SHA, zero run/attempt; valid 40/64-character SHA passes. Invalid project syntax was checked in the initial implementation, not retested in this latest suite.
- Newly created resource failure cleanup: fake runner observed delete after readiness and URL-resolution failures; reused branch failure never deleted.
- Deletion authorization boundaries: tests reject mismatched/unverifiable create identity, changed parent/protection, wrong project/name, duplicate identity and lookup failure; missing resource is idempotent success.
- Cleanup error behavior: test observes generic surfaced failure without raw provider details. Normal original errors remain preserved when rollback succeeds; this is not a general secret-redaction audit.
- Existing integration compatibility: scripts package typecheck and reuse-path regressions passed. Actual preview workflow, live Neon deletion and hosted story execution remain untested.
- Working-tree preservation: only pre-existing `.env.test`, `.neon`, `.firecrawl`, and `neon.ts` remain dirty/untracked.

Evidence supports the bounded prerequisite changes, not the full user outcome. Lifecycle fixture conflict and approval gates remain unresolved; no full acceptance/deployment claim is warranted.
