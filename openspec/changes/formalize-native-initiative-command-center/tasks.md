## 1. Authority and documentation

- [ ] 1.1 Map the current initiative tool, goal persistence, app-core repository, TUI, daemon API, SolidStart UI, ambient scheduler, run projections, and Orca adapter. Record exact source seams and authority labels.
- [ ] 1.2 Add the native initiative/Command Center architecture guide with the authority graph and concise comparison matrix.
- [ ] 1.3 Add contributor guidance for discovering and extending native surfaces before creating a new UI or persistence path.

## 2. Contract alignment

- [ ] 2.1 Audit TUI, tool, API, and web terminology against the shared lifecycle vocabulary.
- [ ] 2.2 Add or update contract comments and generated documentation without changing compatible wire shapes.
- [ ] 2.3 Document revision, idempotency, reconnect, replay-gap, and degraded-state requirements for mutations and projections.

## 3. Verification and closeout

- [ ] 3.1 Run focused initiative, app-core, Command Center, TUI, and generated-contract tests.
- [ ] 3.2 Run applicable browser workflows and verify no independent frontend persistence exists.
- [ ] 3.3 Run `openspec validate formalize-native-initiative-command-center --strict --no-interactive`.
- [ ] 3.4 Update the proposal with validation evidence and archive only after acceptance.
