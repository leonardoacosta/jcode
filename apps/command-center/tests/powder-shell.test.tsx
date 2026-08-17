import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AppShell } from "../src/components/CommandCenter";
import { liveSnapshot } from "./fixtures/snapshots";

describe("Powder command-center shell", () => {
  it("replaces the legacy header and split-workspace landmark with explicit shell landmarks", () => {
    render(() => (
      <AppShell snapshot={liveSnapshot}>
        <p>Route content</p>
      </AppShell>
    ));

    expect(screen.queryByRole("banner")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Split initiative and execution workspace" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("main")).toBeInTheDocument();
  });

  it("provides Powder side navigation, mobile navigation, and stable Inbox/Ambient routes", () => {
    render(() => (
      <AppShell snapshot={liveSnapshot}>
        <p>Route content</p>
      </AppShell>
    ));

    const sideNavigation = screen.getByRole("navigation", { name: "Command Center" });
    expect(sideNavigation).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Decision inbox/i })).toHaveAttribute("href", "/inbox");
    expect(screen.getByRole("link", { name: /Ambient cycles/i })).toHaveAttribute(
      "href",
      "/ambient",
    );
    expect(screen.getByRole("button", { name: /Find run or receipt/i })).toBeInTheDocument();

    const mobileNavigation = screen.getByRole("navigation", { name: "Mobile navigation" });
    expect(mobileNavigation).toBeInTheDocument();
    expect(mobileNavigation.querySelectorAll("a")).toHaveLength(2);
    expect(mobileNavigation.querySelector('a[href="/inbox"]')).toBeInTheDocument();
    expect(mobileNavigation.querySelector('a[href="/ambient"]')).toBeInTheDocument();
  });
});
