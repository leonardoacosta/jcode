import type {
  CommandCenterSnapshot,
  EventEnvelope,
} from "../../src/generated/command-center-contract";

export const liveSnapshot: CommandCenterSnapshot = {
  meta: {
    protocolVersion: "command-center.v1",
    snapshotRevision: 7,
    streamId: "stream-local",
    sequence: 10,
  },
  connection: { state: "live", lastConnectedAt: "2026-08-11T05:00:00Z" },
  initiatives: [],
  selectedInitiative: {
    id: "init-command-center",
    title: "Jcode Command Center",
    outcome: "Supervise durable initiatives beside live execution.",
    status: "active",
    revision: 3,
    currentMilestone: {
      id: "m1",
      title: "Vertical slice",
      status: "running",
      steps: [
        {
          id: "s1",
          title: "Contract projection",
          status: "completed",
          evidence: "Generated client consumed",
        },
        { id: "s2", title: "Workspace frontend", status: "running" },
      ],
    },
    successCriteria: ["Durable state visible", "Live run projected", "Reconnect reconciles"],
    blockers: ["Runtime topology is fixture-backed for frontend tests"],
    nextActions: ["Validate e2e degraded mode"],
    children: [
      {
        id: "add-solidstart-command-center-vertical-slice",
        kind: "openspec",
        title: "SolidStart vertical slice",
      },
    ],
    schedules: [
      {
        id: "sched-1",
        cadence: "every 30 minutes",
        timezone: "UTC",
        nextFire: "2026-08-11T06:00:00Z",
        lastResult: "completed",
        retryState: "none",
        freshness: "live",
        evidence: "Last schedule evidence from fixture",
      },
    ],
    checkpoints: [
      { id: "cp-1", summary: "Frontend route established", createdAt: "2026-08-11T05:10:00Z" },
    ],
    freshness: "live",
    updatedAt: "2026-08-11T05:10:00Z",
    availableActions: {
      checkpoint: true,
      updateMilestone: true,
      startRun: false,
      retryRun: true,
      cancelRun: true,
    },
  },
  selectedRun: {
    id: "run-1",
    initiativeId: "init-command-center",
    status: "running",
    health: "live",
    orcaProjectId: "orca-project-1",
    orcaRunId: "orca-run-1",
    lastObservedAt: "2026-08-11T05:12:00Z",
    workers: [{ id: "w1", label: "frontend", status: "active", sessionId: "sess-1" }],
    gates: [{ id: "g1", title: "Review gate", status: "open" }],
    timeline: Array.from({ length: 80 }, (_, index) => ({
      id: `e${index + 1}`,
      sequence: index + 1,
      timestamp: "2026-08-11T05:12:00Z",
      source: index % 2 ? "orca" : "jcode",
      message: `Ordered event ${index + 1}`,
      severity: "info",
    })),
    attention: ["Review gate open"],
    availableActions: { startRun: false, retryRun: true, cancelRun: true },
  },
};
liveSnapshot.initiatives = [liveSnapshot.selectedInitiative!];

export const unavailableSnapshot: CommandCenterSnapshot = {
  ...liveSnapshot,
  connection: { state: "stale", reason: "Orca unavailable" },
  selectedRun: {
    ...liveSnapshot.selectedRun!,
    health: "unavailable",
    lastObservedAt: "2026-08-11T04:45:00Z",
    availableActions: { startRun: false, retryRun: false, cancelRun: false },
  },
};

export const nextEvent: EventEnvelope = {
  protocolVersion: "command-center.v1",
  streamId: "stream-local",
  sequence: 11,
  timestamp: "2026-08-11T05:13:00Z",
  source: "jcode",
  entityRefs: ["run-1"],
  payload: {
    type: "timeline_appended",
    runId: "run-1",
    event: {
      id: "e81",
      sequence: 81,
      timestamp: "2026-08-11T05:13:00Z",
      source: "jcode",
      message: "Reconnect event applied",
      severity: "info",
    },
  },
};
