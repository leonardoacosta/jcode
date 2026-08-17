import { Router, Route, useLocation } from "@solidjs/router";
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
import {
  AppShell,
  DecisionInbox,
  InitiativeList,
  SplitWorkspace,
  StateCard,
} from "./components/CommandCenter";
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
  const path = () => location.pathname;
  const [hydrated, setHydrated] = createSignal(false);
  const [failure, setFailure] = createSignal<string>();
  const [loadError, setLoadError] = createSignal<unknown>();
  const [pending, setPending] = createSignal(false);
  const store = createProjectionStore();
  onMount(() => setHydrated(true));
  const [snapshot] = createResource(
    () => (hydrated() ? path() : undefined),
    async (currentPath) => {
      try {
        const next = await transport.loadSnapshot(currentPath);
        store.installSnapshot(next);
        setLoadError(undefined);
        return next;
      } catch (error) {
        setLoadError(error);
        return undefined;
      }
    },
  );
  const [decisionInbox] = createResource(hydrated, async (ready) => {
    if (!ready) return undefined;
    try {
      return await transport.loadDecisionInbox();
    } catch {
      return undefined;
    }
  });
  const current = () => store.snapshot ?? snapshot();
  const checkpoint = async (summary: string) => {
    const initiative = current()?.selectedInitiative;
    if (!initiative) return;
    setPending(true);
    setFailure(undefined);
    const result = await transport.sendCommand({
      idempotencyKey: crypto.randomUUID(),
      payload: {
        type: "checkpoint_initiative",
        initiativeId: initiative.id,
        expectedRevision: initiative.revision,
        summary,
        blockers: initiative.blockers,
        nextActions: initiative.nextActions,
      },
    });
    setPending(false);
    if (result.snapshot) store.installSnapshot(result.snapshot);
    if (result.state === "failed") setFailure(result.error?.message ?? "Command failed");
  };
  const updateStep = async (
    stepId: string,
    status: "pending" | "running" | "blocked" | "completed",
  ) => {
    const initiative = current()?.selectedInitiative;
    if (!initiative) return;
    setPending(true);
    setFailure(undefined);
    const result = await transport.sendCommand({
      idempotencyKey: crypto.randomUUID(),
      payload: {
        type: "update_step",
        initiativeId: initiative.id,
        expectedRevision: initiative.revision,
        milestoneId: initiative.currentMilestone.id,
        stepId,
        status,
      },
    });
    setPending(false);
    if (result.snapshot) store.installSnapshot(result.snapshot);
    if (result.state === "failed") setFailure(result.error?.message ?? "Command failed");
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
    <AppShell snapshot={current()} announcement={store.ui.announcement} activePath={path()}>
      <Show when={path() === "/ambient"}>
        <AmbientRoute />
      </Show>
      <Show when={path() === "/find"}>
        <FindRoute />
      </Show>
      <Show when={path() !== "/ambient" && path() !== "/find"}>
        <DecisionInbox snapshot={decisionInbox()} />
        <Show
          when={!snapshot.loading}
          fallback={
            <StateCard
              title="Loading authoritative snapshot"
              message="Jcode is loading a scoped command-center snapshot."
            />
          }
        >
          <Show when={!loadError()} fallback={<StateCard {...loadFailureState(loadError())} />}>
            <Show
              when={current()?.selectedInitiative}
              fallback={<InitiativeList initiatives={current()?.initiatives ?? []} />}
              keyed
            >
              {(initiative) => (
                <SplitWorkspace
                  initiative={initiative}
                  run={current()?.selectedRun}
                  onCheckpoint={checkpoint}
                  onUpdateStep={updateStep}
                  pending={pending()}
                  failure={failure()}
                />
              )}
            </Show>
          </Show>
        </Show>
      </Show>
    </AppShell>
  );
}

function AmbientRoute() {
  return (
    <section class="page" aria-labelledby="ambient-title">
      <header class="page-bar">
        <div>
          <p class="eyebrow">Background work</p>
          <h1 id="ambient-title">Ambient activity</h1>
        </div>
        <p>Stable route boundary for observed cycles and retained wake evidence.</p>
      </header>
      <StateCard
        title="Ambient workflow boundary"
        message="Ambient cycle details will be added behind this stable route interface."
      />
    </section>
  );
}

function FindRoute() {
  return (
    <section class="page" aria-labelledby="find-route-title">
      <header class="page-bar">
        <div>
          <p class="eyebrow">Global lookup</p>
          <h1 id="find-route-title">Find run or receipt</h1>
        </div>
        <p>Use the global lookup control to search durable initiative and run references.</p>
      </header>
      <StateCard
        title="Find workflow boundary"
        message="Search results and receipt inspection will be added behind this stable interface."
      />
    </section>
  );
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
