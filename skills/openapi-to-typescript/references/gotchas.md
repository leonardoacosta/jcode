
# Gotchas

Hard-won failure modes in OpenAPI-to-TypeScript conversion — each one produces a generated type
or type guard that type-checks cleanly but is silently wrong at runtime. Generic "unknown type ->
warn" advice doesn't catch these; they require actually knowing the OpenAPI 3.0 spec semantics.

- **NEVER collapse `nullable: true` into "not required."** OpenAPI models nullability and
  optionality as two independent flags: `nullable` on the schema, presence in `required[]`. A
  field that is BOTH `required` and `nullable: true` needs `field: string | null`, never `field?:
  string` — the API always sends the key, but its value may be `null`. Treating `nullable` as a
  synonym for optional produces a type that silently accepts `undefined` where the wire format
  never omits the key.

- **NEVER assume `additionalProperties` is banned by default.** Its absence means arbitrary extra
  properties are legal per spec — a schema is NOT closed unless `additionalProperties: false` is
  explicit. Emitting a strict/closed `interface` for a schema that never set this key produces
  false type errors the moment the API adds a field the client doesn't yet know about.

- **NEVER emit a type guard for `oneOf` without a `discriminator` as if it were safe.** Without
  `discriminator.propertyName`, disambiguating union members requires structurally probing every
  member schema — if two members share a field, the "safe" type guard can return true for the
  wrong member. Flag the ambiguity in the output (a comment, a TODO in the generated file) rather
  than silently picking a disambiguating field that happens to work for the example in hand.

- **NEVER "last one wins" merge conflicting `allOf` property overrides.** When two schemas in the
  same `allOf` both declare a property with different types, spec semantics require the value to
  satisfy BOTH (their intersection) — a naive merge that just overwrites with the later schema
  silently drops the stricter constraint. Rare, but produces confusing generated interfaces when
  it happens and is easy to miss without knowing to look for it.

- **NEVER resolve every `$ref` as if it points into `#/components/schemas/*`.** A `$ref` inside a
  `parameters` array frequently points at `#/components/parameters/*` (a shared parameter
  definition), which has a different shape than a schema. Check the ref path prefix before
  resolving — assuming schemas is the more common case, but assuming it universally produces the
  wrong TypeScript shape for shared-parameter refs.

- **NEVER try to inline a circular `$ref` chain (A -> B -> A).** Naive inlining recurses forever.
  Break the cycle with a named type alias that references the other type by name, not an inline
  object literal — this is a structural requirement, not a style choice.

- **NEVER claim a generated type guard "validates" a `format`-constrained field.** `format: uuid` /
  `date-time` / `email` all map to plain `string` in TypeScript, and the generated guard's
  `typeof === 'string'` check passes for ANY string — `"not-a-uuid"` satisfies `isUser()` just as
  well as a real UUID. The guard validates the primitive shape only; format constraints need
  separate runtime validation (a regex, a library) if the caller actually needs them enforced.
