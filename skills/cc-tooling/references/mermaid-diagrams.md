# Mermaid Diagrams (harness binding)

Portable — see `agent-tooling` skill's `references/mermaid-diagrams.md` for `mmdc` usage,
flowchart syntax rules (`<br/>` vs `\n`, subgraph comment restrictions, special characters), the
`stateDiagram-v2` label caveat, and the Mermaid-vs-alternative diagram-type table. This file
supplies only the one thing that's genuinely local to this box: the currently-installed
`chrome-headless-shell` path `mmdc` needs.

## This box's `PUPPETEER_EXECUTABLE_PATH`

```bash
PUPPETEER_EXECUTABLE_PATH=~/.cache/puppeteer/chrome-headless-shell/linux-146.0.7680.31/chrome-headless-shell-linux64/chrome-headless-shell \
  mmdc -i input.mmd -o output.svg -b transparent
```

Shell globs (`linux-*`) do NOT expand inside env var assignments — resolve the exact version dir
every time `chrome-headless-shell` bumps:

```bash
npx puppeteer browsers install chrome-headless-shell
ls ~/.cache/puppeteer/chrome-headless-shell/   # confirm the new version dir, update the path above
```

## File Placement

Diagram source files: `docs/*.mmd`. Rendered output: `docs/*.svg` or `docs/*.png`, committed
beside the source.
