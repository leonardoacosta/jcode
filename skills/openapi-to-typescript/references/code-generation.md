
# Code Generation

## File Header

```typescript
/**
 * Auto-generated from: {source_file}
 * Generated at: {timestamp}
 *
 * DO NOT EDIT MANUALLY - Regenerate from OpenAPI schema
 */
```

## Interfaces (from components/schemas)

For each schema in `components/schemas`:

```typescript
export interface Product {
  /** Product unique identifier */
  id: string;

  /** Product title */
  title: string;

  /** Product price */
  price: number;

  /** Created timestamp */
  created_at?: string;
}
```

- Use OpenAPI description as JSDoc
- Fields in `required[]` have no `?`
- Fields outside `required[]` have `?`
- A field that is BOTH `required` and `nullable: true` is `field: string | null`, never `field?:
  string` — see `references/gotchas.md` for why collapsing these two independent flags is wrong.

## Request/Response Types (from paths)

For each endpoint in `paths`:

```typescript
// GET /products - query params
export interface GetProductsRequest {
  page?: number;
  limit?: number;
}

// GET /products - response 200
export type GetProductsResponse = ProductList;

// POST /products - request body
export interface CreateProductRequest {
  title: string;
  price: number;
}

// POST /products - response 201
export type CreateProductResponse = Product;
```

Naming convention:
- `{Method}{Path}Request` for params/body
- `{Method}{Path}Response` for response

## Type Guards

For each main interface, generate a type guard:

```typescript
export function isProduct(value: unknown): value is Product {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    typeof (value as any).id === 'string' &&
    'title' in value &&
    typeof (value as any).title === 'string' &&
    'price' in value &&
    typeof (value as any).price === 'number'
  );
}
```

Type guard rules:
- Check `typeof value === 'object' && value !== null`
- For each required field: check `'field' in value`
- For primitive fields: check `typeof`
- For arrays: check `Array.isArray()`
- For enums: check `.includes()`
- A generated type guard validates the TypeScript primitive shape only — it does NOT validate
  `format` constraints (a `format: uuid` field passes with any string) and it CANNOT safely
  disambiguate an undiscriminated `oneOf`. See `references/gotchas.md`.

## Error Type (always include)

```typescript
export interface ApiError {
  status: number;
  error: string;
  detail?: string;
}

export function isApiError(value: unknown): value is ApiError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'status' in value &&
    typeof (value as any).status === 'number' &&
    'error' in value &&
    typeof (value as any).error === 'string'
  );
}
```
