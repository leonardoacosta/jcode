import { render, screen, fireEvent } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { SplitWorkspace, InitiativeList } from "../src/components/CommandCenter";
import { createProjectionStore } from "../src/stores/projection";
import { HttpCommandCenterTransport } from "../src/transport/client";
import { liveSnapshot, nextEvent, unavailableSnapshot } from "./fixtures/snapshots";

describe("command center components", () => {
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
  });

  it("atomically replaces snapshots after reconnect", () => {
    const store = createProjectionStore(liveSnapshot);
    store.setUi("checkpointDraft", "local draft");
    store.installSnapshot(unavailableSnapshot);
    expect(store.snapshot?.selectedRun?.health).toBe("unavailable");
    expect(store.ui.checkpointDraft).toBe("local draft");
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
});
