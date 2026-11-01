# Tasks

## 1. Config

- [ ] 1.1 Add `agents.roles.{research,edit,compaction,websearch,council}` optional model fields.

## 2. Resolution

- [ ] 2.1 Add optional `role` to swarm spawn; resolve explicit model, then role slot, then `swarm_model`, then inherit.
- [ ] 2.2 Use `compaction` and `websearch` slots in their internal callers when set.
- [ ] 2.3 Apply a read-only tool profile for `research` workers.
- [ ] 2.4 Report source `slot:<role>` via `show-resolved-worker-model` reporting.
- [ ] 2.5 Show configured slots in `swarm list_models`.

## 3. Verification

- [ ] 3.1 Unit tests for resolution precedence and `slot:<role>` source.
- [ ] 3.2 Test research worker cannot call edit tools.
- [ ] 3.3 Run `openspec validate add-role-model-slots --strict`.
