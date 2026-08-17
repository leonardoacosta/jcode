import type { CommandCenterSnapshot } from "../../generated/command-center-contract";
import { FindDrawer } from "./FindDrawer";
import { ConnectionBadge, MobileNavigation, SideNavigation } from "./navigation";
import { createSignal } from "solid-js";

export function AppShell(props: {
  snapshot?: CommandCenterSnapshot;
  children: any;
  announcement?: string;
  activePath?: string;
}) {
  const [findOpen, setFindOpen] = createSignal(false);
  const activePath = () =>
    props.activePath ?? (typeof window === "undefined" ? "/inbox" : window.location.pathname);
  const connectionState = () => props.snapshot?.connection.state ?? "loading";
  const openFind = () => {
    setFindOpen(true);
    document.getElementById("global-find-query")?.focus();
  };

  return (
    <div class="app-shell">
      <a class="skip" href="#main">
        Skip to command center
      </a>
      <SideNavigation
        activePath={activePath()}
        connectionState={connectionState()}
        connectionReason={props.snapshot?.connection.reason}
        onFind={openFind}
      />
      <div role="status" aria-live="polite" class="sr-only">
        {props.announcement}
      </div>
      <main id="main" tabindex="-1">
        {props.children}
      </main>
      <MobileNavigation activePath={activePath()} />
      <FindDrawer open={findOpen()} snapshot={props.snapshot} onClose={() => setFindOpen(false)} />
    </div>
  );
}

export { ConnectionBadge };
