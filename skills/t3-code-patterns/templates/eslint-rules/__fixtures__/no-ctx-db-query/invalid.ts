// Fixture: invalid — every ctx.db.query.* access below should trigger
// `no-ctx-db-query`. Mirrors the violation cluster found at
// acme/packages/api/src/router/admin/safety-reports/triage.ts:59.

export async function bad1(ctx: any) {
  // expect: report on ctx.db.query
  return ctx.db.query.safetyReports.findMany({});
}

export async function bad2(ctx: any) {
  // expect: report on ctx.db.query (different sub-table)
  const row = await ctx.db.query.user.findFirst({});
  return row;
}

export async function bad3(ctx: any) {
  // expect: report — chained .where() is irrelevant; the smell is .query
  return ctx.db.query.events.findMany({}).then((r: any) => r);
}
