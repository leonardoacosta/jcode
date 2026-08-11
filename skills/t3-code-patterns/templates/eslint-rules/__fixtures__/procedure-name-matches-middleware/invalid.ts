// Fixture: invalid — each declaration matching the default
// `procedurePatterns` (["^admin", "Admin$"]) without a role/permission
// middleware should be flagged.

declare const t: any;
declare const protectedProcedure: any;
declare const requireTenantContext: any;
declare const logRequest: any;

// expect: report — name starts with "admin" but only tenancy middleware.
export const adminProcedure = protectedProcedure.use(
  requireTenantContext("event"),
);

// expect: report — name ends with "Admin" but no role check in chain.
export const tenantAdmin = protectedProcedure
  .use(requireTenantContext("event"))
  .use(logRequest);

// expect: report — no .use() at all.
export const adminPublic = t.procedure;
