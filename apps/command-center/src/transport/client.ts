import type {
  CommandCenterSnapshot,
  CommandEnvelope,
  CommandResult,
  EventEnvelope,
} from "../generated/command-center-contract";

export interface CommandCenterTransport {
  loadSnapshot(path: string): Promise<CommandCenterSnapshot>;
  sendCommand(command: CommandEnvelope): Promise<CommandResult>;
  subscribe(
    streamId: string,
    after: number,
    onEvent: (event: EventEnvelope) => void,
    onStatus: (status: "live" | "disconnected") => void,
  ): () => void;
}

export class HttpCommandCenterTransport implements CommandCenterTransport {
  constructor(private readonly baseUrl = "") {}

  async loadSnapshot(path: string): Promise<CommandCenterSnapshot> {
    const response = await fetch(
      `${this.baseUrl}/api/command-center/snapshot?route=${encodeURIComponent(path)}`,
      { credentials: "same-origin" },
    );
    if (!response.ok) throw new Error(`snapshot_${response.status}`);
    return (await response.json()) as CommandCenterSnapshot;
  }

  async sendCommand(command: CommandEnvelope): Promise<CommandResult> {
    const response = await fetch(`${this.baseUrl}/api/command-center/commands`, {
      method: "POST",
      credentials: "same-origin",
      headers: { "content-type": "application/json", "x-jcode-csrf": "same-origin" },
      body: JSON.stringify(command),
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
    return (await response.json()) as CommandResult;
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
    const events = new EventSource(
      `${this.baseUrl}/api/command-center/events?stream=${encodeURIComponent(streamId)}&after=${after}`,
    );
    events.onopen = () => onStatus("live");
    events.onerror = () => onStatus("disconnected");
    events.onmessage = (message) => onEvent(JSON.parse(message.data) as EventEnvelope);
    return () => events.close();
  }
}
