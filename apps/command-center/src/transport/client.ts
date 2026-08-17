import type {
  CommandCenterSnapshot,
  CommandEnvelope,
  CommandResult,
  DecisionInboxSnapshot,
  EventEnvelope,
} from "../generated/command-center-contract";
import type { MxHealthProjection } from "../generated/mx-health-contract";

export interface CommandCenterTransport {
  loadSnapshot(path: string): Promise<CommandCenterSnapshot>;
  loadDecisionInbox(): Promise<DecisionInboxSnapshot>;
  loadMxHealth(): Promise<MxHealthProjection>;
  sendCommand(command: CommandEnvelope): Promise<CommandResult>;
  subscribe(
    streamId: string,
    after: number,
    onEvent: (event: EventEnvelope) => void,
    onStatus: (status: "live" | "disconnected") => void,
  ): () => void;
}

interface BrowserSession {
  id: string;
  csrf_token: string;
  expires_at: string;
}

interface ReplayBatch {
  events: EventEnvelope[];
  snapshot_required?: boolean;
  snapshotRequired?: boolean;
}

export interface CommandCenterTransportMetrics {
  disconnectedCount: number;
  reconnectCount: number;
  replayPollCount: number;
  lastEventLatencyMs?: number;
  snapshotRequiredCount: number;
}

export interface CommandCenterTransportOptions {
  onMetrics?: (metrics: CommandCenterTransportMetrics) => void;
}

function transportMetrics(): CommandCenterTransportMetrics {
  return {
    disconnectedCount: 0,
    reconnectCount: 0,
    replayPollCount: 0,
    snapshotRequiredCount: 0,
  };
}

interface RustCommandResult {
  commandId?: string;
  idempotencyKey?: string;
  correlationId?: string;
  state: CommandResult["state"];
  authoritative?: unknown;
  snapshot?: CommandCenterSnapshot;
  error?: { kind?: string; message?: string; reason?: string; entity?: string; inspect?: string };
}

function camelize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(camelize);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, item]) => [
      key.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase()),
      camelize(item),
    ]),
  );
}

function initiativeIdFromPath(path: string): string | undefined {
  return path.match(/^\/initiatives\/([^/]+)/)?.[1];
}

function commandErrorCode(
  error: RustCommandResult["error"],
): NonNullable<CommandResult["error"]>["code"] {
  switch (error?.kind) {
    case "reauthentication_required":
    case "unauthorized":
      return "reauthentication_required";
    case "forbidden":
      return "forbidden";
    case "stale_revision":
      return "stale_revision";
    case "orca_unavailable":
      return "unavailable";
    default:
      return "invalid";
  }
}

function commandErrorMessage(error: RustCommandResult["error"]): string {
  if (!error) return "Command failed";
  if (error.message) return error.message;
  if (error.reason) return error.reason;
  if (error.entity) return `${error.entity} not found`;
  return error.kind?.replaceAll("_", " ") ?? "Command failed";
}

function browserCommandResult(value: unknown): CommandResult {
  const result = camelize(value) as RustCommandResult;
  return {
    state: result.state,
    correlationId: result.correlationId ?? result.commandId ?? result.idempotencyKey ?? "unknown",
    snapshot: result.snapshot,
    error: result.error
      ? {
          code: commandErrorCode(result.error),
          message: commandErrorMessage(result.error),
          inspect: result.error.inspect,
        }
      : undefined,
  };
}

export class HttpCommandCenterTransport implements CommandCenterTransport {
  private session?: BrowserSession;
  private sessionRequest?: Promise<BrowserSession>;
  private readonly onMetrics?: (metrics: CommandCenterTransportMetrics) => void;
  private readonly metrics = transportMetrics();

  constructor(
    private readonly baseUrl = "",
    options: CommandCenterTransportOptions = {},
  ) {
    this.onMetrics = options.onMetrics;
  }

  private updateMetrics(update: Partial<CommandCenterTransportMetrics>): void {
    Object.assign(this.metrics, update);
    this.onMetrics?.({ ...this.metrics });
  }

  private reportStatus(status: "live" | "disconnected", state: { disconnected: boolean }): void {
    if (status === "disconnected") {
      if (!state.disconnected) {
        state.disconnected = true;
        this.updateMetrics({ disconnectedCount: this.metrics.disconnectedCount + 1 });
      }
      return;
    }
    if (state.disconnected) {
      state.disconnected = false;
      this.updateMetrics({ reconnectCount: this.metrics.reconnectCount + 1 });
    }
  }

  private async authenticatedHeaders(mutating = false): Promise<Record<string, string>> {
    if (!this.session || Date.parse(this.session.expires_at) <= Date.now()) {
      this.sessionRequest ??= fetch(`${this.baseUrl}/api/command-center/bootstrap`, {
        method: "POST",
        credentials: "same-origin",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({}),
      })
        .then(async (response) => {
          if (!response.ok) throw new Error(`bootstrap_${response.status}`);
          return (await response.json()) as BrowserSession;
        })
        .finally(() => {
          this.sessionRequest = undefined;
        });
      this.session = await this.sessionRequest;
    }
    return {
      authorization: `Bearer ${this.session.id}`,
      ...(mutating ? { "x-csrf-token": this.session.csrf_token } : {}),
    };
  }

