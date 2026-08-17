# Refined agent-browser skills 0.27.3

This directory preserves locally refined copies of every skill enumerated by
`agent-browser skills list --json` in agent-browser 0.27.3:

- `agentcore`
- `core`
- `dogfood`
- `electron`
- `slack`
- `vercel-sandbox`

Each skill was independently evaluated and edited using the `/skill-judge`
8-dimension, 120-point rubric. The corresponding before/after reports are in
`reports/`.

The live installed copies are under the mise agent-browser installation's
`skill-data/` directory. A future agent-browser update may replace those live
files, so this versioned directory is the durable source for reapplying or
upstreaming the refinements.

## Restore to a matching installation

First verify the installed version is exactly `0.27.3`, then copy the desired
skill directory into the path returned by `agent-browser skills path <name>`.
Do not apply these files blindly to another version because command syntax and
bundled references may have changed.
