
# ai-media-gen

> Formerly a standalone skill (`ai-media-gen`), demoted to a `frontend-design` reference
> (`skill-classification-and-trial-lifecycle`, 2026-07-18) — its weakest dimension was
> CLI-wrapper procedures with no domain mindset beyond the wrapper contract. Generate raster
> images and video from prompts via the `ai` CLI (Vercel AI Gateway). Use when you need a real
> raster image or video — hero banners, conceptual illustrations, social cards, mockup imagery,
> or any pixel/motion asset that mermaid, `wayfinder` HTML, excalidraw, or `ascii-wireframe`
> (all vector) cannot produce.

The portable skill corpus generates only *vector* visuals (mermaid, `wayfinder` HTML, excalidraw MCP,
`ascii-wireframe`). This skill closes the **raster + motion** gap by wrapping the `ai` CLI
(`ai-cli`, npm bin `ai`, v0.3.0) behind a repository-aware wrapper: **`scripts/bin/ai-media`**.

**Always call `scripts/bin/ai-media`, not raw `ai`.** The wrapper injects a default output path,
forces `--json`, and maps results to a three-state exit code — removing the non-TTY
stdout-binary footgun by construction.

## When to Use

- A raster hero banner or conceptual illustration for a `wayfinder` / `frontend-design` page
- A photo/picture/render an agent or user explicitly asks for
- Video from a prompt or from a still image
- Side-by-side comparison of one prompt across multiple models
- A composable media pipeline (image → video, image edit via stdin)

Do NOT use for diagrams, charts, schemas, flowcharts, or tables — those are vector work owned by
`wayfinder` (HTML/mermaid), `mermaid-diagrams`, or `ascii-wireframe`.

## Prerequisites

`AI_GATEWAY_API_KEY` must be set. The canonical location is `~/.env` (your `~/.zshrc` auto-exports
it via `set -a; source ~/.env; set +a`). If it is unset the wrapper exits `3` with guidance.
Verify gateway auth at any time with `scripts/bin/ai-media models --type image`.

**Free tier vs paid (image/video).** The CLI's default image model `openai/gpt-image-2` requires
**paid** AI Gateway credits and errors on the free tier. Verified free-tier-accessible image model:
**`bfl/flux-2-flex`** (`-m bfl/flux-2-flex`, ~9-11s). `ai text` runs free (`openai/gpt-5.5`). For
image/video on free tier, always pass `-m` with an accessible model or set `AI_CLI_IMAGE_MODEL` /
`AI_CLI_VIDEO_MODEL`. Check access per-model with `scripts/bin/ai-media models --type image`.

## Commands

```bash
scripts/bin/ai-media image "a sunset over matte-black mountains"   # generate an image
scripts/bin/ai-media video "a slow drone shot of a canyon"         # generate a video
scripts/bin/ai-media text  "summarize this" < notes.txt            # generate text
scripts/bin/ai-media models --type image                           # list image models
```

## Key Flags (passed through to `ai`)

```
-m, --model <id>     Model id (creator/name or short), comma-separated for multi-model
-o, --output <path>  Output file or dir. Wrapper defaults to docs/diagrams/assets/ if omitted.
-n, --count <n>      Generations per model
-i, --image <path>   Reference image (image cmd) / vision input (text cmd), repeatable
--size <WxH>         Image size      --aspect-ratio <W:H>   Aspect ratio
```

## Output Behavior (important for agents)

The wrapper **always** writes to a file and prints `--json` to stdout. Read the artifact path
from `results[].file` — never expect raw binary. Without an explicit `-o`, output lands in
`${AI_CLI_OUTPUT_DIR:-docs/diagrams/assets}/`.

```json
{ "elapsed_ms": 3420, "count": 1,
  "results": [ { "index": 1, "model": "openai/gpt-image-2", "success": true, "file": ".../output.png" } ] }
```

## Exit Codes

| Code | Class | Meaning |
|------|-------|---------|
| `0` | generation | all results succeeded |
| `1` | generation | all failed, or `ai` errored / emitted non-JSON |
| `2` | generation | partial — some succeeded, some failed |
| `3` | config | `AI_GATEWAY_API_KEY` unset (preflight, before any call) |

Branch on `2` to handle partial multi-model/multi-count runs distinctly from total failure.

## Piping Patterns

```bash
# Summarize piped content (text)
git diff | scripts/bin/ai-media text "write a commit message"

# Image → video pipeline
scripts/bin/ai-media image "a dragon" -o /tmp/d.png && \
  scripts/bin/ai-media video "animate this" -i /tmp/d.png
```

## Embedding into a wayfinder / frontend-design page

Generate, base64-encode, inline for self-containment:

```bash
scripts/bin/ai-media image "isometric matte-navy server rack, single cyan accent" \
  -o /tmp/hero.png --aspect-ratio 16:9
IMG=$(base64 -w 0 /tmp/hero.png)          # Linux; macOS: base64 -i /tmp/hero.png
# <img src="data:image/png;base64,${IMG}" alt="...">
rm /tmp/hero.png
```

Match the prompt to the page palette and aesthetic direction (style + dominant colors). Specific
prompts beat vague ones — "isometric illustration of a message queue, cyan nodes on dark navy"
beats "a diagram of a queue".

## NEVER

- **NEVER call the raw `ai` CLI directly.** Always go through `scripts/bin/ai-media` — the wrapper
  forces `--json` and normalizes exit codes; raw `ai` can print binary or human-formatted output
  to stdout depending on TTY detection, and its exit codes don't carry the partial-failure (`2`)
  distinction the wrapper adds.
- **NEVER expect raw binary on stdout.** The wrapper always writes the artifact to a file and
  prints `--json` to stdout — read the path from `results[].file`. Piping stdout to an image
  viewer or `> out.png` captures JSON, not pixels.
- **NEVER use this skill for diagrams, charts, schemas, flowcharts, or tables.** That's vector
  work owned by `wayfinder` (HTML/mermaid), `mermaid-diagrams`, or `ascii-wireframe` —
  raster models cannot produce crisp lines/text at arbitrary zoom the way vector output can.
- **NEVER assume the default image/video model works on the free tier.** `openai/gpt-image-2`
  (the CLI default) requires paid AI Gateway credits and errors on the free tier. Pass `-m
  bfl/flux-2-flex` (image) or set `AI_CLI_IMAGE_MODEL`/`AI_CLI_VIDEO_MODEL`, or check first with
  `scripts/bin/ai-media models --type image`.
- **NEVER skip the `AI_GATEWAY_API_KEY` preflight.** A missing key exits `3` before any generation
  call — don't retry the same command expecting a transient failure; fix the env var first.
- **NEVER treat exit code `2` as success or as total failure.** It means partial completion on a
  multi-model/multi-count run — inspect `results[]` for which entries have `success: false` and
  decide whether to retry just those, not the whole batch.
- **NEVER leave the temp file behind after inlining into HTML.** The base64-embed recipe copies
  the file's bytes into the page; the on-disk artifact (`/tmp/hero.png`) serves no further purpose
  and MUST be `rm`'d — it is not the deliverable, the inlined `<img>` tag is.

## Cross-References

| Need | Skill |
|------|-------|
| The page the image goes into | `wayfinder`, `frontend-design` |
| Vector diagrams (do NOT use this skill) | `mermaid-diagrams`, `ascii-wireframe` |
| Brand logos (not generated) | `~/.claude/skills/frontend-design/references/icon-sourcing.md` |

Source / adoption verdict: `docs/recon/vercel-labs-ai-cli.{md,html}`.
