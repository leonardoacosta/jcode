---
name: openapi-to-typescript
description: Converts OpenAPI 3.0 JSON/YAML to TypeScript interfaces and type guards. This skill should be used when the user asks to generate types from OpenAPI, convert schema to TS, create API interfaces, or generate TypeScript types from an API specification.
source: ~/.agents/skills@2026-07-13
---


# OpenAPI to TypeScript

Converts OpenAPI 3.0 specifications to TypeScript interfaces and type guards.

**Input:** OpenAPI file (JSON or YAML)
**Output:** TypeScript file with interfaces and type guards

## When to Use

- "generate types from openapi"
- "convert openapi to typescript"
- "create API interfaces"
- "generate types from spec"

## Workflow

This is a decision-point workflow, not a linear transcription — the schema itself tells you which
gotcha to check for at each step. Naive codegen breaks on exactly these decision points, so treat
each one as a required check, not an optional deep-dive.

1. **Request the OpenAPI file path** (if not provided).
2. **Read and validate the file** (must be OpenAPI 3.0.x).
3. **Extract schemas from `components/schemas`.** For each schema, before writing the interface:
   - Is a property BOTH in `required[]` AND `nullable: true`? -> emit `field: T | null`, never
     `field?: T` — the key is always present, only the value may be null (gotchas.md
     nullable-vs-optional).
   - Does the schema omit `additionalProperties`? -> do NOT emit a closed/strict interface;
     absence means extra properties are legal per spec (gotchas.md additionalProperties).
   - Does the schema use `oneOf`? -> check for `discriminator.propertyName` before writing a type
     guard. No discriminator means structural probing can pick the wrong member — flag the
     ambiguity in the generated file rather than silently guessing (gotchas.md oneOf-discriminator).
   - Does the schema use `allOf`? -> check whether two member schemas override the same property
     with different types; that needs an intersection, not a last-one-wins overwrite (gotchas.md
     allOf-override).
   - Does the schema contain a `$ref`? -> confirm the path prefix before resolving (schemas vs.
     parameters — see step 4) and watch for a circular chain (A -> B -> A); break cycles with a
     named type alias, never inline recursion (gotchas.md circular-refs).
4. **Extract endpoints from `paths`** (request/response types). The same `$ref`-prefix check from
   step 3 applies here with a twist: parameter refs commonly point at
   `#/components/parameters/*`, not `#/components/schemas/*` — resolving them as schemas produces
   the wrong shape (gotchas.md $ref-path-prefix).
5. **Generate TypeScript** (interfaces + type guards). For any `format`-constrained field
   (`uuid`, `date-time`, `email`, ...), the generated guard can only validate the primitive shape
   (`typeof === 'string'`) — never claim or comment that it "validates" the format itself
   (gotchas.md format-constraint-false-confidence).
6. **Ask where to save** (default: `types/api.ts` in current directory).
7. **Write the file.**

Read `references/gotchas.md` before generating a type guard for any schema that uses `oneOf`,
`allOf`, or `nullable` — the generic type-mapping tables in `references/type-mapping.md` cover the
happy path, not these failure modes.

## OpenAPI Validation

Check before processing:

```
- Field "openapi" must exist and start with "3.0"
- Field "paths" must exist
- Field "components.schemas" must exist (if there are types)
```

If invalid, report the error and stop.

## Reference Files

Distinct sub-topics live in `references/` — read the one relevant to the current step, not all
of them:

- **`references/type-mapping.md`** — primitive/format-modifier tables, object/array/enum/oneOf/
  allOf complex-type shapes, `$ref` resolution basics.
- **`references/code-generation.md`** — file header convention, interface generation from
  `components/schemas`, request/response naming (`{Method}{Path}Request/Response`), type guard
  generation rules, the always-included `ApiError` type.
- **`references/worked-example.md`** — one complete input OpenAPI doc -> output TypeScript file,
  to check overall shape and naming against.
- **`references/gotchas.md`** — hard-won failure modes where a generated type or type guard
  type-checks cleanly but is silently wrong at runtime (nullable-vs-optional, `additionalProperties`
  defaults, undiscriminated `oneOf`, `allOf` override conflicts, `$ref` path-prefix mixups,
  circular refs, format-constraint false confidence). Read this before generating a type guard for
  any schema that uses `oneOf`, `allOf`, or `nullable` — these are exactly the cases where the
  generic type-mapping tables above aren't enough.
