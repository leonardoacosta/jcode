# Monitor Examples

## Vercel deploy

User request:
`Monitor the dev deploy for tribal-cities on Vercel`

Command:

```bash
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-vercel-deploy tribal-cities dev
```

## GitHub Actions test workflow

`Watch the hosted test workflow on dev for Priceless-Development/tribal-cities`

```bash
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-gh-actions dev Priceless-Development/tribal-cities
```

## Depot CI

`Monitor Depot run dep_123`

```bash
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-depot-ci dep_123
```

## Azure pipeline

`Watch pipeline 42 in storefront under org acme`

```bash
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-azure-pipeline acme storefront 42
```

## Azure classic release

`Monitor release definition 7 for environment 99 in storefront`

```bash
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-azure-release acme storefront 7 99
```
