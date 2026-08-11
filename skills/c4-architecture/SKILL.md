---
name: c4-architecture
description: Generate architecture documentation using C4 model Mermaid diagrams. Use when asked to create architecture diagrams, document system architecture, visualize software structure, create C4 diagrams, or generate context/container/component/deployment diagrams. Triggers include "architecture diagram", "C4 diagram", "system context", "container diagram", "component diagram", "deployment diagram", "document architecture", "visualize architecture".
source: ~/.agents/skills@2026-07-13
---


# C4 Architecture Documentation

Generate software architecture documentation using C4 model diagrams in Mermaid syntax.

## Workflow

1. **Understand scope** - Determine which C4 level(s) are needed based on audience
2. **Analyze codebase** - Explore the system to identify components, containers, and relationships
3. **Generate diagrams** - Create Mermaid C4 diagrams at appropriate abstraction levels
4. **Document** - Write diagrams to markdown files with explanatory context

## Decision Framework: C4 vs a Simpler Diagram

C4 solves one specific problem: communicating a system's structure across **audience-tiered
zoom levels** (exec summary -> architect -> developer) with **consistent notation between
levels**. That's narrower than "I need to draw a diagram" — most diagram requests aren't that,
and reaching for C4 anyway means paying its notation overhead (Person/System/Container/Component
vocabulary, boundary syntax, legend) for a problem it wasn't built to solve.

| You're documenting... | Reach for | Why C4 is the wrong (or right) fit |
|---|---|---|
| A single request/response flow, an auth sequence, one API call chain | Mermaid `sequenceDiagram` | C4Dynamic covers this too, but a plain sequence diagram needs no abstraction-level scaffolding and renders in any markdown viewer without the C4 mental model |
| A decision tree, business process, algorithm, CI/CD pipeline | Mermaid `flowchart` | No systems/containers to model — C4's Person/System/Container vocabulary doesn't map onto a linear process |
| Domain model, entity relationships, OOP class structure | `classDiagram` / ERD | C4 documents runtime structure, not data/domain shape — see `documentation-writer`'s mermaid reference for these diagram types |
| Multiple systems/actors AND genuinely different audiences need different detail (exec vs architect vs developer) | C4 (Context + Container minimum) | This is the actual problem C4 solves — nothing else gives consistent notation across zoom levels for the same system |
| One service's internal call graph, no external actors, no multi-audience split | Plain flowchart, or skip the diagram | C4Component is explicitly "only if adds value" (see Level 3 below) — a single-container system rarely earns Component-level C4 |
| A diagram approaching 20+ elements at one level | Split by bounded context/domain into multiple focused diagrams, or drop to a coarser level | C4's whole model is zoom levels for exactly this reason — cramming detail into one diagram instead of using a coarser level throws away the abstraction-tier benefit this table is about |

**Overkill signal:** you're about to draw a C4Context diagram with exactly one System box and one
Person box, with no Container/Deployment diagram planned to follow it. One isolated level gets
none of C4's actual benefit (cross-level consistency) and pays the full notation cost — use a
plain flowchart instead.

**Under-kill signal (should have used C4):** you're hand-rolling a flowchart with boxes labeled
"Frontend"/"API"/"Database" and inventing your own ad-hoc legend for box shape/color meaning —
that's C4Container reinvented from scratch, without the `C4Container` renderer's built-in legend,
styling, and this skill's audience-tiering table (below) that already solves that exact need.

## C4 Diagram Levels

Select the appropriate level based on the documentation need:

| Level | Diagram Type | Audience | Shows | When to Create |
|-------|-------------|----------|-------|----------------|
| 1 | **C4Context** | Everyone | System + external actors | Always (required) |
| 2 | **C4Container** | Technical | Apps, databases, services | Always (required) |
| 3 | **C4Component** | Developers | Internal components | Only if adds value |
| 4 | **C4Deployment** | DevOps | Infrastructure nodes | For production systems |
| - | **C4Dynamic** | Technical | Request flows (numbered) | For complex workflows |

**Key Insight:** "Context + Container diagrams are sufficient for most software development teams." Only create Component/Code diagrams when they genuinely add value.

## Diagram Levels — Syntax and Examples

