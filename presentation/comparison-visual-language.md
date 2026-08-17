# Technical comparison visual language

## Intent

Compare Claude Code, Codex, pi, and jcode as **different runtime shapes**, not as products on a winner's podium. The deck should help the audience answer:

> Which parts of an agent workflow are explicit, composable, inspectable, and durable in each system?

It should not answer “which one is best?” or imply that a single feature matrix is a benchmark.

## Core metaphor: the same task, four control surfaces

Use one neutral workflow as the constant:

`request → context → model turn → tool/action → evidence → continuation`

Each system is shown as a different **control surface** around that workflow. The visual language compares where control lives, how it is exposed, and how far it extends over time.

Do not use bar charts, rankings, stars, medals, green/red winners, or aggregate scores.

## Five comparison lenses

Use these as repeated, non-scoring lenses across slides:

1. **Invocation** — how a run starts and how much setup is visible.
2. **Context assembly** — prompts, files, memory, skills, overlays, and session state.
3. **Action boundary** — built-in tools, MCP, hooks, browser, shell, and external services.
4. **Coordination** — one turn, subprocesses, agents, task graphs, or durable jobs.
5. **Evidence and continuity** — logs, diffs, artifacts, lifecycle events, retries, and follow-up.

Each lens is a question, not a rating. A system can be strong in one lens and intentionally narrow in another.

## Visual primitives

### 1. Workflow rail

A thin horizontal rail with six labeled stations is the base comparison primitive. Place a system's markers above or below the stations to show where behavior is explicit.

- solid connector: native path or clearly documented boundary
- dotted connector: adapter, extension, or integration-dependent path
- faint connector: possible but outside the deck's verified scope
- small `?` marker: not assessed, rather than absent

### 2. Capability envelope

Use nested outlines, not filled territory:

- inner ring: core turn loop
- middle ring: tools and context extensions
- outer ring: lifecycle, coordination, and continuity

The envelope says “surface area represented here,” not “more is better.” Keep all four systems in the same geometric frame.

### 3. Contract cards

For each system, use a compact card with four rows:

- **Primary unit:** turn / session / subprocess / workflow
- **Extension seam:** prompt / tool / hook / plugin / adapter
- **Coordination shape:** single-run / delegated / graph / durable queue
- **Evidence shape:** terminal output / transcript / artifacts / event stream

Cards should contain nouns and short phrases, never adjectives such as “powerful,” “limited,” or “best.”

### 4. State chips

Use small neutral chips to distinguish evidence status:

- `SHIPPED` — observed in the current implementation or official interface
- `COMPATIBLE` — supported through an adapter or documented compatibility surface
- `COMPOSED` — requires combining multiple primitives
- `ROADMAP` — explicitly planned, not current behavior
- `UNASSESSED` — intentionally not claimed

Chip colors indicate epistemic status, not quality. Recommended palette: cyan for shipped, violet for compatible, white outline for composed, amber for roadmap, grey for unassessed.

### 5. Boundary lines

Show boundaries as first-class objects:

- `prompt boundary`
- `tool boundary`
- `process boundary`
- `session boundary`
- `workflow boundary`

A boundary line answers “where can an operator or extension intervene?” It avoids the marketing language of “features.”

## Identity without brand competition

Give every system the same neutral treatment: a small monogram, a restrained tint, and a shared card shape. Never borrow or amplify brand palettes.

- Claude Code: slate + soft coral marker
- Codex: slate + soft green marker
- pi: slate + soft amber marker
- jcode: slate + electric blue-violet marker

Keep all tints below 60% saturation and use them only for identity markers. Structural lines, labels, and evidence chips use the common palette.

## Recommended slide sequence

### 1. The question is about control, not performance

Show the common workflow rail. Caption: “The comparison is about where the runtime makes control explicit.”

### 2. Four runtime shapes

Four cards, same fields, no checkmarks. Use the primary-unit row to establish that the systems organize work differently.

### 3. Context assembly

Show four horizontal prompt/context stacks. Highlight source attribution, layering, and session capture where verified. Avoid implying that a taller stack is superior.

### 4. Action boundaries

Show a shared tool-bus diagram. Each system gets only the boundaries it actually exposes or can reach through an adapter. Mark unknowns as `UNASSESSED`.

### 5. Coordination and time

Use a time axis from “one turn” to “durable workflow.” Place each system's explicit coordination primitives along the axis. This is a topology map, not a maturity ladder.

### 6. Evidence model

Use identical evidence slots: transcript, diff, artifact, event, retry, continuation. Empty slots remain neutral grey and are labeled `not assessed` rather than “missing.”

### 7. jcode as a composition case

Do not compare totals. Animate one jcode workflow by adding boundaries: skill → MCP → hook → memory → swarm → ambient runtime. The claim is that these surfaces compose inside one runtime, not that jcode wins each isolated category.

### 8. Choosing the right lens

End with task archetypes, not a verdict:

- “I want the shortest local loop.”
- “I need a compatible coding-agent surface.”
- “I want a small, inspectable runtime.”
- “I need a repeatable workflow with coordination and continuity.”

Each archetype points to the relevant lens and trade-off questions. No recommendation badge.

## Language rules for narration and labels

Prefer:

- “organizes work around…”
- “makes this boundary explicit…”
- “supports this through…”
- “the verified surface in this deck is…”
- “this requires composition or an adapter…”
- “not assessed here…”

Avoid:

- “beats,” “wins,” “more advanced,” “full-stack,” “best,” “only,” “limited,” “lacks”
- “feature parity” unless the exact scope is defined
- “benchmark,” “score,” “leaderboard,” or “comparison matrix”

## Anti-marketing guardrails

1. Put a scope note on every comparison slide: `Technical surface map · not a benchmark`.
2. Add a source/evidence footer with date and confidence, for example: `Observed interfaces + repository docs · 2026-08 · scope: local runtime behavior`.
3. Keep the same visual area and number of rows for every system.
4. Never let jcode receive a larger card, brighter background, or more favorable adjective. Its differentiation should emerge from the composition diagram.
5. Separate “not present,” “not verified,” and “requires an adapter.” They are different claims.
6. If a slide has a conclusion, make it about a workflow decision or boundary, not a product verdict.

## A compact legend

```text
● native path       ◌ adapter / integration       ? unassessed
━━ explicit seam     ┄┄ indirect seam              ◇ composed surface

SHIPPED   COMPATIBLE   COMPOSED   ROADMAP   UNASSESSED
```

## Visual tone

Retain the existing feature-tour language: dark canvas, Geist Sans/Mono, restrained grey surfaces, electric blue-violet used for jcode's own composition story, and motion that reveals structure rather than spectacle. The new comparison slides should feel like an instrumentation overlay on the current deck, not a sales intermission.
