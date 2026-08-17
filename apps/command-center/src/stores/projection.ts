import { createSignal } from "solid-js";
import type { CommandCenterSnapshot, EventEnvelope } from "../generated/command-center-contract";

type UnknownRecord = Record<string, unknown>;

function isRecord(value: unknown): value is UnknownRecord {
  return typeof value === "object" && value !== null;
}

function isInitiativeProjection(
  value: unknown,
): value is CommandCenterSnapshot["initiatives"][number] {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.title === "string" &&
    isRecord(value.currentMilestone) &&
    Array.isArray(value.schedules) &&
    isRecord(value.availableActions)
  );
}

function isRunProjection(
  value: unknown,
): value is NonNullable<CommandCenterSnapshot["selectedRun"]> {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.initiativeId === "string" &&
    typeof value.health === "string" &&
    Array.isArray(value.workers) &&
    Array.isArray(value.gates) &&
    Array.isArray(value.timeline) &&
    isRecord(value.availableActions)
  );
}

function unknownEventRequiresSnapshot(payload: UnknownRecord): boolean {
  return payload.requiresSnapshot === true || payload.requires_snapshot === true;
}

export interface LocalUiState {
  durablePanePercent: number;
  selectedRunId?: string;
  timelineFilter: "all" | "attention";
  checkpointDraft: string;
  announcement: string;
}

export function createProjectionStore(initial?: CommandCenterSnapshot) {
  const [snapshot, setSnapshotSignal] = createSignal<CommandCenterSnapshot | undefined>(initial);
  const [uiState, setUiState] = createSignal<LocalUiState>({
    durablePanePercent: 48,
    timelineFilter: "all",
    checkpointDraft: "",
    announcement: "Command center ready",
  });

  const setUi = <K extends keyof LocalUiState>(key: K, value: LocalUiState[K]) => {
    setUiState((current) => ({ ...current, [key]: value }));
  };

  const installSnapshot = (next: CommandCenterSnapshot) => {
    setSnapshotSignal(next);
    if (next.selectedRun?.id) setUi("selectedRunId", next.selectedRun.id);
    setUi("announcement", `Snapshot revision ${next.meta.snapshotRevision} installed`);
  };

  const applyEvent = (event: EventEnvelope): "applied" | "ignored" | "snapshot_required" => {
    const current = snapshot();
    if (!current) return "ignored";
    if (event.streamId !== current.meta.streamId || event.sequence !== current.meta.sequence + 1) {
      setSnapshotSignal({
        ...current,
        connection: { state: "stale", reason: "Event gap detected" },
      });
      setUi("announcement", "Event gap detected. Fresh snapshot required.");
      return "snapshot_required";
    }
    const withSequence = { ...current, meta: { ...current.meta, sequence: event.sequence } };
    const payload = event.payload;
    if (payload.type === "initiative_updated") {
      if (!isInitiativeProjection(payload.initiative)) return "snapshot_required";
      setSnapshotSignal({
        ...withSequence,
        selectedInitiative: payload.initiative,
        initiatives: withSequence.initiatives.map((item) =>
          item.id === payload.initiative.id ? payload.initiative : item,
        ),
      });
      setUi("announcement", `Initiative ${payload.initiative.title} updated`);
      return "applied";
    }
    if (payload.type === "run_updated") {
      if (!isRunProjection(payload.run)) return "snapshot_required";
      setSnapshotSignal({ ...withSequence, selectedRun: payload.run });
      setUi("announcement", `Run ${payload.run.id} updated`);
      return "applied";
    }
    if (payload.type === "timeline_appended") {
      setSnapshotSignal({
        ...withSequence,
        selectedRun:
          withSequence.selectedRun?.id === payload.runId
            ? {
                ...withSequence.selectedRun,
                timeline: [...withSequence.selectedRun.timeline, payload.event],
              }
            : withSequence.selectedRun,
      });
      setUi("announcement", payload.event.message);
      return "applied";
    }
    if (payload.type === "snapshot_required") return "snapshot_required";
    if (payload.type === "unknown" && unknownEventRequiresSnapshot(payload)) {
      return "snapshot_required";
    }
    setSnapshotSignal(withSequence);
    return "applied";
  };

  const markDisconnected = (reason: string) => {
    const current = snapshot();
    if (!current) return;
    setSnapshotSignal({
      ...current,
      connection: { ...current.connection, state: "stale", reason },
    });
    setUi("announcement", reason);
  };

  return {
    get snapshot() {
      return snapshot();
    },
    get ui() {
      return uiState();
    },
    setUi,
    installSnapshot,
    applyEvent,
    markDisconnected,
  };
}
