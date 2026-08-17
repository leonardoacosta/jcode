import { render, screen, fireEvent } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { DecisionInbox, SplitWorkspace, InitiativeList } from "../src/components/CommandCenter";
import { loadFailureState } from "../src/app";
import { createProjectionStore } from "../src/stores/projection";
import { HttpCommandCenterTransport } from "../src/transport/client";
import { liveSnapshot, nextEvent, unavailableSnapshot } from "./fixtures/snapshots";
import type { EventEnvelope } from "../src/generated/command-center-contract";

describe("command center components", () => {
  it("renders durable Decision Inbox provenance, categories, and approval state", () => {
    render(() => (
      <DecisionInbox
        snapshot={{
          generatedAt: "2026-08-17T05:00:00Z",
          items: [
            {
              recordId: 1,
              source: {
                adapter: "slack",
                senderIdentity: "operator",
                conversation: "sl:D123",
              },
              receivedAt: "2026-08-17T05:00:00Z",
              content: "implement the Decision Inbox",
              category: "work_request",
              status: "awaiting_approval",
              proposal: { id: 1, state: "awaiting_approval" },
              dedupeKey: "sha256:test",
              duplicateDeliveries: 1,
              retryDeliveries: 0,
              redacted: false,
              rawPayloadRetained: true,
            },
          ],
        }}
      />
    ));
    expect(screen.getByRole("heading", { name: "Decision Inbox" })).toBeInTheDocument();
    expect(screen.getByText("Slack")).toBeInTheDocument();
    expect(screen.getByText("Work request")).toBeInTheDocument();
    expect(screen.getByText("Awaiting approval")).toBeInTheDocument();
    expect(screen.getByText("1 duplicate delivery retained")).toBeInTheDocument();
    expect(screen.getByText("sl:D123")).toBeInTheDocument();
  });

  it("renders durable and live panes with accessible states", () => {
    render(() => (
      <SplitWorkspace
        initiative={liveSnapshot.selectedInitiative!}
        run={liveSnapshot.selectedRun}
        onCheckpoint={vi.fn()}
      />
    ));
    expect(screen.getByRole("heading", { name: "Jcode Command Center" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Live execution" })).toBeInTheDocument();
    expect(screen.getByText("every 30 minutes")).toBeInTheDocument();
    expect(screen.getByLabelText("Virtualized event timeline").children.length).toBeLessThanOrEqual(
      40,
    );
  });

  it("keeps durable pane usable when Orca is unavailable", () => {
    render(() => (
      <SplitWorkspace
        initiative={unavailableSnapshot.selectedInitiative!}
        run={unavailableSnapshot.selectedRun}
        onCheckpoint={vi.fn()}
      />
    ));
    expect(screen.getByText("Orca runtime unavailable")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeDisabled();
    expect(screen.getByLabelText("Checkpoint summary")).toBeEnabled();
  });

  it("shows pending and failed command recovery affordances", () => {
    render(() => (
      <SplitWorkspace
        initiative={liveSnapshot.selectedInitiative!}
        run={liveSnapshot.selectedRun}
        onCheckpoint={vi.fn()}
        pending
        failure="Stale revision"
      />
    ));
    expect(screen.getByRole("button", { name: "Checkpoint pending" })).toBeDisabled();
    expect(screen.getByRole("alert")).toHaveTextContent("Stale revision");
    expect(screen.getByRole("button", { name: "Inspect" })).toBeInTheDocument();
  });

  it("renders empty initiative list state", () => {
    render(() => <InitiativeList initiatives={[]} />);
    expect(screen.getByText("No initiatives")).toBeInTheDocument();
  });

  it("resizes split panes through keyboard-operable range input", () => {
    render(() => (
      <SplitWorkspace
        initiative={liveSnapshot.selectedInitiative!}
        run={liveSnapshot.selectedRun}
        onCheckpoint={vi.fn()}
      />
    ));
    const slider = screen.getByLabelText("Pane size");
    fireEvent.input(slider, { target: { value: "60" } });
    expect(slider).toHaveValue("60");
  });

  it("keeps skip-link, labeled resizer, status, and action controls accessible", () => {
    render(() => (
      <SplitWorkspace
        initiative={liveSnapshot.selectedInitiative!}
        run={liveSnapshot.selectedRun}
        onCheckpoint={vi.fn()}
      />
    ));
    expect(screen.getByLabelText("Split initiative and execution workspace")).toBeInTheDocument();
    expect(
      screen.getAllByRole("status").some((status) => status.textContent?.includes("Run health")),
    ).toBe(true);
    screen.getByLabelText("Pane size").focus();
    expect(screen.getByLabelText("Pane size")).toHaveFocus();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeEnabled();
  });

  it("classifies auth expiry, forbidden, not-found, and fallback states explicitly", () => {
    expect(loadFailureState(new Error("bootstrap_401")).title).toBe("Authentication expired");
    expect(loadFailureState(new Error("snapshot_403")).title).toBe("Initiative forbidden");
    expect(loadFailureState(new Error("snapshot_404")).title).toBe("Initiative not found");
    expect(loadFailureState(new Error("snapshot_500")).title).toBe("Snapshot failed");
  });

  it("keeps embedded width and reduced-motion CSS guarantees", () => {
    const css = readFileSync("src/styles.css", "utf8");
    expect(css).toContain("@media (max-width: 760px)");
    expect(css).toContain("grid-template-columns: 1fr");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toContain("animation: none !important");
  });
});

describe("projection store", () => {
  it("applies only next sequence events and preserves local state", () => {
    const store = createProjectionStore(liveSnapshot);
    store.setUi("durablePanePercent", 60);
    expect(store.applyEvent(nextEvent)).toBe("applied");
    expect(store.snapshot?.selectedRun?.timeline.at(-1)?.message).toBe("Reconnect event applied");
    expect(store.ui.durablePanePercent).toBe(60);
  });

  it("detects event gaps and requires snapshot replacement", () => {
    const store = createProjectionStore(liveSnapshot);
    expect(store.applyEvent({ ...nextEvent, sequence: 13 })).toBe("snapshot_required");
    expect(store.snapshot?.connection.state).toBe("stale");
    expect(store.snapshot?.connection.reason).toBe("Event gap detected");
    expect(store.ui.announcement).toContain("Fresh snapshot required");
  });

  it("atomically replaces snapshots after reconnect", () => {
    const store = createProjectionStore(liveSnapshot);
    store.setUi("checkpointDraft", "local draft");
    store.installSnapshot(unavailableSnapshot);
    expect(store.snapshot?.selectedRun?.health).toBe("unavailable");
    expect(store.ui.checkpointDraft).toBe("local draft");
  });

  it("requires a snapshot for camelized Rust unknown events", () => {
    const store = createProjectionStore(liveSnapshot);
    const event = {
      ...nextEvent,
      payload: { type: "unknown", name: "schedule_updated", requiresSnapshot: true },
    } as unknown as EventEnvelope;

    expect(store.applyEvent(event)).toBe("snapshot_required");
    expect(store.snapshot?.meta.sequence).toBe(liveSnapshot.meta.sequence);
  });

  it("applies ignorable unknown events without discarding local UI state", () => {
    const store = createProjectionStore(liveSnapshot);
    store.setUi("checkpointDraft", "keep me");
    const event = {
      ...nextEvent,
      payload: { type: "unknown", name: "diagnostic_ping", requiresSnapshot: false },
    } as unknown as EventEnvelope;

    expect(store.applyEvent(event)).toBe("applied");
    expect(store.snapshot?.meta.sequence).toBe(nextEvent.sequence);
    expect(store.ui.checkpointDraft).toBe("keep me");
  });

  it("does not install a Rust run reference as a browser run projection", () => {
    const store = createProjectionStore(liveSnapshot);
    const event = {
      ...nextEvent,
      payload: {
        type: "run_updated",
        run: {
          id: "run-1",
          initiativeId: "init-command-center",
          status: "running",
          createdAt: "2026-08-11T05:00:00Z",
          updatedAt: "2026-08-11T05:13:00Z",
        },
      },
    } as unknown as EventEnvelope;

    expect(store.applyEvent(event)).toBe("snapshot_required");
    expect(store.snapshot?.selectedRun).toBe(liveSnapshot.selectedRun);
  });
});

describe("transport event cursor", () => {
  it("rejects scoped cursor escapes before opening an event stream", () => {
    const original = globalThis.EventSource;
    const eventSource = vi.fn();
    globalThis.EventSource = eventSource as unknown as typeof EventSource;
    const statuses: string[] = [];
    const unsubscribe = new HttpCommandCenterTransport().subscribe(
      "../other-stream",
      0,
      vi.fn(),
      (status) => statuses.push(status),
    );
    unsubscribe();
    expect(statuses).toEqual(["disconnected"]);
    expect(eventSource).not.toHaveBeenCalled();
    globalThis.EventSource = original;
  });

  it("refreshes expired auth before command send and reshapes exact Rust DTO transport", async () => {
    const requests: { url: string; init?: RequestInit }[] = [];
    const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
      requests.push({ url, init });
      if (url.endsWith("/bootstrap")) {
        return new Response(
          JSON.stringify({
            id: `session-${requests.length}`,
            csrf_token: `csrf-${requests.length}`,
            expires_at: requests.length === 1 ? "2000-01-01T00:00:00Z" : "2099-01-01T00:00:00Z",
          }),
          { status: 200 },
        );
      }
      return new Response(
        JSON.stringify({
          command_id: "cmd-1",
          idempotency_key: "idem-1",
          correlation_id: "corr-1",
          state: "failed",
          authoritative: null,
          error: { kind: "forbidden" },
        }),
        { status: 200 },
      );
    });
    vi.stubGlobal("fetch", fetchMock);

    const transport = new HttpCommandCenterTransport();
    await transport.loadSnapshot("/initiatives").catch(() => undefined);
    const result = await transport.sendCommand({
      idempotencyKey: "idem-1",
      payload: {
        type: "update_step",
        initiativeId: "init-command-center",
        expectedRevision: 3,
        milestoneId: "m1",
        stepId: "s2",
        status: "completed",
      },
    });

    const commandRequest = requests.find((request) => request.url.endsWith("/commands"));
    expect(fetchMock).toHaveBeenCalledTimes(4);
    expect(commandRequest?.init?.headers).toMatchObject({
      authorization: "Bearer session-3",
      "x-csrf-token": "csrf-3",
      "x-expected-revision": "3",
    });
    expect(JSON.parse(String(commandRequest?.init?.body))).toEqual({
      idempotencyKey: "idem-1",
      payload: {
        type: "update_step",
        initiative_id: "init-command-center",
        milestone_id: "m1",
        step_id: "s2",
        status: "completed",
      },
    });
    expect(result).toEqual({
      state: "failed",
      correlationId: "corr-1",
      snapshot: undefined,
      error: { code: "forbidden", message: "forbidden", inspect: undefined },
    });
  });
});
