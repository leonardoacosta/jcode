## 1. Implementation
- [ ] 1.1 Generate golden fixtures from `jcode-harness-api` types into a checked-in directory
- [ ] 1.2 Add TypeScript test parsing fixtures and comparing version constants
- [ ] 1.3 Add server tests for unsupported version, malformed frame, cancel ordering

## 2. Verification
- [ ] 2.1 `cargo test -p jcode-harness-api -p jcode-harness-api-server`
- [ ] 2.2 `cd sdk/typescript && npm test`
- [ ] 2.3 Negative check: temporarily change TS minor constant and confirm the drift test fails
- [ ] 2.4 `openspec validate add-harness-sdk-contract-tests --strict`
