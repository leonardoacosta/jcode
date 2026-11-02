# Tasks

## 1. Lint

- [x] 1.1 Add a `warnings: Vec<String>` (or equivalent) field to the skill struct in `crates/jcode-base/src/skill.rs`.
- [x] 1.2 On load, compare folder name to frontmatter `name` (namespace-stripped) and record a mismatch warning.
- [x] 1.3 Log each mismatch once per reload.

## 2. Surface

- [x] 2.1 Show warnings next to affected skills in `/skill list` output.

## 3. Verification

- [x] 3.1 Unit tests: matching name has no warning; mismatch warns and still loads under frontmatter name.
- [x] 3.2 Test `/skill list` output includes the warning.
  - Verified 2026-11-02 via `selfdev test`: `scripts/dev_cargo.sh test -p jcode-app-core --lib tool::skill::tests::test_list_name_mismatch_warnings_and_load -- --exact --nocapture` (1 passed, exit 0).
  - Hermetic temp-workspace fixtures use existing test tool/context helpers. Actual `SkillTool::execute` list/load calls verify warnings beside mismatched entries, loading under frontmatter names, case-sensitive comparison, last-colon namespace stripping, matching names, and trimmed folder names.
  - Regression checks via `selfdev test`: `scripts/dev_cargo.sh test -p jcode-base --lib skill::tests::folder_name_mismatch_warns_but_loads -- --exact` (1 passed), then `scripts/dev_cargo.sh test -p jcode-app-core --lib tool::skill::tests:: -- --skip tool::skill::tests::test_list_empty` (14 passed). Both exited 0. Unrelated prompt identity assertion was not run or changed.
- [x] 3.3 Run `openspec validate lint-skill-name-and-folder --strict`.