  async loadSnapshot(path: string): Promise<CommandCenterSnapshot> {
    const initiativeId = initiativeIdFromPath(path);
    const endpoint = initiativeId
      ? `/api/command-center/initiatives/${encodeURIComponent(initiativeId)}/snapshot`
      : "/api/command-center/initiatives";
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      credentials: "same-origin",
      headers: await this.authenticatedHeaders(),
    });
    if (!response.ok) throw new Error(`snapshot_${response.status}`);
    return camelize(await response.json()) as CommandCenterSnapshot;
  }

  async loadDecisionInbox(): Promise<DecisionInboxSnapshot> {
    const response = await fetch(`${this.baseUrl}/api/command-center/decision-inbox`, {
      credentials: "same-origin",
      headers: await this.authenticatedHeaders(),
    });
    if (!response.ok) throw new Error(`decision_inbox_${response.status}`);
    return camelize(await response.json()) as DecisionInboxSnapshot;
  }

  async loadMxHealth(): Promise<MxHealthProjection> {
    const response = await fetch(`${this.baseUrl}/api/command-center/mx-health`, {
      credentials: "same-origin",
      headers: await this.authenticatedHeaders(),
    });
    if (!response.ok) throw new Error(`mx_health_${response.status}`);
    return camelize(await response.json()) as MxHealthProjection;
  }

  async sendCommand(command: CommandEnvelope): Promise<CommandResult> {
    const expectedRevision = command.payload.expectedRevision;
    const payload =
      command.payload.type === "checkpoint_initiative"
        ? {
            type: "checkpoint",
            initiative_id: command.payload.initiativeId,
            summary: command.payload.summary,
            blockers: command.payload.blockers,
            next_actions: command.payload.nextActions,
          }
        : command.payload.type === "update_step"
          ? {
              type: "update_step",
              initiative_id: command.payload.initiativeId,
              milestone_id: command.payload.milestoneId,
              step_id: command.payload.stepId,
              status: command.payload.status,
            }
          : {
              type: command.payload.type,
              initiative_id: command.payload.initiativeId,
              ...(command.payload.runId ? { run_id: command.payload.runId } : {}),
            };
    const response = await fetch(`${this.baseUrl}/api/command-center/commands`, {
      method: "POST",
      credentials: "same-origin",
      headers: {
        "content-type": "application/json",
        "x-expected-revision": String(expectedRevision),
        ...(await this.authenticatedHeaders(true)),
      },
      body: JSON.stringify({ idempotencyKey: command.idempotencyKey, payload }),
    });
    if (!response.ok)
      return {
        state: "failed",
        correlationId: command.idempotencyKey,
        error: {
          code: response.status === 401 ? "reauthentication_required" : "invalid",
          message: `Command failed with HTTP ${response.status}`,
        },
      };
    return browserCommandResult(await response.json());
  }

  subscribe(
    streamId: string,
    after: number,
    onEvent: (event: EventEnvelope) => void,
    onStatus: (status: "live" | "disconnected") => void,
  ): () => void {
    if (!/^[-A-Za-z0-9_.:]+$/.test(streamId) || !Number.isSafeInteger(after) || after < 0) {
      onStatus("disconnected");
      return () => undefined;
    }
    let canceled = false;
    let sequence = after;
    const state = { disconnected: false };
    const poll = async () => {
      try {
        this.updateMetrics({ replayPollCount: this.metrics.replayPollCount + 1 });
        const headers = await this.authenticatedHeaders();
        const response = await fetch(
          `${this.baseUrl}/api/command-center/replay?stream_id=${encodeURIComponent(streamId)}&sequence=${sequence}`,
          { credentials: "same-origin", headers },
        );
        if (!response.ok) throw new Error(`replay_${response.status}`);
        const batch = camelize(await response.json()) as ReplayBatch;
        if (batch.snapshot_required || batch.snapshotRequired) {
          this.updateMetrics({ snapshotRequiredCount: this.metrics.snapshotRequiredCount + 1 });
        }
        for (const event of batch.events ?? []) {
          if (canceled) return;
          sequence = event.sequence;
          const eventTimestamp = Date.parse(event.timestamp);
          if (Number.isFinite(eventTimestamp)) {
            this.updateMetrics({ lastEventLatencyMs: Math.max(0, Date.now() - eventTimestamp) });
          }
          onEvent(event);
        }
        this.reportStatus("live", state);
        onStatus("live");
      } catch {
        this.reportStatus("disconnected", state);
        onStatus("disconnected");
      }
      if (!canceled) window.setTimeout(poll, 1_000);
    };
    void poll();
    return () => {
      canceled = true;
    };
  }
}
