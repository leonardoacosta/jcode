# Domain-Modeling Convention: `CONTEXT.md` + `docs/adr/`

> Distinct from § Repo `docs/` Tree Conventions in the main skill body. That table governs
> `docs/{reference,notes,guides,diagrams}/` — how-it-IS state, investigation journals, runbooks,
> visual explainers. This file governs two adjacent artifacts that sit **outside** that tree:
> a repo-root ubiquitous-language glossary and a decision-record log. Different question
> ("what does this word mean here", "why did we reject that path") than the docs canon answers
> ("what is the current state of this subsystem").

## The two artifacts

| Artifact | Location | Answers |
| --- | --- | --- |
| `CONTEXT.md` | Repo root (not `docs/`) | "What does term X mean in this codebase, and what code entity does it map to?" |
| `CONTEXT-MAP.md` | Repo root, alongside `CONTEXT.md` | "How do this repo's bounded contexts relate to each other?" (multi-context repos only) |
| `docs/adr/NNNN-title.md` | `docs/adr/`, one file per decision | "Why did we reject the obvious alternative, and is it safe to re-propose?" |

`CONTEXT.md` holds the domain's ubiquitous language — terms specific to *this repo's business
domain*, not generic engineering vocabulary already covered elsewhere (`deep-modules.md` owns
module/interface/seam/depth; this file owns *this repo's* nouns — "badge", "volunteer",
"reconciliation window", whatever the domain actually calls its things). One entry per term:
name, one-line definition, and — where it sharpens rather than pads — the code entity it maps
to (a type, a table, a router). When more than one word competes for the same concept, add an
`_Avoid_:` field naming the rejected synonym(s), so the same terminology-drift question doesn't
resurface unrecorded in a later session:

```
**Order**:
{one-line definition}
_Avoid_: Purchase, transaction
```

A repo spanning more than one bounded context gets a `CONTEXT-MAP.md` beside it describing how
the contexts relate. Two ways to phrase a relationship, depending on which reads clearer for the
pair in question:

- DDD-jargon shorthand — shared kernel, customer/supplier, anti-corruption layer — when the
  relationship is structural/ownership-shaped.
- Concrete event-flow phrasing — e.g. "Ordering emits `OrderPlaced` events; Fulfillment consumes
  them to start picking." — when the relationship is a runtime data/event flow the jargon would
  obscure rather than clarify.

Keep it light either way — a pointer between contexts, not a DDD textbook chapter.

`docs/adr/` holds one file per architecture decision, standard ADR shape: title, status,
context, decision, consequences. This is a well-known convention — don't over-specify the
format beyond that shape.

## Lazy creation — neither file is scaffolded up front

Do not pre-populate `CONTEXT.md`, `CONTEXT-MAP.md`, or `docs/adr/` speculatively. Each is
created the first time it earns its keep:

- `CONTEXT.md` — created the first time a domain term actually needs sharpening: someone uses a
  word two different ways in the same conversation, or a new contributor asks "what does X
  mean here?" A repo with zero terminology friction carries no `CONTEXT.md`, and that's correct,
  not a gap.
- An ADR — created the first time a **rejected** design alternative is load-bearing enough that
  a future explorer would otherwise waste real time re-deriving or re-proposing it (see § below).

A speculative glossary entry or ADR written before either trigger fires is exactly the kind of
padding the docs canon's "age is not decay" principle warns against in the other direction —
manufactured content nobody asked for, decaying from day one because nothing keeps it honest.

## Glossary-challenge / term-sharpening protocol

Apply the same kind of question `deep-modules.md`'s deletion test asks of an interface, aimed
at a term instead of a module boundary:

> Could two people read this definition and walk away pointing at different code?

If yes, the definition isn't done — sharpen it until they can't. A one-line definition that
still lets two readers disagree on which entity it names is worse than no entry, because it
looks authoritative while being useless as a tie-breaker.

