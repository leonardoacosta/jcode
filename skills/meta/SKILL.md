---
name: meta
description: Claude Code meta-repo automation (meta) — skills, commands, agents, openspec
category: Meta
level: framework
engineer: general-purpose
gate: "bash -n scripts/**/*.sh && openspec validate --strict"
bundles: []
allowed-tools: Read, Glob, Grep, Bash
---

# Meta

Composition metadata carrier — frontmatter is the product (see
`commands/apply/references/skill-metadata-schema.md`). Body intentionally minimal. Do not
delete: `compose_stack` hard-fails without this registration.
