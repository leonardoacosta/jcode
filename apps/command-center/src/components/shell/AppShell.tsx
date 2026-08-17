import type { CommandCenterSnapshot } from "../../generated/command-center-contract";
import type { DecisionInboxSnapshot } from "../../generated/command-center-contract";
import { FindDrawer } from "./FindDrawer";
import { ConnectionBadge, MobileNavigation, SideNavigation } from "./navigation";
import { createSignal, onCleanup } from "solid-js";

export function AppShell(props: {
  snapshot?: CommandCenterSnapshot;
  decisionInbox?: DecisionInboxSnapshot;
  children: any;
  announcement?: string;
  activePath?: string;
}) {
  const [findOpen, setFindOpen] = createSignal(false);
  const [findTrigger, setFindTrigger] = createSignal<HTMLElement>();
  const activePath = () =>
    props.activePath ?? (typeof window === "undefined" ? "/inbox" : window.location.pathname);
  const connectionState = () => props.snapshot?.connection.state ?? "loading";
  const openFind = (trigger?: HTMLElement) => {
    setFindTrigger(
      trigger ??
        (document.activeElement instanceof HTMLElement ? document.activeElement : undefined),
    );
    setFindOpen(true);
    document.getElementById("global-find-query")?.focus();
  };
  const closeFind = () => {
    setFindOpen(false);
    findTrigger()?.focus();
  };

  const onGlobalKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      openFind();
    }
  };
  document.addEventListener("keydown", onGlobalKeyDown);
  onCleanup(() => document.removeEventListener("keydown", onGlobalKeyDown));

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
      <FindDrawer
        open={findOpen()}
        snapshot={props.snapshot}
        decisionInbox={props.decisionInbox}
        onClose={closeFind}
      />
    </div>
  );
}

export { ConnectionBadge };
