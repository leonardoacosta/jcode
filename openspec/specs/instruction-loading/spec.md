# instruction-loading Specification

## Purpose
Load shared agent instructions deterministically into the prompt and expose instruction provenance without duplicate equivalent files.

## Requirements

### Requirement: Shared instruction file loads natively

The prompt builder MUST load `~/.agents/AGENTS.md` when present, ordered before global `~/AGENTS.md` and project instructions, without requiring a model tool call.

#### Scenario: Shared file present
- **WHEN** `~/.agents/AGENTS.md` exists and is readable
- **THEN** its content appears in the system prompt under a "Shared Instructions" header before global and project layers

#### Scenario: Shared file missing or unreadable
- **WHEN** the file is absent or unreadable
- **THEN** prompt building succeeds without that layer and records status `skipped-missing` or `unreadable`

### Requirement: Paired instruction files are deduplicated

For each directory the loader MUST prefer `AGENTS.md`, include `CLAUDE.md` only when it adds distinct content, and MUST NOT include any file twice by canonical path or normalized content.

#### Scenario: Only CLAUDE.md present
- **WHEN** a project has `CLAUDE.md` and no `AGENTS.md`
- **THEN** `CLAUDE.md` is loaded as project instructions

#### Scenario: Identical or pointer pair
- **WHEN** `CLAUDE.md` is a symlink to, identical to, or only references `AGENTS.md`
- **THEN** only `AGENTS.md` is loaded and `CLAUDE.md` is recorded `skipped-duplicate`

#### Scenario: Cross-layer duplicate
- **WHEN** the working directory is `$HOME` so project and global `AGENTS.md` are the same file
- **THEN** the content is included once

### Requirement: Effective instructions are inspectable

Users MUST be able to list every candidate instruction file with layer, path, status, and size, in prompt order.

#### Scenario: View in TUI
- **WHEN** the user runs `/context instructions`
- **THEN** the TUI lists each candidate with its status, one row per file, truncated rather than wrapped

#### Scenario: View from CLI
- **WHEN** the user runs `jcode debug instructions --dir <path>`
- **THEN** the same listing prints for that directory and the command exits 0

#### Scenario: Invalid directory
- **WHEN** `--dir` names a path that does not exist
- **THEN** the command exits non-zero with an error naming the path
