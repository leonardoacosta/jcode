# Tasks

## 1. Lint

- [ ] 1.1 Add a `warnings: Vec<String>` (or equivalent) field to the skill struct in `crates/jcode-base/src/skill.rs`.
- [ ] 1.2 On load, compare folder name to frontmatter `name` (namespace-stripped) and record a mismatch warning.
- [ ] 1.3 Log each mismatch once per reload.

## 2. Surface

- [ ] 2.1 Show warnings next to affected skills in `/skill list` output.

## 3. Verification

- [ ] 3.1 Unit tests: matching name has no warning; mismatch warns and still loads under frontmatter name.
- [ ] 3.2 Test `/skill list` output includes the warning.
- [ ] 3.3 Run `openspec validate lint-skill-name-and-folder --strict`.
