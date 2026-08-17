import { Router, Route, useLocation, useSearchParams } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  on,
  onCleanup,
  onMount,
  Show,
  untrack,
} from "solid-js";
import { AmbientActivity, AppShell, DecisionInbox, FindPage } from "./components/CommandCenter";
import type { CommandCenterSnapshot } from "./generated/command-center-contract";
import { createProjectionStore } from "./stores/projection";
import { HttpCommandCenterTransport } from "./transport/client";
import "./styles.css";

const transport = new HttpCommandCenterTransport();

export function loadFailureState(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("bootstrap_401") || message.includes("snapshot_401")) {
    return {
      title: "Authentication expired",
      message:
        "Your command-center browser session expired. Reload to request a fresh scoped session.",
    };
  }
  if (message.includes("snapshot_403")) {
    return {
      title: "Initiative forbidden",
      message: "This browser session is not allowed to inspect the requested initiative.",
    };
  }
  if (message.includes("snapshot_404")) {
    return {
      title: "Initiative not found",
      message: "No authoritative Jcode initiative exists for this route.",
    };
  }
  return {
    title: "Snapshot failed",
    message:
      "The route cannot obtain authoritative data. Use retry after authentication or daemon recovery.",
  };
}

function WorkspaceRoute() {
  const location = useLocation();
  const [searchParams] = useSearchParams<{ packet?: string; entry?: string }>();
  const path = () => location.pathname;
  const [hydrated, setHydrated] = createSignal(false);
  const [retryAttempt, setRetryAttempt] = createSignal(0);
  const store = createProjectionStore();
  onMount(() => setHydrated(true));
  const [snapshot] = createResource(
    () => (hydrated() ? { path: path(), retry: retryAttempt() } : undefined),
    async ({ path: currentPath }) => {
      const next = await transport.loadSnapshot(currentPath);
      store.installSnapshot(next);
      return next;
    },
  );
  const [decisionInbox] = createResource(
    () => (hydrated() ? retryAttempt() : undefined),
    async (attempt) => {
      if (attempt === undefined) return undefined;
      return transport.loadDecisionInbox();
    },
  );
  const retry = () => {
    setRetryAttempt((attempt) => attempt + 1);
  };
  const current = () => store.snapshot ?? snapshot();
  const routeState = () => {
    if (snapshot.error) return loadFailureState(snapshot.error);
    if (snapshot.loading || !hydrated()) return undefined;
    return null;
  };
  createEffect(
    on(
      () => current()?.meta.streamId,
      (streamId) => {
        if (!streamId) return;
        const sequence = untrack(() => current()?.meta.sequence);
        if (sequence === undefined) return;
        const unsubscribe = transport.subscribe(
          streamId,
          sequence,
          (event) => {
            const result = store.applyEvent(event);
            if (result === "snapshot_required") {
              void transport.loadSnapshot(path()).then((next) => store.installSnapshot(next));
            }
          },
          (state) => {
            if (state === "disconnected") store.markDisconnected("Event stream disconnected");
          },
        );
        onCleanup(unsubscribe);
      },
    ),
  );
  return (
    <AppShell
      snapshot={current()}
      decisionInbox={decisionInbox()}
      announcement={store.ui.announcement}
      activePath={path()}
    >
      <Show
        when={routeState() === null}
        fallback={
          <RouteState
            state={routeState()}
            loading={snapshot.loading || !hydrated()}
            onRetry={retry}
          />
        }
      >
        <Show when={path() === "/ambient"}>
          <AmbientRoute snapshot={current()} initialEntryId={searchParams.entry} />
        </Show>
        <Show when={path() === "/find"}>
          <FindRoute snapshot={current()} decisionInbox={decisionInbox()} />
        </Show>
        <Show when={path() !== "/ambient" && path() !== "/find"}>
          <DecisionInbox
            snapshot={decisionInbox()}
            loading={decisionInbox.loading}
            error={decisionInbox.error}
            initialRecordId={searchParams.packet ? Number(searchParams.packet) : undefined}
          />
        </Show>
      </Show>
    </AppShell>
  );
}

function RouteState(props: {
  state?: ReturnType<typeof loadFailureState> | null;
  loading: boolean;
  onRetry: () => void;
}) {
  return (
    <section class="page state-page" aria-live="polite">
      <Show
        when={!props.loading}
        fallback={
          <>
            <p class="eyebrow">Connecting</p>
            <h1>Loading Command Center</h1>
            <p>Requesting an authoritative snapshot from Jcode.</p>
          </>
        }
      >
        <p class="eyebrow">Route unavailable</p>
        <h1>{props.state?.title ?? "Snapshot failed"}</h1>
        <p>{props.state?.message ?? "The route could not obtain authoritative data."}</p>
        <button class="button primary" type="button" onClick={() => props.onRetry()}>
          Retry snapshot
        </button>
      </Show>
    </section>
  );
}

function AmbientRoute(props: { snapshot?: CommandCenterSnapshot; initialEntryId?: string }) {
  return <AmbientActivity snapshot={props.snapshot} initialEntryId={props.initialEntryId} />;
}

function FindRoute(props: {
  snapshot?: CommandCenterSnapshot;
  decisionInbox?: import("./generated/command-center-contract").DecisionInboxSnapshot;
}) {
  return <FindPage snapshot={props.snapshot} decisionInbox={props.decisionInbox} />;
}

export default function App() {
  return (
    <Router>
      <Route path="/" component={() => <WorkspaceRoute />} />
      <Route path="/inbox" component={WorkspaceRoute} />
      <Route path="/ambient" component={WorkspaceRoute} />
      <Route path="/find" component={WorkspaceRoute} />
      <Route path="/initiatives" component={WorkspaceRoute} />
      <Route path="/initiatives/:initiativeId" component={WorkspaceRoute} />
      <Route path="/initiatives/:initiativeId/runs/:runId" component={WorkspaceRoute} />
    </Router>
  );
}
