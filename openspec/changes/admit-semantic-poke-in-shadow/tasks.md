# Tasks

- [ ] Add opt-in shadow flag; call the supervision client with the evidence record under a bounded timeout.
- [ ] Log verdict, latency and call count; map errors, timeouts and malformed output to `uncertain`.
- [ ] Add replay comparing verdicts with decisions, with negative controls (waits, blocked work, premature stop, required-check failure).
- [ ] Tests: decision unchanged in shadow; failure paths; replay missed-check and missing-evidence cases.
- [ ] Run `cargo test -p jcode-todo-policy` and `cargo test -p jcode-tui poke`.
- [ ] Run `openspec validate admit-semantic-poke-in-shadow --strict`.
