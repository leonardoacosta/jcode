
# Type Mapping

## Primitives

| OpenAPI     | TypeScript   |
|-------------|--------------|
| `string`    | `string`     |
| `number`    | `number`     |
| `integer`   | `number`     |
| `boolean`   | `boolean`    |
| `null`      | `null`       |

## Format Modifiers

| Format        | TypeScript              |
|---------------|-------------------------|
| `uuid`        | `string` (comment UUID) |
| `date`        | `string` (comment date) |
| `date-time`   | `string` (comment ISO)  |
| `email`       | `string` (comment email)|
| `uri`         | `string` (comment URI)  |

Format is documentation, not a distinct TypeScript type or a runtime guarantee — see
`references/gotchas.md` for why this matters for generated type guards.

## Complex Types

**Object:**
```typescript
// OpenAPI: type: object, properties: {id, name}, required: [id]
interface Example {
  id: string;      // required: no ?
  name?: string;   // optional: with ?
}
```

**Array:**
```typescript
// OpenAPI: type: array, items: {type: string}
type Names = string[];
```

**Enum:**
```typescript
// OpenAPI: type: string, enum: [active, draft]
type Status = "active" | "draft";
```

**oneOf (Union):**
```typescript
// OpenAPI: oneOf: [{$ref: Cat}, {$ref: Dog}]
type Pet = Cat | Dog;
```

**allOf (Intersection/Extends):**
```typescript
// OpenAPI: allOf: [{$ref: Base}, {type: object, properties: ...}]
interface Extended extends Base {
  extraField: string;
}
```

## $ref Resolution

When encountering `{"$ref": "#/components/schemas/Product"}`:
1. Extract the schema name (`Product`)
2. Use the type directly (don't resolve inline)

```typescript
// OpenAPI: items: {$ref: "#/components/schemas/Product"}
// TypeScript:
items: Product[]  // reference, not inline
```

Check the ref path prefix before resolving — see `references/gotchas.md` for the
`#/components/parameters/*` vs `#/components/schemas/*` mixup and circular-ref handling.
