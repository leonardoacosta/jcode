import { render, screen, cleanup } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AmbientActivity } from "../src/components/CommandCenter";
import { createProjectionStore } from "../src/stores/projection";
import { liveSnapshot, nextEvent } from "./fixtures/snapshots";
import type { EventEnvelope, RunProjection } from "../src/generated/command-center-contract";

const thresholds = {
  largeTimelineRenderMs: 750,
  eventUpdateMs: 50,
  reconnectSnapshotMs: 50,
  maxRenderedActivityRows: 10,
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
      <AmbientActivity
        snapshot={{
          ...liveSnapshot,
          selectedRun: largeRun(5_000),
        }}
      />
    ));
    const elapsed = performance.now() - started;
    const ledger = screen.getByRole("list", { name: "Ambient activity ledger" });

    expect(elapsed).toBeLessThan(thresholds.largeTimelineRenderMs);
    expect(ledger.children.length).toBeLessThanOrEqual(thresholds.maxRenderedActivityRows);

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
