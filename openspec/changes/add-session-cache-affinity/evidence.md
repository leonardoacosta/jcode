# Historical blocked checkpoint: 2026-11-10 12:52 CST

This section records the earlier failed gate. See the later approved repair and fresh green gates below for current state.

## Implemented and verified

Opt-in config/runtime implemented through two swarm lanes. Default false. Persisted profile/session opaque key, matching header through chat/Responses and retry, explicit override conflict/invalid-value rejection. Independent review found padded SP/HTAB identity ambiguity and metadata network before invalid-affinity rejection. Both reproduced RED then repaired; post-repair source review PASS. CLI profile initializer includes explicit false. Existing dirty Azure test file includes only one owned added None argument; other hunks must not be staged.

Final package runtime: 211 passed, 0 failed, 2 pre-existing ignored. Config types:29 passed. Base provider tests:183 passed. Dedicated actual MultiProvider lifecycle:2 passed, existing session binding:4 passed. Scoped rustfmt, diff check and strict named OpenSpec passed. Runtime tests initial budget timeout passed isolated and full subsequent runs; do not erase the historical failure.

## Test RC

Built with scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode. Initial missing NamedProviderConfig initializer in CLI repaired and rebuilt. Repaired staged RC SHA256 0ceebc023d9335677767486d7f8fc45009c8e725bcb5cd2424607f88bb214dc5, version v0.99.212-dev(cc84a9723,dirty), path ~/.jcode/scratch/session-affinity-rc/jcode-rc-final. Active shared binary unchanged. Private JCODE_HOME/profile/socket used, no live profile enabled.

Actual RC command workflow passed tool read/result round-trip, persisted session resume across separate processes and append-only turns. Final warm turn12672/12919 cached, idle/resume12800/12961 cached (~98.8%). Transcript records two actual read tool calls. This is real Jcode/gateway/provider acceptance, not only HTTP synthetic fixtures. Private test daemon used earlier RC for debug-dependent root tests and exited after task timeout; never claim final RC daemon reload was exercised.

## Full RC remains BLOCKED

Full E2E rerun:61 passed,0failed,9existing ignored. Previous desktop_busy_owner_disconnect_with_successor_finishes_original_turn timeout passed isolated and full rerun, no unrelated source edited.

Root unit gate initially347/353 due environment-sensitive tests. Test-only provider env isolation, external TMPDIR and private debug-enabled RC server produced353/353 on pre-review-repair build. Final repaired-source root run instead351passed/2failed: cli::login::jcode_device::tests::cancellation_during_consumed_exchange_finishes_and_returns_the_key and polling_pending_slow_down_then_approval. Isolated login module reproduced3passed/2failed, with polling_denied_has_clear_redacted_error and polling_pending_slow_down_then_approval failing. Exact failing set varies; transport/backoff timeout class confirmed, root cause not proven. Tests use unrelated existing single-read localhost fixture, but that is only a lead. No test skipped, timeout increased, source changed or gate waived.

Required full RC is not green. Do not promote, enable active profiles, commit under mandatory terminal gate, or archive. Source remains uncommitted and branch/main/default mismatch preserved. User decision needed: authorize bounded investigation/repair of unrelated login test fixture within existing dirty checkout, or retain blocked RC until its owner resolves it. Unrelated changes must not be silently included in promotion.

## Evidence locators

Background tasks:4573037lx6(build+life),52132639zv(final RCtool/resume),716317kreg(idle),682004ifj0(config/provider),566322i5u2(E2E61pass then root failures),6290488qvv(isolated login failure). Worker runtime logs scratch/affinity-review-serial.log; red regression scratch/affinity-review-red.log. No secret values included.

# Approved repair and green gates: 2026-11-10 13:24 CST

User approved bounded fixture repair at12:55 CST. Instrumented fixture captured unrelated GET / probes (Go-http-client/1.1) consuming scripted responses before real POST, causing ConnectionRefused/backoff timeouts. Deterministic GET-before-POST regression failed on predecessor (428 instead of404). Fixture-only helper now filters expected POST /v1/auth/token, drains complete Content-Length, bounds headers/body64KiB and idle reads2s; both scripted and cancellation listeners use it. Production login behavior unchanged. Diagnostic stress removed; final tests.rs SHA2567cb1bd0e0b3350280799e001c8b0193cbd4c3e59667c78048ffd6624452d9cea.

Final module7/7 passed three serial runs (177905h33h), coordinator7/7 rerun563717sldn. Final frozen whole RC5949035121 passed355root,8binary,61E2E with9existingignored at13:24:04CST, no failures. Earlier oversized fixture assertion failure ran during pre-freeze bounds editing and is not final-source evidence. Fresh full diffcheck and strict named OpenSpec valid13:26CST. Independent post-gate source and staged-scope review PASS at13:30CST. Actual commit-range and clean-artifact review remain pending. Promotion/active-profile enablement not performed; clean commit-only artifact acceptance still required to avoid including unrelated dirty source.
