## ADDED Requirements

### Requirement: Mismatch warning without blocking
The skill loader SHALL record a warning when a skill's folder name differs from its frontmatter `name`, and SHALL still load the skill under its frontmatter name.

#### Scenario: Mismatched folder
- **WHEN** `skills/foo/SKILL.md` declares `name: bar`
- **THEN** skill `bar` is loaded and carries a warning naming folder `foo`

#### Scenario: Matching folder
- **WHEN** `skills/foo/SKILL.md` declares `name: foo`
- **THEN** no warning is recorded

### Requirement: Warning visible in skill list
The `/skill list` output SHALL display each recorded name-mismatch warning beside its skill.

#### Scenario: List shows warning
- **WHEN** the user runs `/skill list` and skill `bar` has a mismatch warning
- **THEN** the `bar` entry shows the warning text
