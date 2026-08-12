import { render, screen, cleanup } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { SplitWorkspace } from "../src/components/CommandCenter";
import { createProjectionStore } from "../src/stores/projection";
import { liveSnapshot, nextEvent } from "./fixtures/snapshots";
import type { EventEnvelope, RunProjection } from "../src/generated/command-center-contract";

const thresholds = {
  largeTimelineRenderMs: 750,
  eventUpdateMs: 50,
  reconnectSnapshotMs: 50,
  maxRenderedTimelineRows: 40,
  maxRetainedChildNodesAfterCleanup: 0,
};

function largeRun(eventCount: number): RunProjection {
  const timeline = Array.from({ length: eventCount }, (_, index) => ({
    id: `perf-event-${index + 1}`,
    sequence: index + 1,
    timestamp: "2026-08-11T05:00:00Z",
    source: index % 3 === 0 ? ("jcode" as const) : ("orca" as const),
    severity: "info" as const,
    message: `bounded perf event ${index + 1}`,
  }));
  return { ...liveSnapshot.selectedRun!, timeline };
}

describe("command center bounded performance envelope", () => {
  it("renders a large timeline within the bounded virtualized row budget", () => {
    const started = performance.now();
    const result = render(() => (
      <SplitWorkspace
        initiative={liveSnapshot.selectedInitiative!}
        run={largeRun(5_000)}
        onCheckpoint={() => undefined}
      />
    ));
    const elapsed = performance.now() - started;
    const timeline = screen.getByLabelText("Virtualized event timeline");

    expect(elapsed).toBeLessThan(thresholds.largeTimelineRenderMs);
    expect(timeline.children.length).toBeLessThanOrEqual(thresholds.maxRenderedTimelineRows);

    result.unmount();
    cleanup();
    expect(document.body.childElementCount).toBe(thresholds.maxRetainedChildNodesAfterCleanup);
  });

  it("applies event and reconnect updates without unbounded event growth", () => {
    const store = createProjectionStore({
      ...liveSnapshot,
      selectedRun: largeRun(5_000),
      meta: { ...liveSnapshot.meta, sequence: nextEvent.sequence - 1 },
    });
    const event = {
      ...nextEvent,
      payload: {
        type: "run_updated",
        run: largeRun(5_001),
      },
    } as EventEnvelope;

    const eventStarted = performance.now();
    expect(store.applyEvent(event)).toBe("applied");
    const eventElapsed = performance.now() - eventStarted;

    const reconnectStarted = performance.now();
    store.installSnapshot(liveSnapshot);
    const reconnectElapsed = performance.now() - reconnectStarted;

    expect(eventElapsed).toBeLessThan(thresholds.eventUpdateMs);
    expect(reconnectElapsed).toBeLessThan(thresholds.reconnectSnapshotMs);
    expect(store.snapshot?.selectedRun?.timeline.length).toBe(
      liveSnapshot.selectedRun?.timeline.length,
    );
  });
});
