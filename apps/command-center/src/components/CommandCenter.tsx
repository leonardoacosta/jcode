import { For, Show, createMemo, createSignal } from "solid-js";
import type {
  CommandCenterSnapshot,
  InitiativeProjection,
  RunProjection,
  TimelineEvent,
} from "../generated/command-center-contract";

export function AppShell(props: {
  snapshot?: CommandCenterSnapshot;
  children: any;
  announcement?: string;
}) {
  return (
    <div class="app-shell">
      <a class="skip" href="#main">
        Skip to command center
      </a>
      <header>
        <a href="/initiatives" class="brand">
          Jcode Command Center
        </a>
        <ConnectionBadge
          state={props.snapshot?.connection.state ?? "loading"}
          reason={props.snapshot?.connection.reason}
        />
      </header>
      <div role="status" aria-live="polite" class="sr-only">
        {props.announcement}
      </div>
      <main id="main" tabindex="-1">
        {props.children}
      </main>
    </div>
  );
}

export function ConnectionBadge(props: { state: string; reason?: string }) {
  return (
    <div class={`badge ${props.state}`} aria-label={`Connection ${props.state}`}>
      {props.state}
      {props.reason ? `: ${props.reason}` : ""}
    </div>
  );
}

export function InitiativeList(props: { initiatives: InitiativeProjection[] }) {
  return (
    <section aria-labelledby="initiatives-title">
      <h1 id="initiatives-title">Initiatives</h1>
      <Show
        when={props.initiatives.length}
        fallback={
          <StateCard
            title="No initiatives"
            message="No accessible resumable or historical initiatives were returned by Jcode."
          />
        }
      >
        <div class="initiative-grid">
          <For each={props.initiatives}>
            {(initiative) => (
              <a class="initiative-card" href={`/initiatives/${initiative.id}`}>
                <h2>{initiative.title}</h2>
                <p>{initiative.outcome}</p>
                <dl>
                  <dt>Status</dt>
                  <dd>{initiative.status}</dd>
                  <dt>Milestone</dt>
                  <dd>{initiative.currentMilestone.title}</dd>
                  <dt>Freshness</dt>
                  <dd>{initiative.freshness}</dd>
                </dl>
                <Show when={initiative.blockers.length}>
                  <strong>{initiative.blockers.length} blocker(s)</strong>
                </Show>
              </a>
            )}
          </For>
        </div>
      </Show>
    </section>
  );
}

export function SplitWorkspace(props: {
  initiative: InitiativeProjection;
  run?: RunProjection;
  onCheckpoint: (summary: string) => void;
  onUpdateStep?: (stepId: string, status: "pending" | "running" | "blocked" | "completed") => void;
  pending?: boolean;
  failure?: string;
}) {
  const [durableWidth, setDurableWidth] = createSignal(48);
  return (
    <section
      class="workspace"
      style={{ "--durable-width": `${durableWidth()}%` }}
      aria-label="Split initiative and execution workspace"
    >
      <div class="pane durable">
        <DurablePane
          initiative={props.initiative}
          onCheckpoint={props.onCheckpoint}
          onUpdateStep={props.onUpdateStep}
          pending={props.pending}
          failure={props.failure}
        />
      </div>
      <div class="resizer">
        <label for="pane-size">Pane size</label>
        <input
          id="pane-size"
          type="range"
          min="35"
          max="65"
          value={durableWidth()}
          onInput={(event) => setDurableWidth(Number(event.currentTarget.value))}
        />
      </div>
      <div class="pane live">
        <ExecutionPane run={props.run} />
      </div>
    </section>
  );
}

export function DurablePane(props: {
  initiative: InitiativeProjection;
  onCheckpoint: (summary: string) => void;
  onUpdateStep?: (stepId: string, status: "pending" | "running" | "blocked" | "completed") => void;
  pending?: boolean;
  failure?: string;
}) {
  const [summary, setSummary] = createSignal("");
  return (
    <section aria-labelledby="durable-title">
      <h1 id="durable-title">{props.initiative.title}</h1>
      <p class="outcome">{props.initiative.outcome}</p>
      <StatusStrip label="Initiative freshness" value={props.initiative.freshness} />
      <h2>Current milestone</h2>
      <h3>{props.initiative.currentMilestone.title}</h3>
      <ul class="steps">
        <For each={props.initiative.currentMilestone.steps}>
          {(step) => (
            <li>
              <span class={`dot ${step.status}`} />
              {step.title}
              <small>{step.status}</small>
              <button
                type="button"
                disabled={!props.initiative.availableActions.updateMilestone || props.pending}
                onClick={() => props.onUpdateStep?.(step.id, "completed")}
              >
                Mark {step.title} complete
              </button>
            </li>
          )}
        </For>
      </ul>
      <SectionList
        title="Success criteria"
        items={props.initiative.successCriteria}
        empty="No success criteria recorded."
      />
      <SectionList title="Blockers" items={props.initiative.blockers} empty="No blockers." />
      <SectionList
        title="Next actions"
        items={props.initiative.nextActions}
        empty="No next actions."
      />
      <h2>Linked schedules</h2>
      <Show
        when={props.initiative.schedules.length}
        fallback={
          <StateCard
            title="No schedule linked"
            message="Jcode did not project a linked schedule for this initiative."
          />
        }
      >
        <For each={props.initiative.schedules}>
          {(schedule) => (
            <article class="schedule">
              <h3>{schedule.cadence}</h3>
              <p>
                {schedule.timezone} · {schedule.nextFire ?? "No next fire"}
              </p>
              <StatusStrip label="Schedule freshness" value={schedule.freshness} />
              <Show when={schedule.evidence}>
                <p>{schedule.evidence}</p>
              </Show>
            </article>
          )}
        </For>
      </Show>
      <h2>Checkpoint history</h2>
      <For each={props.initiative.checkpoints}>
        {(checkpoint) => (
          <article class="checkpoint">
            <time>{checkpoint.createdAt}</time>
            <p>{checkpoint.summary}</p>
          </article>
        )}
      </For>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          props.onCheckpoint(summary());
        }}
        aria-describedby={props.failure ? "checkpoint-error" : undefined}
      >
        <label for="checkpoint-summary">Checkpoint summary</label>
        <textarea
          id="checkpoint-summary"
          value={summary()}
          onInput={(event) => setSummary(event.currentTarget.value)}
          disabled={!props.initiative.availableActions.checkpoint || props.pending}
        />
        <button
          type="submit"
          disabled={
            !props.initiative.availableActions.checkpoint ||
            props.pending ||
            summary().trim().length === 0
          }
        >
          {props.pending ? "Checkpoint pending" : "Checkpoint progress"}
        </button>
        <Show when={props.failure}>
          <p id="checkpoint-error" role="alert" class="error">
            {props.failure} <button type="button">Inspect</button>
            <button type="button">Dismiss</button>
          </p>
        </Show>
      </form>
    </section>
  );
}

