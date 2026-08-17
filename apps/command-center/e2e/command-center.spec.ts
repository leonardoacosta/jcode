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
  await expect(page.getByRole("heading", { name: "Decision queue" })).toBeVisible();
});

test("Decision Inbox renders durable provider provenance @fixture-only", async ({ page }) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await expect(page.getByText("Telegram · tg:42", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Review the Command Center delivery" }),
  ).toBeVisible();
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

test("discovery route opens the Decision Inbox primary view @fixture-only", async ({ page }) => {
  await page.goto("/initiatives");
  await expect(page.getByRole("heading", { name: "Decision queue" })).toBeVisible();
  await expect(page.getByRole("group", { name: "Filter by type" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Durable packets" })).toBeVisible();
});

test("packet selection opens the evidence detail pane @fixture-only", async ({ page }) => {
  await page.goto("/initiatives");
  await page.getByRole("button", { name: /Review the Command Center delivery packet/i }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Source" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Authority" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Blast radius and rollback" })).toBeVisible();
});

test("global Find opens as an accessible drawer from the selected run route @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");

  const trigger = page.getByRole("button", { name: /Find run or receipt/i });
  await expect(trigger).toHaveAttribute("aria-haspopup", "dialog");
  await expect(trigger).toHaveAttribute("aria-controls", "find-drawer");
  await trigger.click();

  const drawer = page.getByRole("dialog", { name: "Find run or receipt" });
  await expect(drawer).toBeVisible();
  await expect(page.getByRole("searchbox", { name: "Search durable references" })).toBeFocused();
});

test("global Find filters durable references and updates the result count @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await page.getByRole("button", { name: /Find run or receipt/i }).click();

  const query = page.getByRole("searchbox", { name: "Search durable references" });
  await query.fill("run-1");

  const drawer = page.getByRole("dialog", { name: "Find run or receipt" });
  await expect(page.getByText("1 result")).toBeVisible();
  await expect(drawer.locator("a.find-result").filter({ hasText: "run-1" })).toBeVisible();
  await expect(
    drawer.locator("a.find-result").filter({ hasText: "Jcode Command Center" }),
  ).toBeHidden();
});

test("global Find result links preserve initiative and run deep links @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives/init-command-center/runs/run-1");
  await page.getByRole("button", { name: /Find run or receipt/i }).click();

  const drawer = page.getByRole("dialog", { name: "Find run or receipt" });
  await expect(
    drawer.locator("a.find-result").filter({ hasText: "Jcode Command Center" }),
  ).toHaveAttribute("href", "/initiatives/init-command-center");
  await expect(drawer.locator("a.find-result").filter({ hasText: "run-1" })).toHaveAttribute(
    "href",
    "/initiatives/init-command-center/runs/run-1",
  );
});

test("Decision Inbox filters and sorts packets @fixture-only", async ({ page }) => {
  await page.goto("/initiatives");
  await page.getByRole("button", { name: "Approvals" }).click();
  await expect(
    page.getByRole("button", { name: /Review the Command Center delivery packet/i }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Questions" }).click();
  await expect(page.getByText("No packets match this filter.")).toBeVisible();
  await page.getByRole("button", { name: "All 1" }).click();
  await page.getByRole("combobox", { name: "Sort packets" }).selectOption("oldest");
  await expect(
    page.getByRole("button", { name: /Review the Command Center delivery packet/i }),
  ).toBeVisible();
});

test("Decision Inbox keeps bounded actions disabled when transport is unsupported @fixture-only", async ({
  page,
}) => {
  await page.goto("/initiatives");
  await page.getByRole("button", { name: /Review the Command Center delivery packet/i }).click();
  await expect(page.getByRole("button", { name: "Approve delivery" })).toBeDisabled();
  await expect(page.getByText(/unsupported by the current inbox transport/i)).toBeVisible();
});

test("Decision Inbox keeps the detail sheet reachable on mobile @fixture-only", async ({
  page,
}) => {
  await page.setViewportSize({ width: 520, height: 720 });
  await page.goto("/initiatives");
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("button", { name: "Back to queue" })).toBeVisible();
  await expect(page.getByRole("list", { name: "Durable decision packets" })).toBeVisible();
});

test("Ambient activity keeps evidence content ahead of controls @fixture-only", async ({
  page,
}) => {
  await page.goto("/ambient");

  await expect(page.getByRole("heading", { name: "Ambient activity" })).toBeVisible();
  await expect(page.getByRole("list", { name: "Ambient activity ledger" })).toBeVisible();
  await expect(page.getByText("Wake schedule · every 30 minutes")).toBeVisible();
  await expect(page.getByText("Frontend route established")).toBeVisible();

  await page.getByRole("button", { name: "Receipts" }).click();
  await expect(page.getByText("Wake schedule · every 30 minutes")).toBeVisible();
  await expect(page.getByText("Frontend route established")).toBeVisible();
  await expect(page.getByText("Jcode Command Center")).toBeHidden();

  await page.getByRole("button", { name: "Paused" }).click();
  await expect(page.getByText("Jcode Command Center")).toBeVisible();
});

test("Ambient create and inspect drawers are accessible and fail closed @fixture-only", async ({
  page,
}) => {
  await page.goto("/ambient");

  const createTrigger = page.getByRole("button", { name: "New ambient cycle" });
  await createTrigger.click();
  const createDrawer = page.getByRole("dialog", { name: "Create ambient cycle" });
  await expect(createDrawer).toBeVisible();
  await expect(page.getByLabel("Cycle objective")).toBeFocused();
  await expect(page.getByRole("button", { name: "Create cycle" })).toBeDisabled();
  await expect(page.getByText(/ambient-cycle create contract is not available/i)).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(createDrawer).toBeHidden();
  await expect(createTrigger).toBeFocused();

  await page.getByRole("button", { name: /Inspect Wake schedule/ }).click();
  const inspectDrawer = page.getByRole("dialog", { name: "Inspect ambient activity" });
  await expect(inspectDrawer).toBeVisible();
  await expect(inspectDrawer.getByRole("heading", { name: "Latest logs" })).toBeVisible();
  await expect(inspectDrawer.getByRole("heading", { name: "Evidence" })).toBeVisible();
  await expect(inspectDrawer.getByRole("heading", { name: "Retained checkpoint" })).toBeVisible();
  await expect(inspectDrawer.getByRole("heading", { name: "Owner trail" })).toBeVisible();
  await expect(inspectDrawer.getByRole("button", { name: "Resume cycle" })).toBeDisabled();
  await expect(page.getByText(/ambient-cycle resume contract is not available/i)).toBeVisible();
});
