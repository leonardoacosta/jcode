## 1. Regression coverage
- [x] 1.1 Add table-driven parser regressions for string, null, boolean, array, and object probability values alongside numeric entries totaling one. Confirm failures reproduce the current coercion bug.
- [x] 1.2 Add a valid numeric-zero control preserving current acceptance and thresholds.

## 2. Scoped fix
- [x] 2.1 Replace nonnumeric-to-zero coercion in `parse_poke_assessment` with explicit `MalformedResponse` abstention, preserving all existing validations.

## 3. Verification and delivery
- [x] 3.1 Run targeted `jcode-app-core` poke parser tests and confirm regression and control cases pass.
- [x] 3.2 Run the relevant package suite and formatting checks, recording any external blockers without claiming unrun checks passed.
- [x] 3.3 Inspect the final scoped diff, preserve unrelated working-tree edits, and commit only approved change files using configured Git identity.

## Verification evidence
- Before fix: nonnumeric regression failed with `accepted nonnumeric probability: "0"`.
- After fix: `cargo test -p jcode-app-core --lib` passed: 1530 passed, 13 ignored, zero failures.
- `git diff --check` passed. File-level rustfmt check fails on pre-existing formatting, confirmed against HEAD; unrelated formatting was preserved.
