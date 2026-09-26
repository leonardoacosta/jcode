# Implementation verification

Approved 2026-09-26, revision `9635a9425`. Implementation remains open until the operational acceptance gates below pass. No production service or Tailscale configuration changed.

## Current evidence

- Final browser follow-through: `05504820lj` rebuilt the corrected binary and passed all eight HTTP/UI tests. Real Chromium keyboard pairing, pagination to the second page (nine retained rows), page retention across five-second polling, focused input preservation, automatic three-occurrence preview, and keyboard result expansion surviving polling all passed. Restart retained the two successful fixture records with definitions paused and no replay.
- Browser checks caught and fixed stale `dirty.form` references, duplicate hidden `edit_id` inputs, an obsolete inline HTMX handler incompatible with no-eval CSP, and focus protection suppressing deliberate pagination. Regression assertions cover the asset defects. Owned browser and isolated daemon were closed afterward.

- `777039ehbc` (36 library tests and two CLI tests): `cargo test -p jcode-app-core automations:: --lib`, `cargo test -p jcode cli::automations:: --lib`, and `cargo check -p jcode --bin jcode` passed on the follow-through implementation.
- Deterministic real-Agent tests verify installed skill activation in the dynamic system prompt, captured workspace/session identity, final response and source digest, provider failure, missing skill/directory, unsupported provider selection, human-input blocking, cancellation acknowledgement, and listener recovery after a bind conflict. A fake provider supplies deterministic responses, so this is representative integration evidence, not proof of a live-provider deployment.
- `471784cekj`: 11 runtime tests passed, including a shortened test-only deadline retaining the run slot until agent acknowledgement and two successful executions followed by store reopen without replay. Production deadline remains 30 minutes. Closing the stream channel no longer bypasses deadline/cancellation observation.
- Storage tests cover version/corruption preservation, single writer, persisted claim, serial execution, fast/slow completion, interval boundaries, DST gaps/folds, invalid future timezone isolation, restart interruption, output truncation and retention.
- Real HTTP tests cover pairing, expiry, local/remote cookie security, origin isolation, CSRF, token replay, revocation, body limits, creation, edit recovery, calendar preview and native edit routes.
- Provisioning tests cover actual flat Tailscale ServeConfig parsing, conflicting routes/Funnel, exact unrelated-config comparison, command output draining, timeouts when descendants retain pipes, and distinct systemd Environment/WorkingDirectory/ExecStart quoting. CLI initialization and provisioning share a tested exclusive mutation lock. These tests perform no live Tailscale or systemd mutation.
- Fresh TUI build `286042n7ow` passed using `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode`. Coordinated builds had aborted before execution during external reloads, so the documented fallback was used.
- Managed Chromium session `automation-bulletin-chick`, isolated daemon/data/socket/runtime: pairing, one-minute creation, calendar edit, validation recovery and pause persisted correctly. At 390px width there was no horizontal overflow.
- With all application JavaScript requests blocked (`typeof htmx === "undefined"`), native edit navigation restored saved calendar fields, changing to `15m` persisted 900 seconds without duplicating the definition or enabling a paused schedule, and resume/pause redirected to controls reflecting the new state. Native preview returned three occurrences. A final follow-up adds a full native preview page and return link, covered by HTTP tests.
- Initial isolated local-provider attempts failed because no Ollama service was listening. A loopback deterministic OpenAI-compatible HTTP fixture then exercised the actual provider adapter and daemon: two unattended runs persisted `Success` and `BULLETIN_OK` at 08:28:30Z and 08:29:31Z (`3565992a4q`). No browser/TUI was active. This validates scheduling/transport/session/output integration but is synthetic provider evidence, not live model quality or account readiness. Fixture and daemon were stopped and the isolated definition paused.

## Browser-driven correction

Chromium native form POSTs under `Referrer-Policy: no-referrer` emitted a null Origin and were correctly rejected by CSRF checks. The implementation uses `strict-origin` instead: origin validation still applies, while referrers exclude path/query bootstrap tokens. Native browser pairing and forms passed after this change.

## Remaining acceptance gates

1. Live-account/provider smoke check. Two unattended real-daemon runs through a deterministic HTTP provider and final-binary paused-definition restart passed, but live model/account readiness remains unverified.
2. Real Tailscale HTTPS from a second authorized device, denied-device/outside-tailnet checks, remote outage recovery, and owned-route removal. Requires explicit live provisioning authorization and existing Tailscale login/HTTPS/policy prerequisites.
3. User-service installation, logout/reboot continuity and uninstall in an authorized disposable environment. No linger or service changes were made here.
4. Broader assistive-technology audit remains optional follow-up. Requested keyboard controls, no-application-script forms, mobile layout, JavaScript pagination/polling, and deterministic deadline acknowledgement have concrete checks above.
5. CLI status now queries a local-host-only authenticated control endpoint, reports identified bulletin service, persistence readiness and active/enabled counts. This does not prove remote tailnet reachability or a successful scheduler cycle. Remote operational status remains an acceptance gate.

Do not archive this change or claim deployment complete. Tasks stay unchecked where their complete named verification has not passed.

## Persistence and scope

- `f3cf9f764`: durable timezone-aware foundation.
- `e775383d4`: opt-in daemon, HTTP and provisioning integration checkpoint.
- Follow-through changes fix native UI workflows, cancellation races, future-zone recovery and provisioning ownership/process boundaries.
- Unrelated dirty work remains untouched except the necessary missing `pending_question_tx: None` test initializer required to compile existing CLI tests.
- Proposal validation: `openspec validate daemon-skill-automations-bulletin --strict`.
