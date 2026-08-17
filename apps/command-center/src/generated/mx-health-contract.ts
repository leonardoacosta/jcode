/**
 * Deterministic browser projection for the pinned MX mx.health.v1 adapter.
 *
 * This file is generated from the daemon's public Rust projection and contains
 * no endpoint, token, authorization header, or raw upstream error fields.
 */
export type MxOverallStatus = "ok" | "degraded" | "down";
export type MxCheckStatus = "ok" | "degraded" | "down" | "blocked";
export type MxAdapterState =
  | "live"
  | "stale"
  | "unconfigured"
  | "unauthorized"
  | "unreachable"
  | "timeout"
  | "invalid_contract"
  | "unavailable";
export type MxFailureCategory =
  | "unauthorized"
  | "unexpected_status"
  | "timeout"
  | "unreachable"
  | "oversized"
  | "invalid_contract";

export interface MxHealthCheck {
  id: string;
  layer: string;
  status: MxCheckStatus;
  reasonCode: string;
  summary: string;
  dependsOn?: string[];
}

export interface MxHealthSnapshot {
  version: "mx.health.v1";
  generatedAt: string;
  overall: MxOverallStatus;
  redacted: true;
  checks: MxHealthCheck[];
}

export interface MxHealthProvenance {
  id: string;
  repository: string;
  commit: string;
  implementationSha256: string;
  specificationSha256: string;
  openapiSha256: string;
  testsSha256: string;
}

export interface MxStaleMetadata {
  cachedFetchedAt: string;
  cachedGeneratedAt: string;
  ageSeconds: number;
  currentFailure: MxFailureCategory;
}

export interface MxHealthProjection {
  provenance: MxHealthProvenance;
  adapterState: MxAdapterState;
  failureCategory?: MxFailureCategory;
  fetchedAt: string;
  health?: MxHealthSnapshot;
  stale?: MxStaleMetadata;
}
