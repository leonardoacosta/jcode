# Tasks

## 1. Contract and client evidence

- [x] 1.1 Inspect existing custom request construction, result parsing, fallback order, native-provider eligibility, and existing documentation. Verify exact source spans in design.md and run the current websearch tests locally.
- [x] 1.2 Bind the gateway-owned selector placement/value, accepted authentication header, and result pointers to gateway implementation evidence before contract-specific recipe or tests. Verify agreement with the coordinator and update design.md.
- [x] 1.3 Add characterization tests for no native/local bypass and POST/error/result behavior in the existing websearch test file. Verify with `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core tool::websearch::tests --lib` before documentation changes.
- [x] 1.4 Add the verified OmniRoute configuration recipe and native-search limitations to `docs/custom-web-search.md`. Verify the documented fields against request tests and source, including complete environment-backed header values and no client-side fallback.

## 2. Combined verification and closeout

- [x] 2.1 Run fresh local websearch tests, format/diff checks, and `openspec validate use-omniroute-search-api --strict` against the final client diff. Record exact commands and results in design.md.
- [ ] 2.2 Coordinator verifies the combined gateway/client contract and separately authorized live API/runtime path. Record the actual gateway-native interception routes and unsupported routes without inferring coverage from client tests.
- [ ] 2.3 Coordinator obtains independent final-diff review against requirements after fresh gates, resolves actionable findings, and reruns affected checks/review. Verify the review evidence before finalization.
- [ ] 2.4 Coordinator updates verified workflow state and performs authorized scoped commits/persistence. Verify exact included paths/history. Worker does not commit, publish, or activate user configuration.
- [ ] 2.5 Coordinator archives only after required verification, independent review, and commits pass. Verify archive contents, history, and required persistence before declaring workflow completion.

Gateway selector/auth/result contract is now bound in design.md. Client tasks are verified locally. The worker cannot complete coordinator-owned terminal gates by running local fixtures.
