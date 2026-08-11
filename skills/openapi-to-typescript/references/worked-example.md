
# Complete Example

**Input (OpenAPI):**
```json
{
  "openapi": "3.0.0",
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": {"type": "string", "format": "uuid"},
          "email": {"type": "string", "format": "email"},
          "role": {"type": "string", "enum": ["admin", "user"]}
        },
        "required": ["id", "email", "role"]
      }
    }
  },
  "paths": {
    "/users/{id}": {
      "get": {
        "parameters": [{"name": "id", "in": "path", "required": true}],
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": {"$ref": "#/components/schemas/User"}
              }
            }
          }
        }
      }
    }
  }
}
```

**Output (TypeScript):**
```typescript
/**
 * Auto-generated from: api.openapi.json
 * Generated at: 2025-01-15T10:30:00Z
 *
 * DO NOT EDIT MANUALLY - Regenerate from OpenAPI schema
 */

// ============================================================================
// Types
// ============================================================================

export type UserRole = "admin" | "user";

export interface User {
  /** UUID */
  id: string;

  /** Email */
  email: string;

  role: UserRole;
}

// ============================================================================
// Request/Response Types
// ============================================================================

export interface GetUserByIdRequest {
  id: string;
}

export type GetUserByIdResponse = User;

// ============================================================================
// Type Guards
// ============================================================================

export function isUser(value: unknown): value is User {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    typeof (value as any).id === 'string' &&
    'email' in value &&
    typeof (value as any).email === 'string' &&
    'role' in value &&
    ['admin', 'user'].includes((value as any).role)
  );
}

// ============================================================================
// Error Types
// ============================================================================

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
