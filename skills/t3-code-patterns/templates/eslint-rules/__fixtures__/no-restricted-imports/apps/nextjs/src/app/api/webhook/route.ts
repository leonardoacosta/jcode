// Fixture: valid — App Router route handlers under app/api/**/route.ts are the
// explicit escape hatch. The same @<ws>/db imports that would fire elsewhere
// are allowed here, because route handlers are the canonical direct-db boundary.

// expect: 0 reports — route.ts is exempt
import { db } from "@acme/db";
import { users } from "@acme/db/schema";

// Standard route handler pattern: receive webhook, write to DB directly.
// No tRPC overhead for raw endpoints that need request/response control.
export async function POST(req: Request) {
  const body = await req.json();
  await db.insert(users).values({ email: body.email });
  return new Response("ok", { status: 200 });
}
