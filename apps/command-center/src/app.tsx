import { Router, Route } from "@solidjs/router";
import { createEffect, createResource, createSignal, on, onCleanup, Show, untrack } from "solid-js";
import { AppShell, InitiativeList, SplitWorkspace, StateCard } from "./components/CommandCenter";
import { createProjectionStore } from "./stores/projection";
import { HttpCommandCenterTransport } from "./transport/client";
import "./styles.css";

const transport = new HttpCommandCenterTransport();

function WorkspaceRoute() {
  const path = typeof location === "undefined" ? "/initiatives" : location.pathname;
  const [failure, setFailure] = createSignal<string>();
  const [pending, setPending] = createSignal(false);
  const store = createProjectionStore();
  const [snapshot] = createResource(async () => {
    const next = await transport.loadSnapshot(path);
    store.installSnapshot(next);
    return next;
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
              void transport.loadSnapshot(path).then((next) => store.installSnapshot(next));
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
    <AppShell snapshot={current()} announcement={store.ui.announcement}>
      <Show
        when={!snapshot.loading}
        fallback={
          <StateCard
            title="Loading authoritative snapshot"
            message="Jcode is loading a scoped command-center snapshot."
          />
        }
      >
        <Show
          when={!snapshot.error}
          fallback={
            <StateCard
              title="Snapshot failed"
              message="The route cannot obtain authoritative data. Use retry after authentication or daemon recovery."
            />
          }
        >
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
    </AppShell>
  );
}

export default function App() {
  return (
    <Router>
      <Route path="/" component={() => <WorkspaceRoute />} />
      <Route path="/initiatives" component={WorkspaceRoute} />
      <Route path="/initiatives/:initiativeId" component={WorkspaceRoute} />
      <Route path="/initiatives/:initiativeId/runs/:runId" component={WorkspaceRoute} />
    </Router>
  );
}
