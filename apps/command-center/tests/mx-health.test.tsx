import { fireEvent, render, screen } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { MxHealthPage, topologyChecks } from "../src/components/MxHealth";
import { HttpCommandCenterTransport } from "../src/transport/client";
import {
  adapterFailure,
  degradedHealth,
  downHealth,
  healthyHealth,
  invalidContractProjection,
  liveProjection,
  staleProjection,
  timeoutProjection,
  unauthorizedProjection,
  unconfiguredProjection,
  unreachableProjection,
} from "./fixtures/mx-health";

const failureCases = [
  ["unauthorized", unauthorizedProjection, "Upstream unauthorized"],
  ["unreachable", unreachableProjection, "Upstream unreachable"],
  ["timeout", timeoutProjection, "Read timed out"],
  ["invalid contract", invalidContractProjection, "Invalid contract"],
] as const;

describe("MX health page", () => {
  it("renders the loading and unconfigured states without configuration values", () => {
    render(() => <MxHealthPage loading />);
    expect(screen.getByRole("heading", { name: "Loading MX health" })).toBeVisible();

    render(() => <MxHealthPage projection={unconfiguredProjection} />);
    expect(screen.getByRole("heading", { name: "MX health is not configured" })).toBeVisible();
    expect(screen.getByText(/setup required/i)).toBeVisible();
    expect(screen.queryByText(/example\.invalid|token|bearer/i)).not.toBeInTheDocument();
  });

  it.each([
    ["healthy", healthyHealth, "Healthy"],
    ["degraded", degradedHealth, "Degraded"],
    ["authoritative down/503", downHealth, "Down"],
  ] as const)(
    "renders the %s authority with semantic and topology surfaces",
    (_name, health, label) => {
      render(() => <MxHealthPage projection={liveProjection(health)} />);

      expect(screen.getByRole("heading", { name: "MX health" })).toBeVisible();
      expect(screen.getByText(label, { selector: ".mx-overall-label" })).toBeVisible();
      expect(screen.getByRole("img", { name: /MX health dependency topology/ })).toBeVisible();
      expect(screen.getByRole("list", { name: "MX health checks" })).toBeVisible();
      for (const check of health.checks) {
        expect(screen.getByRole("button", { name: new RegExp(check.id) })).toBeVisible();
        expect(screen.getByText(check.id, { selector: "strong" })).toBeVisible();
      }
    },
  );

  it("keeps provider availability separate from down persistence and blocked workflows", () => {
    render(() => <MxHealthPage projection={liveProjection(downHealth)} />);

    expect(screen.getByRole("button", { name: /source\.gmail/ })).toHaveTextContent("OK");
    expect(screen.getAllByText("Down").length).toBeGreaterThan(1);
    expect(screen.getAllByText("Blocked").length).toBeGreaterThan(0);
    expect(screen.getByText(/persistence\.sqlite is down/i)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /workflow\.inbox/ }));
    expect(screen.getByText("persistence.sqlite", { selector: "code" })).toBeVisible();
  });

  it("labels stale data with the current adapter failure and exposes read-only retry", () => {
    const onRetry = vi.fn();
    render(() => <MxHealthPage projection={staleProjection} onRetry={onRetry} />);

    expect(screen.getAllByText("Stale last-known-good").length).toBeGreaterThan(0);
    expect(screen.getByText(/bounded read timed out/i)).toBeVisible();
    fireEvent.click(screen.getAllByRole("button", { name: "Retry read" })[0]);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it.each(failureCases)(
    "renders the %s adapter failure with safe retry copy",
    (_name, projection, label) => {
      const onRetry = vi.fn();
      render(() => <MxHealthPage projection={projection} onRetry={onRetry} />);

      expect(screen.getByText(label)).toBeVisible();
      expect(screen.getAllByRole("button", { name: "Retry read" }).length).toBeGreaterThan(0);
      fireEvent.click(screen.getAllByRole("button", { name: "Retry read" })[0]);
      expect(onRetry).toHaveBeenCalledTimes(1);
    },
  );

  it("supports keyboard selection and keeps details synchronized with the semantic list", () => {
    render(() => <MxHealthPage projection={liveProjection(healthyHealth)} />);

    const source = screen.getByRole("button", { name: /source\.gmail/ });
    const persistence = screen.getByRole("button", { name: /persistence\.sqlite/ });
    source.focus();
    fireEvent.keyDown(source, { key: "ArrowDown" });

    expect(persistence).toHaveFocus();
    expect(persistence).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("persistence_ready", { selector: "code" })).toBeVisible();
  });

  it("preserves the selected check and details on passive refresh", () => {
    const Wrapper = () => {
      const [projection, setProjection] = createSignal(liveProjection(healthyHealth));
      return (
        <>
          <button type="button" onClick={() => setProjection(liveProjection(degradedHealth))}>
            Install refresh
          </button>
          <MxHealthPage projection={projection()} />
        </>
      );
    };
    render(() => <Wrapper />);

    fireEvent.click(screen.getByRole("button", { name: /persistence\.sqlite/ }));
    fireEvent.click(screen.getByRole("button", { name: "Install refresh" }));

    expect(screen.getByRole("button", { name: /persistence\.sqlite/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByText("persistence_ready", { selector: "code" })).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(/MX health refreshed/);
  });

  it("derives topology edges only from declared dependencies", () => {
    const topology = topologyChecks(healthyHealth.checks);
    expect(topology.edges).toHaveLength(1);
    expect(topology.edges[0].from.check.id).toBe("persistence.sqlite");
    expect(topology.edges[0].to.check.id).toBe("workflow.inbox");
  });

  it("keeps the browser boundary read-only and secret-safe", () => {
    const componentSource = readFileSync("src/components/MxHealth.tsx", "utf8");
    const transportSource = readFileSync("src/transport/client.ts", "utf8");
    const stylesheet = readFileSync("src/styles.css", "utf8");

    expect(componentSource).not.toMatch(/restart|reconnect|credential-refresh|database-repair/i);
    expect(componentSource).not.toMatch(/fetch\s*\(/);
    expect(transportSource).not.toMatch(/mx-health[\s\S]{0,240}method:\s*["']POST/i);
    expect(transportSource).not.toMatch(/mx-health[\s\S]{0,240}x-csrf-token/i);
    expect(stylesheet).toContain("@media (forced-colors: active)");
    expect(stylesheet).toContain("@media (prefers-reduced-motion: reduce)");
    expect(stylesheet).toContain("overflow-x: clip");
  });

  it("uses the Jcode read endpoint and never calls MX directly", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockImplementation(async (input) => {
      if (String(input).includes("/bootstrap")) {
        return new Response(
          JSON.stringify({ id: "session", csrf_token: "csrf", expires_at: "2999-01-01T00:00:00Z" }),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify(liveProjection()), { status: 200 });
    });
    const transport = new HttpCommandCenterTransport();
    await transport.loadMxHealth();
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/command-center/mx-health",
      expect.objectContaining({ credentials: "same-origin" }),
    );
    const mxCall = fetchMock.mock.calls.find(([url]) => url === "/api/command-center/mx-health");
    expect(mxCall?.[0]).not.toMatch(/health\/v1|example\.invalid/);
    fetchMock.mockRestore();
  });

  it("keeps the invalid-contract fixture distinct from transport failures", () => {
    expect(invalidContractProjection.adapterState).toBe("invalid_contract");
    expect(adapterFailure("invalid_contract", "invalid_contract").health).toBeUndefined();
    expect(unreachableProjection.adapterState).toBe("unreachable");
    expect(timeoutProjection.adapterState).toBe("timeout");
    expect(unauthorizedProjection.adapterState).toBe("unauthorized");
  });
});
