# Tasks

## Execution checkpoint: 2026-11-04 UTC

Selected by the user with `route-jev-supervision-explicitly` and
`evaluate-code-search-strategies`, source base
`6393b322b481453c058c504cec087d982abc74ea`. Strict validation and canonical
resolved-path dependency validation passed for the complete selected collection.
P03.O3 reciprocally binds every requirement/scenario to the six tasks below.
The archived P03.O1 prerequisite records actual isolated TUI session-log acceptance
and typed failed-required-check evidence, not merely archive presence.

Initial implementation was dependency-blocked until explicit routing was freshly
verified, independently reviewed, archived and integrated. That prerequisite is
now satisfied by the coordinator-confirmed upstream `358294f93` route closeout. Final shadow acceptance must include actual
public runtime/replay interfaces and failure/recovery paths. Synthetic fixtures
cannot establish live semantic quality. The roadmap additionally calls for
redacted actual-session replay through an authorized inference path. No paid
inference, external evidence disclosure, shared RC activation, overnight work or
production semantic decisions are authorized by this execution request.
The initial preflight alone completed no implementation task. Fresh offline
implementation evidence is recorded below, without claiming semantic acceptance.

Fresh prerequisite checks on the current base, local-only via `selfdev test`:
`JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-todo-policy --lib`
passed 78 tests (task `983113zmr8`), and
`JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-tui continuation --lib -- --test-threads=1`
passed 23 tests (task `005509aox4`). These refresh project regression evidence,
not live semantic evaluation or acceptance of the unimplemented shadow feature.

- [x] Add opt-in shadow flag; call the supervision client with the evidence record under a bounded timeout.
- [x] Log verdict, latency and call count; map errors, timeouts and malformed output to `uncertain`.
- [x] Add replay comparing verdicts with decisions, with negative controls (waits, blocked work, premature stop, required-check failure).
- [x] Tests: decision unchanged in shadow; failure paths; replay missed-check and missing-evidence cases.
- [x] Run `cargo test -p jcode-todo-policy` and `cargo test -p jcode-tui poke`.
- [x] Run `openspec validate admit-semantic-poke-in-shadow --strict`.

## Offline implementation evidence: 2026-11-04 UTC

Test-first RED failures were observed through `selfdev test`: absent opt-in
field `67125737o6`, absent replay API `7283650bom`, invalid replay latency
`8517352y04`, absent observer `875902hwc0`, and cancellation-accounting/public
CLI failures `005561pnih`. Their subsequent GREEN results include
`697783nmcf`, `8238222xy1`, `963354ruon`, and `501946npe8`.

Final behavioral batch `501946npe8` ran local-only with
`JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh` and passed:
- `test -p jcode-todo-policy --lib`: 81 tests.
- `test -p jcode-tui --lib semantic_shadow -- --test-threads=1`: four tests,
  including actual production-caller deterministic queue/cap parity.
- `test -p jcode-tui --lib continuation -- --test-threads=1`: 24 tests.
- `test -p jcode-tui --lib auto_poke -- --test-threads=1`: 32 tests.
- `test -p jcode --lib semantic_poke_replay -- --test-threads=1`: one test.
The config default-off test passed separately in `466073wg7o` before that
batch stopped on a test-fixture move error, repaired before the final batch.
The loopback observer test exercises 48 requests with continue/stop/wait,
malformed output, HTTP 503, timeout, and 49th-request cap plus route changes.
Cancellation records started calls, releases the single-flight permit and maps
to uncertain. Missing/busy configuration creates no request. Replay validates
strict evidence, duplicates, missing/empty inputs, enums, size and total overflow.

`openspec validate admit-semantic-poke-in-shadow --strict`, exact owned-file
`rustfmt --check`, and `git diff --check` passed after implementation.
Scoped non-strict clippy `523049c8c7` exited zero with existing warnings and
owned test warnings. Owned test warnings were then repaired, so that run is not
claimed as a clean or strict final lint pass. Coordinator final gates must
refresh the final bytes after those test-only repairs.

## Coordinator verified offline checkpoint (2026-11-04)

