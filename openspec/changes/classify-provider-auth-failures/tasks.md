## 1. Implementation

- [ ] 1.1 Add `ProviderFailureKind` and classifier in `jcode-provider-core`
- [ ] 1.2 Wire Anthropic and OpenAI runtimes
- [ ] 1.3 Render kind-specific remedies
- [ ] 1.4 Tests: classifier table for each kind; refresh-cancel leaves creds unchanged

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-provider-core -p jcode-provider-anthropic-runtime -p jcode-provider-openai-runtime` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate classify-provider-auth-failures --strict`
