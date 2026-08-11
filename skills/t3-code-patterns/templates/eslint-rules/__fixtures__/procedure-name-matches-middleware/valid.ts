// Fixture: valid — none of these should fire.

declare const protectedProcedure: any;
declare const requireRole: any;
declare const requireAdminRole: any;
declare const hasPermission: any;
declare const rbacCheck: any;
declare const requireTenantContext: any;

// Has `requireRole` — substring "role" matches roleCheckIdentifiers.
export const adminProcedure = protectedProcedure
  .use(requireTenantContext("event"))
  .use(requireRole("admin"));

// Has `requireAdminRole` — substring "Admin" matches.
export const seriesAdmin = protectedProcedure.use(requireAdminRole());

// Has `hasPermission` — substring "permission" matches.
export const adminMutation = protectedProcedure.use(hasPermission("write"));

// Has `rbacCheck` — substring "rbac" matches.
export const adminQuery = protectedProcedure.use(rbacCheck("read"));

// Name does NOT match procedurePatterns — out of scope, never reported.
export const regularProcedure = protectedProcedure.use(
  requireTenantContext("event"),
);
