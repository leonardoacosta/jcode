# Proposal

## Why

Skills are discovered as `<folder>/SKILL.md` (`crates/jcode-base/src/skill.rs:388`, `:537`) and identified by the frontmatter `name`. When the folder and frontmatter name differ (renames, copied skills), the skill loads under a name the user does not see on disk, `/skill` lookups by folder fail, and duplicates go unnoticed. Nothing reports the mismatch. Clean-room reference (`scratch/empryo-re/adjacent-behavior.md`, skills section) lints this as a warning without blocking load.

## What Changes

- During skill discovery, compare the parent folder name to the frontmatter `name`; on mismatch record a warning on the loaded skill.
- Still load the skill under its frontmatter name.
- Show the warning beside the skill in `/skill list` (`crates/jcode-app-core/src/tool/skill.rs`) and log it once.

## Capabilities

### New Capabilities

- `skill-name-lint`: Non-blocking folder/frontmatter name mismatch warnings.

## Impact

`crates/jcode-base/src/skill.rs`, `crates/jcode-app-core/src/tool/skill.rs`. No overlap with `native-instruction-file-loading`, which covers AGENTS.md files, not skills.

## Open decisions

1. Comparison strictness. Recommended: exact match after trimming; case differences still warn.
2. Plugin-namespaced skills (`plugin:name`). Recommended: compare only the segment after the last `:`.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
