// Fixture: invalid — direct @<ws>/db import from apps/nextjs/src/** triggers the rule.
// Synthetic path: the file lives at __fixtures__/.../apps/nextjs/src/invalid.tsx so
// the rule's APP_SRC_REGEX (/apps\/nextjs\/src\//) matches its filename.

// expect: report — server component importing db client directly
import { db } from "@acme/db";

// expect: report — subpath import is also banned (db/client, db/schema, etc.)
import { users } from "@acme/db/schema";

// expect: report — different workspace name still banned
import { db as tcDb } from "@storefront/db";

export async function ServerComponent() {
  const all = await db.select().from(users);
  return <pre>{JSON.stringify(all)}</pre>;
}

// Allowed imports — these must NOT fire:
import { api } from "@acme/api"; // tRPC client is the canon path
import { cn } from "@acme/ui/utils"; // UI utils
import { z } from "zod"; // external lib
