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
  await page.route("**/api/command-center/bootstrap**", async (route) =>
    route.fulfill({
      json: {
        id: "fixture-session",
        csrf_token: "fixture-csrf",
        expires_at: "2099-01-01T00:00:00Z",
        scope: [],
      },
    }),
  );
  await page.route("**/api/command-center/initiatives**", async (route) => {
    const requestPath = new URL(route.request().url()).pathname;
    const response =
      requestPath === "/api/command-center/initiatives" && current.selectedInitiative
        ? {
            ...current,
            initiatives: [current.selectedInitiative],
            selectedInitiative: undefined,
            selectedRun: undefined,
          }
        : current;
    await route.fulfill({ json: response });
  });
  await page.route("**/api/command-center/decision-inbox", async (route) =>
    route.fulfill({
      json: {
        generated_at: "2026-08-17T05:00:00Z",
        items: [
          {
            record_id: 1,
            source: {
              adapter: "telegram",
              sender_identity: "operator",
              conversation: "tg:42",
            },
            received_at: "2026-08-17T05:00:00Z",
            content: "Review the Command Center delivery",
            category: "work_request",
            status: "awaiting_approval",
            proposal: { id: 1, state: "awaiting_approval" },
            dedupe_key: "sha256:fixture",
            duplicate_deliveries: 0,
            retry_deliveries: 0,
            redacted: false,
            raw_payload_retained: true,
          },
        ],
      },
    }),
  );
  let eventDelivered = false;
  await page.route("**/api/command-center/replay**", async (route) => {
    const events = eventDelivered ? [] : [nextEvent];
    eventDelivered = true;
    await route.fulfill({ json: { events, snapshot_required: false } });
  });
  await page.route("**/api/command-center/commands", async (route) => {
    const body = route.request().postDataJSON() as {
      payload: {
        type: string;
        step_id?: string;
        status?: string;
        summary?: string;
      };
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
                  step.id === body.payload.step_id
                    ? { ...step, status: body.payload.status as never }
                    : step,
                ),
              },
            }
          : current.selectedInitiative,
      };
    }
    if (body.payload.type === "checkpoint" && current.selectedInitiative) {
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
  await expect(
    page.getByRole("banner").getByRole("heading", { name: "Jcode Command Center" }),
  ).toBeVisible();
  await expect(page.getByLabel(/Connection/)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Decision Inbox" })).toBeVisible();
});

test("Decision Inbox renders durable provider provenance @fixture-only", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByText("Telegram")).toBeVisible();
  await expect(page.getByText("Review the Command Center delivery")).toBeVisible();
  await expect(page.getByText("Work request")).toBeVisible();
  await expect(page.getByText("Awaiting approval")).toBeVisible();
});

test("live Telegram message reaches the authenticated Decision Inbox", async ({ page }) => {
  test.skip(
    process.env.JCODE_EXPECT_LIVE_TELEGRAM !== "1",
    "requires the credential-gated live Telegram acceptance database",
  );
  const inboxResponse = page.waitForResponse((response) =>
    response.url().endsWith("/api/command-center/decision-inbox"),
  );
  await page.goto("/initiatives/init-command-center/runs/run-1");
  const response = await inboxResponse;
  expect(response.ok()).toBe(true);
  const inbox = (await response.json()) as {
    items: Array<{ source: { adapter: string }; content?: string }>;
  };
  expect(inbox.items.some((item) => item.source.adapter === "telegram")).toBe(true);
  expect(inbox.items.some((item) => /inbox acceptance ping/i.test(item.content ?? ""))).toBe(true);
  await expect(page.getByText("Telegram").first()).toBeVisible();
  await expect(page.getByText(/inbox acceptance ping/i).first()).toBeVisible();
});

test("discovery route lists accessible initiatives @fixture-only", async ({ page }) => {
  await page.goto("/initiatives");
  await expect(
    page.getByRole("banner").getByRole("heading", { name: "Jcode Command Center" }),
  ).toBeVisible();
  await expect(
    page.getByText("Supervise durable initiatives beside live execution."),
  ).toBeVisible();
});

test("discovery navigation renders the selected initiative workspace @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives");
  await page.locator("a.initiative-card").click();
  await expect(page).toHaveURL(/\/initiatives\/init-command-center$/);
  await expect(
    page.getByRole("region", { name: "Split initiative and execution workspace" }),
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

test("accessibility, keyboard resize, embedded width, and virtualization hold @fixture-only", async ({
  page,
}) => {
  await page.setViewportSize({ width: 520, height: 720 });
  await page.goto("/initiatives/init-command-center/runs/run-1");

  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to command center" })).toBeFocused();
  await page.getByLabel("Pane size").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByLabel("Pane size")).toHaveJSProperty("value", "49");
  await expect(page.getByLabel("Virtualized event timeline").locator("li")).toHaveCount(40);

  const columns = await page
    .getByLabel("Split initiative and execution workspace")
    .evaluate((element) => getComputedStyle(element).gridTemplateColumns);
  expect(columns.trim().split(/\s+/)).toHaveLength(1);
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
