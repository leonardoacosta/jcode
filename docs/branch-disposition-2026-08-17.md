# Branch disposition manifest, 2026-08-17

This manifest is the recovery ledger for pruning the writable repository's historical `fork/*` branches. The repository policy is:

- `/home/nyaptor/dev/jcode` is the only writable checkout.
- `dev` is the default integration branch.
- `main` mirrors `upstream/master` exactly.
- `source/jcode` is a read-only reference checkout and its refs are not modified.

## Canonical state before pruning

- Upstream mirror target: `4c58869beeb3885e5d772054fd763176c992465c`
- Integrated dev tip after salvage: `7ef8c7f2639da08c683b221bf703697cbdff864c`
- Computer observability salvage: `e7fdddf74`
- iOS selective salvage: `8ca250aa0` through `7ef8c7f26`

Every deleted remote ref remains recoverable by the full object ID below as long as the object is retained locally or fetched from another clone. This manifest also provides the evidence needed to recreate a branch from an archived object ID.

## Dispositions

| Remote ref | Recoverable tip | Disposition | Tip subject |
|---|---|---|---|
| `fork/add-fpt-ai-marketplace-provider` | `271546419d2139a431841e27ded53a8701a2dad8` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Add FPT AI Marketplace provider |
| `fork/agent/fix-739-742` | `f044816bf624a913fc66899bf3bc8de7fdfd380b` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: scope fast badge to OpenAI providers (fixes #739) |
| `fork/agent/issue-699-ctrl-d` | `02f4afc00edb608ac6629538d03c7f487b921380` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | chore: rebaseline size ratchets for triage test additions (refs #694, #695, #699) |
| `fork/agent/release-v0.71.1` | `431ee121915e595854cf41e6331c593e06c33619` | deep-audited; optional legacy desktop state graph declined in favor of current architecture | fix(release): preserve Homebrew launcher arguments |
| `fork/agent/sdk-release-followup` | `2f534cf14ddff8a8c9593d29f49126e262b446ce` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | chore(release): prepare v0.71.1 |
| `fork/agent/triage-2026-08-02-clean` | `30de2a2a1bf8dfa2590de5f15c6bbadeeb789ccd` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | test(terminal-launch): move the portable Windows check to a sibling file |
| `fork/agent/triage-2026-08-06` | `09d328b1c636d5fd706bf23050eb795c5270f47a` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | chore(release): prepare v0.71.0 |
| `fork/agent/triage-2026-08-07` | `9445e7211f974010c17003a5eabfbf562884d8da` | deep-audited as already represented in dev; prune | fix: use platform c_char for tty lookup |
| `fork/agent/triage-2026-08-07-pr` | `47e8819ce8f2c24e59d52f6e1289c9813851ebc3` | deep-audited as already represented in dev; prune | fix(ci): compile macOS notification broker |
| `fork/agent/triage-fixes-20260728` | `111ee842de012511a481064e40964c28bfbb2309` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(tui): drop the reasoning tail when whitespace is appended past it |
| `fork/agent/triage-open-issues-20260809` | `44ffa55281fad71c02be984c0674d92412210452` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | chore(release): prepare v0.73.0 |
| `fork/agent/triage-safe-fixes-20260809` | `7522663b10ffc3a7b7606d7740ae2dc1a84580cd` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(tui): open markdown links from rendered labels |
| `fork/agent/triage-verification-tests` | `35e692bc8758d5c21292b097b27a1f456728d2cc` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(provider): strip own routing prefix in direct runtime set_model |
| `fork/arch/app-decomp` | `7634a9ed8941ce7e41e75f997a05b00ce3ef03cf` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | refactor(tui): move workspace_client process-global into App-owned state |
| `fork/arch/integration` | `46929ad081b2b749c1dcbe6155cb42e7986774ad` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | docs(tui): TuiState decomposition plan + section the 114-method trait |
| `fork/arch/server-svc` | `8bb66ae13b14cf7b89e16ddd9130d83e586d54dd` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | test(server): migrate file-touch tests to FileTouchService |
| `fork/chick/compacted-history-visible-window` | `c37dccf34dc4eadf29e419b14261cf4d13928500` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Fix compacted history visible window |
| `fork/dev` | `0474aaa376843620c938cee49dd9f8cf67465500` | canonical integration branch; update to final dev tip | fix: migrate final telemetry source edit |
| `fork/dioxus-gui` | `7abad06cf0803654bec9bb18fbe60824d00ccf19` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Improve GUI border contrast for OLED theme |
| `fork/dioxus-gui-local` | `e914f97abac50a57180fe254b806c9f89d099e0f` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Merge branch 'master' into dioxus-gui-local |
| `fork/docs/release-base-clarity` | `f1e9af276addc86e3f71516da877b94fcf2ba97b` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | docs: clarify release base selection |
| `fork/feat/computer-observability-348` | `941b44b28190698023e2c0132dca0be5b5a265a8` | integrated by recreation/cherry-pick as e7fdddf74; original ref may be pruned | feat(computer): truthful observability for background AX actions (#348) |
| `fork/feat/issue-664-auto-poke-config` | `8acc3081b91f80409d4a6ec1b15a9283cf3cde43` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | feat(config): let features.auto_poke set the auto-poke default for new sessions |
| `fork/feat/windows-setup-copilot-key` | `526af67e0fff6809de5b272df6d83b1422e76790` | deep-audited as already represented in dev; prune | chore(ci): sync ratchets with latest master |
| `fork/fix-browser-setup-201` | `f5d3630958b0ce04ca7a1695c761b556c4665ed2` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Fix latest import clippy guardrails |
| `fork/fix-set-route-model-alias` | `1252aaf2b871643665a76295de75e4ae256161b6` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Finalize v0.26 rebase integration |
| `fork/fix/acp-mcpservers-tolerated` | `0517f01f2c665cc577c501ac433dd0e359530628` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: tolerate ACP mcpServers during session creation (fixes #887) |
| `fork/fix/computer-tool-schema-and-element-at` | `1677d567af831bec5cfb30c66a5cad7ca9975135` | explicitly declined: ScrollWM cross-repository contract is not approved; recoverable by this tip SHA | docs(readme): document ScrollWM window-management integration for headed swarms |
| `fork/fix/herdr-client-hooks` | `3c57514745bc27f8695dca1c43e829ad32253390` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | feat(hooks): support multiple client-scoped commands |
| `fork/fix/installer-path-idempotency` | `c111c10e3aab61048d21a0905a31a5a7707035ad` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(install): don't touch shell rc files when install dir is already on PATH (fixes #624) |
| `fork/fix/issue-543-mcp-format` | `a52ead259cd48c69c0e78d6a9e2974a54efd5894` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Merge branch 'master' into fix/issue-543-mcp-format |
| `fork/fix/issue-657-tract-023` | `49d9dfd620ac96b882238005511829fbd7c1b53d` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(deps): move jcode-embedding to tract 0.23 to clear RUSTSEC-2026-0217 |
| `fork/fix/issue-662-ci-red` | `c3455d9512e953c8d73dd63022183e6f159e18bb` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | ci: run the embedding numeric cohort, and fail if it silently skips |
| `fork/fix/issue-754-gemini-mcp-notification` | `3625a023044b354ccceaed061d1d2ccd2f5299b1` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | feat: preserve accumulated schema and desktop improvements |
| `fork/fix/issue-759-client-hooks` | `c943b04e8edaca0b3b7946a4bf18a070daa5a16a` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: support composable client hooks (fixes #759) |
| `fork/fix/issue-762-celeris-config` | `d7feb6b54d9aeb5b2145d36bfb5b25bbfd13d2af` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: update Celeris model limits (fixes #762) |
| `fork/fix/issue-763-power-inhibitor` | `dc59f652190ad85a83d06c83fb9d66e6249ed874` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: prevent Linux power inhibitor auth loops (fixes #763) |
| `fork/fix/issue-767-favorite-cycle` | `0251e0803a8cacca8c34831f12cf1dde091fdbaf` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: cycle through every favorite model (fixes #767) |
| `fork/fix/issue-768-menubar-ci` | `1e80a4033216064b7eebda1495251d9976a603ea` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: migrate CLI test fixtures to semantic states (fixes #768) |
| `fork/fix/issue-779-acp-resume-subscribe` | `8d46f643c68e21d70ef5d57c6cf547d4ee528eea` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: subscribe before ACP session attach (fixes #779) |
| `fork/fix/latest-remote-session-bugs` | `fefae76c2f9d14c6fd7923b3adefe8ecec4bd3fa` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: propagate active skills to remote sessions (fixes #873) |
| `fork/fix/menubar-root-sessions` | `aac9555e88b8a31db3e9caec1bed0d26b4572454` | deep-audited as already represented in dev; prune | Merge origin/master into fix/menubar-root-sessions |
| `fork/fix/openai-quota-window-dedup` | `30ec7b33719889d61de23aab15c481fd155a1eb9` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | chore(release): prepare v0.75.2 |
| `fork/fix/pinned-todos-config-cache-isolation` | `446601bc3c9337aa67d3e638814e7ee1c05a9253` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | style: apply workspace formatting |
| `fork/fix/report-alternate-keyboard-keys` | `a1d7db4542515030447ba35951665af26aecd442` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | ci: validate integrated branch |
| `fork/fix/skill-invocation-multi-word-619` | `477cb86c6acc6ab390381d18ade4176a34ed39d0` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(skill): resolve multi-word skill names in slash invocations (fixes #619) |
| `fork/fix/soft-interrupt-images` | `9ea612f9db91d10070847dbe9dbc39f7c1ccc68b` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(interrupt): carry image attachments through soft interrupts (fixes #623) |
| `fork/fix/stream-first-byte-timeout` | `2b0e28b3591543a8a8f145c10443ee867bfc948b` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix(stream): bound time-to-first-byte so a blackholed request can't hang forever (fixes #620) |
| `fork/fix/stream-read-error-retry` | `250c71acd9b8ef50fd48a74e5af8025b69303332` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: recognize stream_read_error as transient transport error (fixes #885) |
| `fork/fix/transport-retry-classification` | `664c04142c1909d6d67070442f0537a62a4f4554` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | test(repro): add standalone TLS BadRecordMac fault-injection reproduction |
| `fork/fix/unique-fallback-tool-call-ids` | `e0aea418ef45d801f241ef29aba936179d14e585` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | fix: generate unique fallback tool call IDs (fixes #884) |
| `fork/fix/windows-global-jcode-path` | `0bb4d1084afca9afbc00bc9ec48ff0f3ff3474ad` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Refresh-stale-code-size-baseline |
| `fork/ios/mobile-real-nav` | `673ff4f3e25974f6fd7a9662ef8aebc1d698f8f4` | selectively ported interaction improvements as 8ca250aa0; remaining release/signing history is represented, platform-specific, or superseded | docs(ios): record CI evidence that only the Apple signing permission blocks upload |
| `fork/ios/ux-production` | `887194bdf6be366876905cd762476885c8cbbc6e` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Merge branch 'master' into ios/ux-production |
| `fork/jcode/configurable-colors` | `53c5c0a2f9a1b3557318b8a841b63314b0b8b554` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | refactor(tui-style): move the measured criteria into their own module |
| `fork/logical-commits-20260322` | `2e89a6b457a6bc0aa97272f52be9759c20ed2866` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Overhaul TUI login and dictation flows |
| `fork/main` | `6e2e9ced5d2382af63b69a246dcbaca884c8d74a` | canonical upstream lineage; replace fork/main with exact upstream mirror and remove legacy fork/master | Merge remote-tracking branch 'origin/master' |
| `fork/master` | `fd1ff012cd463c413d53a3de358ceb7a7b8459a2` | canonical upstream lineage; replace fork/main with exact upstream mirror and remove legacy fork/master | chore(release): prepare v0.75.3 |
| `fork/release-workflow-fixes` | `6dd45bf5d01a323ce21fa48e3fef8e76cdba3538` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Allow manual release build dry-runs |
| `fork/test/preserve-iteration-maturity-fixtures` | `9fc15d499de7d85b26b7d365b20d2204c5ebc74f` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | test(tui): satisfy iteration-maturity gate in todo completion fixtures |
| `fork/triage/issues-2026-07-29` | `fe77ce9c7ec1bc767b4f29489918026805742cca` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | refactor(stdin): source TH_STATE_WAITING from libc instead of a literal |
| `fork/windows-lifecycle-e2e` | `63c77688f1edebdcb949873bb5434855ede4e7d6` | audit classified patch-equivalent, represented, stale, superseded, or duplicated; prune with tip SHA retained here | Use release binary for Windows lifecycle e2e |

## Deep-review decisions

- `fork/main` was integrated as an upstream sync into `dev`; root `main` is reset only to the exact upstream commit, never to local feature work.
- Computer observability had unique value and was ported with its focused tests.
- The active iOS lane received the complete audited non-equivalent sequence. Conflicts were resolved in favor of newer `dev` implementations while clean App Store, demo, notification, test-harness, and release-readiness changes were retained.
- The triage, Windows lifecycle, and menubar branches were already represented in `dev`.
- ScrollWM integration was intentionally rejected because its external contract is not approved. Its exact tip is retained above for future recovery.
- The legacy desktop2 state-graph option was intentionally rejected because the current desktop architecture supersedes it.
- The remaining refs were classified in the 62-ref audit as patch-equivalent, semantically represented, stale, superseded, or duplicated.

## Pruning acceptance checks

1. Push the final `dev` tip and exact upstream mirror `main`.
2. Delete all other writable-repository `fork/*` heads.
3. Fetch with pruning and verify `fork/dev` and `fork/main` are the only remote heads.
4. Verify GitHub's default branch is `dev`.
5. Verify `main == upstream/master` and `upstream/master` is an ancestor of `dev`.
6. Verify the read-only source checkout is clean and unchanged.

## Validation evidence

- `cargo check --workspace --all-targets` passed after the upstream sync and again after the iOS port.
- All iOS TestHarness Python files compiled. Reward determinism checked 16 scorers successfully in an isolated environment.
- The production checklist passed 15 repository-level checks. Eleven Apple-platform checks are acceptance-blocked on Linux because Swift, `plutil`, PlistBuddy, `sips`, XcodeGen, and Xcode are unavailable.