Glossary entries drift. Periodically — or opportunistically, whenever other work already has you
looking at the code a term describes — cross-reference the entry against the actual code it
claims to map to. A `CONTEXT.md` entry that no longer matches the code is worse than an absent
one: it actively misdirects the next reader instead of leaving them to ask. Treat a drifted entry
as decay to fix, not an age artifact to leave alone (same distinction the docs canon draws for
`docs/reference/`).

## Active Elicitation

The glossary-challenge protocol above defines the test ("could two people read this and point
at different code?"). This section is the live-session mechanic for applying it — five behaviors
docs-engineer runs *during* the conversation it's already having, not a separate interview
script scheduled for later:

1. **Challenge against the glossary.** When a term the user (or the code) uses conflicts with an
   existing `CONTEXT.md` entry, say so immediately, in the moment — don't silently pick a
   reading and move on. "Your glossary defines `Order` as A, but you seem to mean B here — which
   is it?"
2. **Sharpen fuzzy language.** When a term is vague or overloaded — used differently by
   different people, or used one way in conversation and another in the code — propose a precise
   canonical replacement rather than recording the vague version as-is.
3. **Discuss concrete scenarios.** Stress-test a stated domain relationship with specific edge
   cases ("what happens when an Order has zero line items — is that still an Order, or something
   else?") — concrete scenarios force precision about a boundary that abstract descriptions let
   slide.
4. **Cross-reference with code.** When stated behavior doesn't match what the code actually
   does, surface the contradiction rather than silently trusting either source — neither the
   human's description nor the code's current state is automatically authoritative; the mismatch
   itself is the finding worth recording.
5. **Update `CONTEXT.md` inline, immediately.** Once a term resolves, write the entry into
   `CONTEXT.md` in the same turn — never batch term resolutions for a later pass. A resolution
   held in conversation state and not yet written is one context-compaction away from being lost,
   and the next session re-derives it from scratch.

## ADR-on-rejection rule

Don't record every rejected idea — most rejections are ephemeral or self-evident ("we didn't use
jQuery" needs no ADR in 2026) and recording them is pure ceremony. Write an ADR only when the
rejection is **load-bearing**: a future explorer, with no memory of this conversation, would
plausibly re-propose the same rejected approach and burn real time re-discovering why it doesn't
work. The bar is concrete, not a feeling — ask: would a competent engineer, six months from now,
looking only at the code and no chat history, plausibly re-propose this? If yes, write the ADR.
If the rejection is obvious from the code itself, or from a constraint that will still be
obviously true in six months, skip it.

## Relationship to cc's own lanes (read before applying this to cc itself)

cc already runs two mechanisms that look similar to `CONTEXT.md`/ADRs but serve a narrower,
different scope — this convention does **not** replace either, and does not apply to cc's own
meta-repo the way it applies to a fleet/project repo:

- The `plans/` advisory spine's `settled` list is a **per-audit-run** decision record, scoped to
  one advisory pass (an `improve:*` lens run) — it exists so a later pass doesn't re-report a
  finding Leo already rejected in an earlier one. It is not a durable, cross-run glossary.
- Harness auto-memory (`~/.claude/projects/.../memory/`) is **cc-session-scoped** — it persists
  across sessions but lives outside the repo entirely, is never committed, and captures working
  context (what Leo is mid-way through, what a prior session learned) rather than a repo's
  domain vocabulary.

This convention is for a **fleet or project repo's own domain** — its business vocabulary
(`CONTEXT.md`) and its architecture decisions (`docs/adr/`), committed alongside that repo's
code so a fresh agent or a new contributor can read them without any cc session history. Do not
double-record: a decision that belongs in a project repo's `docs/adr/` should not also be copied
into cc's `plans/settled` list or into auto-memory, and vice versa — cc's own operational
decisions (about cc itself) belong in cc's `plans/`/memory lanes, not in a `CONTEXT.md`/ADR pair,
because cc has no downstream fresh-agent reader that a repo-committed convention is solving for.
