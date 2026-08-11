// Fixture: valid — none of these should fire.
import { db } from "@ws/db/client";

export async function good1() {
  // Direct db import — no ctx.db indirection.
  return db.query.user.findFirst({});
}

export async function good2(ctx: any) {
  // ctx.db (without .query) is fine — only the .query smell is flagged.
  return ctx.db.insert({});
}

export async function good3(ctx: any) {
  // ctx.session.query is unrelated and should not match.
  return ctx.session.query.user;
}

export async function good4(other: any) {
  // Different root identifier — only `ctx.db.query.*` is the target.
  return other.db.query.user.findFirst({});
}
