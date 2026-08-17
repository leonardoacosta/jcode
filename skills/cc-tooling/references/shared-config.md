# Shared Config Architecture

> Reusable configuration structure for all projects

## Directory Structure

```
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/shared/
├── eslint-config/             # ESLint flat config
│   ├── base.js
│   ├── nextjs.js
│   └── oxlintrc.json
└── templates/                 # Project scaffolding templates
```

## Global Directories (not in shared/)

- `rules/` - Standards documentation (SHARED.md, etc.)
- `agents/` - Agent definitions (29 active + 3 archived)
- `skills/` - Skill library with search index
- `commands/` - Slash commands

## Project Overrides

Each project's `.claude/` directory can:

- Override shared files by creating same-named files locally
- Add project-specific configuration in `CLAUDE.md`
- Extend `settings.json` hooks with project-specific logic
- Define custom agents/commands

## Machine-Local Overrides

The `.claude/local/` directory (gitignored) allows:

- Developer-specific overrides without affecting team
- Local API keys, tokens, paths
- Experimental configuration

## Shared Config Legend

| Status      | Meaning                                                              |
| ----------- | ---------------------------------------------------------------------- |
| OK          | Has symlinks to `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/shared/` (rules, agents, skills, helpers) |
| OK (source) | Original source project for shared configuration                     |
| —           | Not yet configured with shared symlinks                              |

See Project Registry in CLAUDE.md for per-project shared config status.
