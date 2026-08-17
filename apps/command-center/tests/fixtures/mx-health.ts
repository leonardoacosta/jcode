import type {
  MxAdapterState,
  MxFailureCategory,
  MxHealthProjection,
  MxHealthSnapshot,
  MxOverallStatus,
} from "../../src/generated/mx-health-contract";

const provenance = {
  id: "mx:fixture:mx.health.v1",
  repository: "https://example.invalid/mx-fixture",
  commit: "fixture-commit",
  implementationSha256: "fixture-implementation",
  specificationSha256: "fixture-specification",
  openapiSha256: "fixture-openapi",
  testsSha256: "fixture-tests",
};

export const healthyHealth: MxHealthSnapshot = {
  version: "mx.health.v1",
  generatedAt: "2026-08-17T18:00:00Z",
  overall: "ok",
  redacted: true,
  checks: [
    {
      id: "source.gmail",
      layer: "source",
      status: "ok",
      reasonCode: "source_serving",
      summary: "Source is serving",
    },
    {
      id: "persistence.sqlite",
      layer: "persistence",
      status: "ok",
      reasonCode: "persistence_ready",
      summary: "Persistence is ready",
    },
    {
      id: "workflow.inbox",
      layer: "workflow",
      status: "ok",
      reasonCode: "workflow_ready",
      summary: "Inbox workflow is ready",
      dependsOn: ["persistence.sqlite"],
    },
  ],
};

export const degradedHealth: MxHealthSnapshot = {
  ...healthyHealth,
  overall: "degraded",
  checks: healthyHealth.checks.map((check) =>
    check.id === "source.gmail"
      ? {
          ...check,
          status: "degraded" as const,
          reasonCode: "source_partial",
          summary: "Source is serving with reduced availability",
        }
      : check,
  ),
};

export const downHealth: MxHealthSnapshot = {
  ...healthyHealth,
  overall: "down",
  checks: healthyHealth.checks.map((check) =>
    check.id === "persistence.sqlite"
      ? {
          ...check,
          status: "down" as const,
          reasonCode: "persistence_unavailable",
          summary: "Persistence is unavailable",
        }
      : check.id === "workflow.inbox"
        ? {
            ...check,
            status: "blocked" as const,
            reasonCode: "dependency_down",
            summary: "Inbox workflow is blocked by persistence",
          }
        : check,
  ),
};

export function liveProjection(
  health: MxHealthSnapshot = healthyHealth,
  adapterState: MxAdapterState = "live",
): MxHealthProjection {
  return {
    provenance,
    adapterState,
    fetchedAt: "2026-08-17T18:00:05Z",
    health,
  };
}

export const staleProjection: MxHealthProjection = {
  ...liveProjection(healthyHealth, "stale"),
  failureCategory: "timeout",
  stale: {
    cachedFetchedAt: "2026-08-17T17:58:05Z",
    cachedGeneratedAt: healthyHealth.generatedAt,
    ageSeconds: 120,
    currentFailure: "timeout",
  },
};

export function adapterFailure(
  adapterState: Exclude<MxAdapterState, "live" | "stale" | "unconfigured">,
  failureCategory: MxFailureCategory,
): MxHealthProjection {
  return {
    ...liveProjection(undefined as never, adapterState),
    health: undefined,
    failureCategory,
  };
}

export const unconfiguredProjection: MxHealthProjection = {
  ...liveProjection(undefined as never, "unconfigured"),
  health: undefined,
};

export const unauthorizedProjection = adapterFailure("unauthorized", "unauthorized");
export const unreachableProjection = adapterFailure("unreachable", "unreachable");
export const timeoutProjection = adapterFailure("timeout", "timeout");
export const invalidContractProjection = adapterFailure("invalid_contract", "invalid_contract");

export function overallLabel(overall: MxOverallStatus) {
  return overall === "ok" ? "Healthy" : overall === "degraded" ? "Degraded" : "Down";
}
