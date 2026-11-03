# Skills UX sessions
Primary: puppy `session_puppy_1793295984363_8303f21a3f94ce1a` (2026-10-31 20:43, "Does jcode really not scan repo level .agents for skills?") → spawned seedling/giraffe/hamster.

| Ask | Session | Status | Evidence |
|---|---|---|---|
| Repo-level .agents skill scan | puppy 10-31 | Done | 210ebfbe6 (ancestor dirs + nested .agents/skills); skill.rs:291,352 scans .jcode/.agents/.claude |
| Impl project discovery in skill.rs load_from_dir | seedling 10-31 | Done | 210ebfbe6 |
| folder==name (leo-voice) | giraffe 10-31 | Open in jcode | no folder/name mismatch warning in skill.rs |
| cfg-inventory differs review | hamster 10-31 | Read-only report | no commit |
| Active-skill display | earlier | Done | 67de129ba (10-18) |
| ACP /skill resolution | earlier | Done | 0c511ab29 |
| skill_manage tool | n/a | Exists | crates/jcode-app-core/src/tool/skill.rs |

Gaps: 1 folder≠name lint; 2 no dedicated "skills UX" ask found beyond puppy chain (pig 11-01 is current search).
