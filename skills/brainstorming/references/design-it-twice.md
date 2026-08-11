---
description: Parallel divergent-constraint design protocol for a contested interface shape — spawn 3+ agents pinned to different design constraints instead of proposing alternatives sequentially in one context. Load at the Design Gate when the interface shape itself is the open question.
---


# Design It Twice: Parallel Divergent-Constraint Design

## When to Invoke

The base brainstorming flow already proposes 2-3 approaches (Key Principles § Explore
alternatives) — but it does so sequentially, in one context, one train of thought. That anchors
every alternative on whichever idea came first. This protocol is an escalation, not a
replacement: reach for it only when the **interface shape itself** is contested, at the Design
Gate (`rules/CORE.md` § Design Gate), and the choice has lasting cost — a public API, a package
boundary, or a seam in the sense `references/deep-modules.md` uses the word (a point where one
implementation could substitute for another).

Signals this applies:

- Two or more genuinely different interface shapes are plausible, not just two implementations
  of the same shape.
- The decision is expensive to reverse (callers will be written against it, a package boundary
  will form around it, a migration would be needed to change course later).
- Sequential brainstorming would obviously bias toward whichever shape you thought of first.

Not every design question needs this. A single-caller utility, an internal helper, or a shape
with one obvious answer stays on the base flow — spinning up three parallel agents to design a
function with one call site is the same YAGNI mistake `deep-modules.md`'s Seam Discipline warns
against (a seam built for a hypothetical, not a real, need).

## Step 1 — Frame the Constraint Space for the User

Before spawning anything, name the real constraints out loud and get the user's reaction. This
is cheap and happens before any agent burns tokens on the wrong axis:

- **Backward-compat need** — is a clean replacement on the table (`rules/CORE.md` § Breaking
  Changes), or must existing callers keep working?
- **Performance ceiling** — is there a latency/throughput bar this interface must clear?
- **Who maintains it** — a solo maintainer favors the smallest interface; a team favors more
  explicit extension points.
- **Dependency category** — which row of `deep-modules.md`'s dependency-category table does the
  thing behind this interface fall into (in-process / local-substitutable / ports & adapters /
  true-external)? That answer shapes what "the seam" even looks like before a single design is
  drafted.

Sketch illustratively — not exhaustively — what a "minimal" version and a "maximal" version
might look like, in a couple of sentences each. The point is to let the user correct the axis
("no, we don't need multi-provider support") before agents design against the wrong constraint,
not to fully specify either extreme.

## Step 2 — Spawn 3+ Parallel Agents, One Constraint Each

Dispatch in a single message so the agents run concurrently and never see each other's output —
that isolation is the entire point; an agent that can read a sibling's draft will anchor on it
exactly like sequential brainstorming does. Pin each agent to exactly one constraint:

| Constraint | Design backwards from |
| --- | --- |
| Minimize interface | Fewest public methods/params. Hide as much as possible behind the seam. |
| Maximize flexibility | Most configurable. Most extension points. Assume future requirements you can't see yet. |
| Optimize the common caller | The single most frequent call site. Everything else is secondary and may cost more to use. |
| Ports & adapters | Assume multiple real implementations from day one. Design the seam first, the implementation second. |

Each agent returns, for its one design:

1. **The interface itself** — the actual signature/shape, not a description of one.
2. **A usage example** — from the caller's side, showing what calling it looks like in context.
3. **What hides behind the seam** — the "deep" part per `deep-modules.md`: what complexity this
   interface is absorbing so the caller doesn't have to hold it in their head.
4. **Dependency category + test strategy** — which row of the dependency-category table it falls
   into, and what that implies for how it gets tested.
5. **Trade-offs** — what this shape makes harder, explicitly. Every design costs something;
   name it instead of leaving it for the comparison step to discover.

## Step 3 — Compare and Recommend

Compare the four returned designs on:

- **Depth** — run the deletion test on each (`deep-modules.md` § The Deletion Test): if you
  inlined this interface at every call site, does the code get simpler (shallow, don't keep it
  in this shape) or messier (deep, this shape is pulling real weight)?
- **Locality** — does the caller need to know about things it shouldn't? A design that leaks
  implementation detail into the caller loses on locality even if its interface looks small.
- **Seam placement** — is this a real seam (two or more concrete implementations already in
  play) or a hypothetical one (`deep-modules.md` § Seam Discipline)? A "ports & adapters" design
  built for a single implementation is a hypothetical seam wearing a real-seam costume.

Then give **one opinionated recommendation** — a straight pick, or a named hybrid if the best
answer borrows from two candidates — per `feedback_recommend_and_defend`: recommendation first,
trade-offs per option, defend the pick. Do not hand the user all four outputs as an
undifferentiated menu; relaying every design with no synthesis defeats the purpose of running
this protocol at all — the value is in the comparison, not in generating four options.

---

Source material is MIT-licensed (`mattpocock/skills` `improve-codebase-architecture`; recon
record at `docs/recon/mattpocock-improve-codebase-architecture.md`). Content here is rewritten in
the corpus's own idiom, not copied verbatim.
