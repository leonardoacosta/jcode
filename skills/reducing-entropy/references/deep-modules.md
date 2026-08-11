---
description: Shared vocabulary for interface depth (Ousterhout) — deep vs shallow modules, seam discipline, the deletion test, and dependency-category test strategy. Load when grading a "shallow wrapper" finding or debating whether an interface earns its keep.
---

# Deep Modules: A Vocabulary for Interface Depth

## The Core Insight

> A module's value is the functionality it provides divided by the size of its interface. A
> **deep** module hides substantial complexity behind a small interface — high leverage. A
> **shallow** module's interface is about as complex as what it does — the indirection barely
> pays for itself.

This gives `reducing-entropy`'s deletion bias a second axis. The skill already asks "is there a
current caller?" — deep-module vocabulary answers the harder follow-up: "given a caller exists,
is this abstraction pulling its weight?"

## Glossary

Each term has one meaning here. The banned substitute is not wrong English — it's a word people
reach for that erases the distinction this vocabulary exists to preserve.

| Term | Definition | Banned substitute | Why it's banned |
| --- | --- | --- | --- |
| Module | A unit with an interface and a hidden implementation — a function, class, package, or service. | "component" | Says nothing about depth; a component can be deep or shallow. |
| Interface | Everything a caller must know to use the module: signature, contract, side effects, error modes. | "API" | Used too loosely across layers — an interface is the specific thing being measured. |
| Implementation | Everything hidden behind the interface — the reason a deep module is worth keeping. | "internals" / "guts" | Fine in conversation, imprecise in a design review where the interface/implementation split is the point. |
| Depth (deep / shallow) | Functionality provided relative to interface size. Deep = small interface hiding real complexity. Shallow = interface about as complex as the thing it does. | "big class" / "small class" | Measures code size, not leverage — a big class can still be shallow. |
| Seam | A point where one implementation can be substituted for another without changing callers. | "wrapper" | Says nothing about whether the seam is real — see Seam Discipline below. |
| Adapter | A concrete implementation living behind a seam. | "layer" | Implies stacking (adding on top); an adapter substitutes, it doesn't stack. |
| Leverage | Complexity hidden divided by interface complexity exposed — a useful depth metric, in place of Ousterhout's line-count ratio (line count conflates depth with verbosity). | "abstraction level" | Too vague to grade against; leverage is a ratio you can actually argue about. |
| Locality | How much of a change stays inside one module versus rippling out to callers. | "coupling" | Coupling covers too many failure modes; locality is specifically about change-containment. |

## The Deletion Test (an interface-design probe)

`reducing-entropy`'s existing decision tree asks whether a caller exists. The deletion test is
the next question, asked of code that already has one:

**If you deleted this module/class/function and inlined its guts at every call site, would the
code get simpler or messier?**

- **Simpler** → the module was shallow: low leverage, the interface wasn't hiding enough to
  justify the indirection. Inline it.
- **Messier** → the module was deep: worth keeping, because the interface is genuinely hiding
  complexity the caller shouldn't have to hold in their head.

This is the same test the SKILL.md decision tree already runs at the "single call site → inline
it" branch — the deletion test just makes the criterion explicit instead of leaving "is this
appropriate" to intuition.

## Seam Discipline

**One adapter behind a seam is a hypothetical seam. Two or more concrete implementations is a
real seam.**

A seam introduced for a single concrete implementation is speculative — the same YAGNI territory
`reducing-entropy`'s anti-patterns already flag ("NEVER create a utility function without a
concrete current caller"). A seam with two or more real implementations already in play earns
its keep: the interface is doing real substitution work, not just adding a hop.

Distinguish two kinds of seam by what's on the other side:

- **Internal seam** — a module boundary within one process (a service interface with two
  in-memory implementations, a strategy interface with two concrete strategies).
- **External seam** — a boundary at the edge of the process (network call, DB, filesystem).
  External seams are held to a lower bar for "two implementations" — a single production
  adapter plus a test double already counts, because the alternative (no seam at all) means
  every test hits the real network/DB/filesystem.

## Dependency Category → Test Strategy

How a module's dependency should be tested depends on what kind of dependency it is — not on a
blanket "mock everything" or "mock nothing" rule.

| Category | What it looks like | Test strategy |
| --- | --- | --- |
| In-process | Same-process pure logic, no I/O. | Test directly — no substitution needed, call the real function. |
| Local-substitutable | A real external system with a fast local equivalent (in-memory queue instead of Redis, SQLite/PGLite instead of Postgres). | Substitute the real thing with the local equivalent; test against that. |
| Ports & adapters | A real external system with no fast local equivalent, but a stable interface. | Test via a test double implementing the same interface — the seam earns its keep here. |
| True-external | A boundary you cannot substitute locally (payment gateway, third-party API with side effects). | Mock the wire protocol itself, sparingly — this is the last resort, not the default. |

## Replace-Don't-Layer Testing

When a module's interface needs to change, prefer replacing the old implementation outright over
adding a compatibility shim or a translation layer on top of it. Apply clean replacement when its compatibility cost would exceed the value of the old interface. If a change
would make you write an adapter whose only job is to make the new interface look like the old
one, that adapter is itself a hypothetical seam per the discipline above: stop and ask whether a
clean replacement is acceptable before building it.

**When to load this file:** when discussing module depth, interface shape, or grading a "shallow
wrapper" finding — not during a routine dead-code sweep (that's the base `reducing-entropy`
decision tree, no depth vocabulary needed).

Source material is MIT-licensed (`mattpocock/skills`, `improve-codebase-architecture`); this guidance is a portable rewrite, not a verbatim copy.