export function ExecutionPane(props: { run?: RunProjection }) {
  return (
    <section aria-labelledby="execution-title">
      <h2 id="execution-title">Live execution</h2>
      <Show
        when={props.run}
        fallback={
          <StateCard
            title="No linked run"
            message="No authoritative Jcode run is linked. The UI will not infer one from nearby activity."
          />
        }
        keyed
      >
        {(run) => (
          <>
            <StatusStrip label="Run health" value={run.health} />
            <p>
              Run {run.id} is {run.status}
            </p>
            <Show when={run.health === "unavailable" || run.health === "stale"}>
              <StateCard
                title="Orca runtime unavailable"
                message={`Last observed: ${run.lastObservedAt ?? "never"}. Unsafe runtime actions are disabled.`}
              />
            </Show>
            <div class="actions">
              <a
                href={`/initiatives/${run.initiativeId}/runs/${run.id}`}
                aria-label={`Open run ${run.id}`}
              >
                Open run
              </a>
              <button disabled={!run.availableActions.startRun}>Start</button>
              <button disabled={!run.availableActions.retryRun}>Retry</button>
              <button disabled={!run.availableActions.cancelRun}>Cancel</button>
            </div>
            <h3>Workers and sessions</h3>
            <For
              each={run.workers}
              fallback={
                <StateCard title="No workers" message="No linked Orca workers were projected." />
              }
            >
              {(worker) => (
                <article class="worker">
                  <strong>{worker.label}</strong>
                  <span>{worker.status}</span>
                  <Show when={worker.attention}>
                    <mark>{worker.attention}</mark>
                  </Show>
                </article>
              )}
            </For>
            <h3>Gates</h3>
            <For each={run.gates}>
              {(gate) => (
                <article class="gate">
                  <strong>{gate.title}</strong>
                  <span>{gate.status}</span>
                </article>
              )}
            </For>
            <VirtualTimeline events={run.timeline} />
          </>
        )}
      </Show>
    </section>
  );
}

export function VirtualTimeline(props: { events: TimelineEvent[] }) {
  const [start, setStart] = createSignal(0);
  const visible = createMemo(() => props.events.slice(start(), start() + 40));
  return (
    <section aria-labelledby="timeline-title">
      <h3 id="timeline-title">Event timeline</h3>
      <p>{props.events.length} ordered events</p>
      <Show when={props.events.length > 40}>
        <button onClick={() => setStart(Math.max(0, start() - 40))}>Previous events</button>
        <button onClick={() => setStart(Math.min(props.events.length - 40, start() + 40))}>
          Next events
        </button>
      </Show>
      <ol class="timeline" aria-label="Virtualized event timeline">
        <For each={visible()}>
          {(event) => (
            <li data-sequence={event.sequence}>
              <time>{event.timestamp}</time>
              <span>{event.source}</span>
              <p>{event.message}</p>
            </li>
          )}
        </For>
      </ol>
    </section>
  );
}

function SectionList(props: { title: string; items: string[]; empty: string }) {
  return (
    <section>
      <h2>{props.title}</h2>
      <Show when={props.items.length} fallback={<p class="muted">{props.empty}</p>}>
        <ul>
          <For each={props.items}>{(item) => <li>{item}</li>}</For>
        </ul>
      </Show>
    </section>
  );
}
function StatusStrip(props: { label: string; value: string }) {
  return (
    <p class={`status ${props.value}`} role="status">
      <span>{props.label}</span>: <strong>{props.value}</strong>
    </p>
  );
}
export function StateCard(props: { title: string; message: string }) {
  return (
    <article class="state-card">
      <h3>{props.title}</h3>
      <p>{props.message}</p>
    </article>
  );
}