Candidate build `351394jfl0` passed without activation. Actual candidate SHA-256
`15a936835c763789a817893f61f10dfc704e37733b2673f63fb309a17d65c7ee`.
Real isolated TUI/daemon loopback execution and public replay passed:
`scratch/p03-shadow-runtime-iif1dpet/results.json`. An actual bound bash exit 7
remained a failed required check, deterministic queued=false despite shadow stop,
one request, and replay missed-check=1. This is synthetic advice transport
acceptance, not live semantic quality. Actual route CLI eight cases passed again
in `scratch/p03-route-public-8_lr14gx`; public complete/dropped/legacy provenance
controls passed in `scratch/p03-capacity-public-zlt8n8vo` with unknown/redundancy
results 0/1, 1/0 and 1/0 respectively. Earlier five invalid replay input cases
also returned nonzero (empty, missing evidence, malformed, oversized and FIFO).

Mapped final gates: `429298m4zx` policy 82 and TUI poke 45 passed before the
preserved fixture failure described below. `537381wpup` passed all four shadow
tests three consecutive times after its test-only framing repair. `553393r791`
passed base Jev 74 (two live tests ignored), root status 5 and root all-target
check. `7730545qnc` passed config/policy strict Clippy and root/TUI no-deps lint
with only documented existing warnings. Scoped rustfmt, whitespace, dependency
boundaries and module checks passed. Native strict validation passed 69/69 in
`scratch/p03-amended-native-final.json`; Python evaluation regressions passed 19.
No whole-suite stability or whole-repository strict-lint claim is made.

Independent full-collection reviewer `session_spider_1793815384494_d3c0de029a4f324f`
requested SHADOW-R1 amendments against the original snapshot, then independently
reverified the original owner's repair and all selected integration boundaries.
Final verdict: APPROVE for offline checkpoint correctness only, base
`6393b322b481453c058c504cec087d982abc74ea` through `358294f93` plus dirty diff
SHA-256 `7dfc7aa18840a816e9cc2590d686c1f43e26ea327384a46b635bef1e6e4447e0`
and the included new modules/design. No unresolved actionable finding.
The reviewer also preserved an unrelated memory-agent HTTP-probe fixture failure,
not a regression attribution or a passing routine-suite claim.

The complete selected outcome remains BLOCKED on authorized fixed-model search
trials and redacted actual-session semantic-quality inference. Keep this change
active and unarchived. Offline checkpoint persistence is not proposal completion,
production semantic activation, or permission to spend/send private evidence.

## Required acceptance and terminal state still open

### Review repair SHADOW-R1: check-capacity loss

Independent review found that eight retained passes could hide a ninth failed
required check. Actual `RequiredChecks` and `RequiredCheckBatch` regression
`semantic_shadow_replay_dropped_required_checks_are_unknown_never_redundant`
failed RED in `1036413y0q` with unknown coverage incorrectly zero. The repair
preserves distinct `required_checks_complete` metadata from actual accumulator
validation/capacity and batch truncation. Legacy records default to incomplete.
Replay counts incomplete coverage as unknown and prohibits hypothetical
redundancy; observer abstains without a request. The all-eight-pass control still
permits comparison despite generic redaction `truncated=true`.

GREEN `143679h5ph` passed policy 82, TUI semantic-shadow four, continuation 24,
and strict no-deps policy clippy (`-- -D warnings`). Source repair touches only
`evidence.rs`, `shadow.rs`, and `input/semantic_shadow.rs`. Coordinator must
refresh public binary replay/candidate and independent review against this schema.

Coordinator final batch `429298m4zx` exposed a loopback fixture-only framing
failure: an accepted bodyless request lacked `Content-Length`, panicked the
fixture, and shifted the expected response controls. The test server now skips
empty connections, answers non-POST health probes without consuming a case,
and frames length/chunked requests with bounded reads. An injected empty
connection and GET probe verify case alignment before the same 48 actual POST
controls. No product code changed for this repair. Three consecutive full
semantic-shadow runs passed (four tests each) in `537381wpup`.

Implementation checks above do not complete the proposal. Coordinator owns the
isolated candidate build, actual public CLI/runtime loopback acceptance and final
independent review. The roadmap's redacted actual-session semantic replay still
requires an authorized inference path and explicit spending/disclosure bounds.
Synthetic loopback success establishes transport integration, not semantic quality.
No savings claim, GO activation, user-config change, reload, commit, push or archive
was performed by this implementation worker. Keep proposal closeout blocked until
all required acceptance and coordinator terminal gates have evidence.
