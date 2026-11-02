# Tasks

- [ ] Expose server git hash and dirty flag in the protocol handshake.
- [ ] Extend `jcode version --json` with source, client, server and verdict fields.
- [ ] Update `/version` rendering with one row per field and ellipsized hashes.
- [ ] Add tests for matched, mismatched, dirty and unknown verdicts.
- [ ] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [ ] Run `openspec validate bind-acceptance-evidence-to-runtime --strict`.
