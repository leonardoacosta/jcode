## 1. Implementation

- [ ] 1.1 Add cancellation select branch and abort handling in `run_turn_monitored`
- [ ] 1.2 Track generation-owned bg/swarm work on cancel using durable control; verify draining/unresolved versus confirmed stopped, no later admission and no false success from a cancellation deadline.
- [ ] 1.3 Add `max_agents`, `max_turn_minutes`, `max_run_tokens` manifest fields and enforcement
- [ ] 1.4 Tests: cancel revokes admission promptly then settles or reports unresolved; spawn beyond cap refused; aggregate token cap includes deduplicated child/resume usage and distinguishes estimates. Verify token caps cannot bypass authoritative monetary dispatch denial.

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate enforce-overnight-cancellation-and-limits --strict`