MANDATORY: Read [references/c4-syntax.md](references/c4-syntax.md) before writing any C4
diagram. It holds the complete element syntax (Person/System/Container/Component/Deployment/
Rel/Boundary variants) AND a full worked example per level (Context, Container, Component,
Dynamic, Deployment) plus styling/layout directives (`UpdateLayoutConfig`, `UpdateElementStyle`,
`UpdateRelStyle`) and Mermaid's PlantUML-C4 feature gaps. This file tells you WHICH level to
reach for (§ Decision Framework, § C4 Diagram Levels above); c4-syntax.md shows HOW to write it —
don't reconstruct examples from memory when that file has them worked out per level already.

## Best Practices

### Notation Rules

1. **Every element carries Name + Type + Technology (where applicable) + Description** — this is
   the actual C4 element contract, not decoration; an element missing one is unclassifiable to a
   reader (see mistake #4 below).
2. **Arrows are unidirectional, action-verb labeled, and carry a technology/protocol** —
   "Sends email using", "Reads/writes order data via JDBC" — never a bare "uses" or a
   bidirectional arrow (ambiguous initiator and payload).
3. **One diagram per file, one abstraction level per diagram** — mixing containers and components
   on the same canvas is the same ambiguity problem as dropping type labels (below).

### What to Avoid

The four sharpest mistakes, ordered by how often they silently mislead a reader rather than
just looking sloppy:

1. **Confusing containers and components.** Containers are deployable units (a service, a
   database); components are non-deployable code inside one (a class, a module). Drawing a class
   as a `Container` implies it scales, restarts, and fails independently — false, and it misleads
   exactly the audience (architects, DevOps) reading the diagram to find what can fail or scale
   on its own.
2. **Modeling a shared library as its own container.** A library is copied into every consumer,
   not deployed separately — showing it as a peer container implies a runtime dependency edge
   (network hop, independent failure mode) that doesn't exist. Show it as a `Component` inside
   each consuming container, or omit it as an implementation detail.
3. **Showing a message broker as one hub-and-spoke container.** Repeating `Rel(svc, kafka,
   "pub/sub")` for every service hides the actual data flow — which service produces which event
   and which consumes it. Model individual topics (or put the topic name on the relationship
   label) so the diagram still answers "what depends on what," the entire reason to draw it.
4. **Removing type labels to "simplify."** Dropping `Container`/`Component`/`System` forces the
   reader to guess deployability and abstraction level — which defeats C4's one real advantage
   over an ad-hoc box diagram: consistent, unambiguous notation across zoom levels (see Decision
   Framework above). A "simplified" diagram that reintroduces that ambiguity isn't simpler, it's
   a flowchart wearing C4 syntax.

MANDATORY: Read [references/common-mistakes.md](references/common-mistakes.md) when reviewing a
drafted diagram before shipping it — full catalog of undefined abstraction levels, external-system
internals leaking into Context diagrams, deployment-diagram mixing, and additional arrow-convention
violations beyond the four above.

## Microservices Guidelines

MANDATORY: Read [references/advanced-patterns.md](references/advanced-patterns.md) when
documenting a system with multiple team-owned services or event-driven/queue-based communication
— covers single-team vs multi-team ownership modeling and event-driven/queue-topic patterns with
worked examples.

## Output Location

Write architecture documentation to `docs/architecture/` with naming convention:
- `c4-context.md` - System context diagram
- `c4-containers.md` - Container diagram
- `c4-components-{feature}.md` - Component diagrams per feature
- `c4-deployment.md` - Deployment diagram
- `c4-dynamic-{flow}.md` - Dynamic diagrams for specific flows

## Audience-Appropriate Detail

| Audience | Recommended Diagrams |
|----------|---------------------|
| Executives | System Context only |
| Product Managers | Context + Container |
| Architects | Context + Container + key Components |
| Developers | All levels as needed |
| DevOps | Container + Deployment |

## References

- [references/c4-syntax.md](references/c4-syntax.md) — MANDATORY before writing any diagram: complete element syntax + a full worked example per level + styling/layout directives
- [references/common-mistakes.md](references/common-mistakes.md) — MANDATORY before shipping a diagram: anti-pattern catalog
- [references/advanced-patterns.md](references/advanced-patterns.md) — MANDATORY when modeling microservices or event-driven systems: multi-team ownership + topic patterns
