import { expect, test } from "@playwright/test";
import { liveSnapshot, nextEvent, unavailableSnapshot } from "../tests/fixtures/snapshots";
import type { CommandCenterSnapshot } from "../src/generated/command-center-contract";

const isFixtureMode = (project: { metadata: Record<string, unknown> }) =>
  project.metadata.fixtureMode === true;
const isOrcaUnavailable = (project: { metadata: Record<string, unknown> }) =>
  project.metadata.orcaUnavailable === true;

async function installFixture(
  page: import("@playwright/test").Page,
  snapshot: CommandCenterSnapshot,
) {
  let current = structuredClone(snapshot) as CommandCenterSnapshot;
  await page.addInitScript((event) => {
    class FixtureEventSource extends EventTarget {
      onopen: (() => void) | null = null;
      onerror: (() => void) | null = null;
      onmessage: ((message: MessageEvent) => void) | null = null;
      constructor(url: string) {
        super();
        if (url.includes("stream=../")) {
          queueMicrotask(() => this.onerror?.());
          return;
        }
        queueMicrotask(() => this.onopen?.());
        setTimeout(
          () => this.onmessage?.(new MessageEvent("message", { data: JSON.stringify(event) })),
          50,
        );
      }
      close() {}
    }
    Object.defineProperty(window, "EventSource", { value: FixtureEventSource });
  }, nextEvent);
  await page.route("**/api/command-center/snapshot**", async (route) =>
    route.fulfill({ json: current }),
  );
  await page.route("**/api/command-center/commands", async (route) => {
    const body = route.request().postDataJSON() as {
      payload: { type: string; stepId?: string; status?: string; summary?: string };
    };
    if (body.payload.type === "update_step") {
      current = {
        ...current,
        selectedInitiative: current.selectedInitiative
          ? {
              ...current.selectedInitiative,
              revision: current.selectedInitiative.revision + 1,
              currentMilestone: {
                ...current.selectedInitiative.currentMilestone,
                steps: current.selectedInitiative.currentMilestone.steps.map((step) =>
                  step.id === body.payload.stepId
                    ? { ...step, status: body.payload.status as never }
                    : step,
                ),
              },
            }
          : current.selectedInitiative,
      };
    }
    if (body.payload.type === "checkpoint_initiative" && current.selectedInitiative) {
      current = {
        ...current,
        selectedInitiative: {
          ...current.selectedInitiative,
          checkpoints: [
            ...current.selectedInitiative.checkpoints,
            {
              id: "cp-e2e",
              summary: body.payload.summary ?? "",
              createdAt: "2026-08-11T05:30:00Z",
            },
          ],
        },
      };
    }
    await route.fulfill({ json: { state: "completed", correlationId: "e2e", snapshot: current } });
  });
}

test.beforeEach(async ({ page }, testInfo) => {
  if (!isFixtureMode(testInfo.project)) return;
  const snapshot = isOrcaUnavailable(testInfo.project) ? unavailableSnapshot : liveSnapshot;
  await installFixture(page, snapshot);
});

test("authenticated bootstrap loads authoritative command center", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByRole("heading", { name: "Jcode Command Center" })).toBeVisible();
  await expect(page.getByLabel(/Connection/)).toBeVisible();
});

test("discovery route lists accessible initiatives @fixture-only", async ({ page }) => {
  await page.goto("/initiatives");
  await expect(page.getByRole("heading", { name: "Jcode Command Center" })).toBeVisible();
  await expect(
    page.getByText("Supervise durable initiatives beside live execution."),
  ).toBeVisible();
});

test("split route deep links to the selected run @fixture-only", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByRole("heading", { name: "Live execution" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Open run run-1" })).toHaveAttribute(
    "href",
    "/initiatives/init-command-center/runs/run-1",
  );
});

test("milestone step update posts command and installs replacement snapshot @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await page.getByRole("button", { name: "Mark Workspace frontend complete" }).click();
  await expect(page.getByText("Workspace frontendcompleted")).toBeVisible();
});

test("checkpoint command appends checkpoint history @fixture-only", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await page.getByLabel("Checkpoint summary").fill("E2E checkpoint");
  await page.getByRole("button", { name: "Checkpoint progress" }).click();
  await expect(page.getByText("E2E checkpoint")).toBeVisible();
});

test("schedule evidence and live event stream render @fixture-only", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByText("Last schedule evidence from fixture")).toBeVisible();
  await expect(page.getByText("Reconnect event applied")).toBeVisible();
});

test("disconnect, replay gap, snapshot replacement, and resume are visible @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByText("Reconnect event applied")).toBeVisible();
  await page.getByRole("button", { name: "Next events" }).click();
  await expect(page.getByText("Ordered event 41")).toBeVisible();
});

test("degraded Orca keeps durable actions and disables runtime controls @fixture-only", async ({
  page,
}, testInfo) => {
  if (!isOrcaUnavailable(testInfo.project)) return;
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByText("Orca runtime unavailable")).toBeVisible();
  await expect(page.getByRole("button", { name: "Retry" })).toBeDisabled();
  await expect(page.getByLabel("Checkpoint summary")).toBeEnabled();
});
