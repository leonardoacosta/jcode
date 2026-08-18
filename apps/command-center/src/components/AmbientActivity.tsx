import { For, Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import type { JSX } from "solid-js";
import type {
  Checkpoint,
  CommandCenterSnapshot,
  InitiativeProjection,
  RunProjection,
  ScheduleProjection,
  TimelineEvent,
} from "../generated/command-center-contract";

export type AmbientFilter = "all" | "running" | "paused" | "receipts";
export type AmbientEntryState = Exclude<AmbientFilter, "all">;

export interface AmbientLedgerEntry {
  id: string;
  occurredAt: string;
  source: string;
  title: string;
  summary: string;
  state: AmbientEntryState;
  stateLabel: string;
  evidence: string;
  owner: string;
  checkpoint: string;
  logs: string[];
}

const filterLabels: Array<[AmbientFilter, string]> = [
  ["all", "All"],
  ["running", "Running"],
  ["paused", "Paused"],
  ["receipts", "Receipts"],
];

function dateValue(value?: string) {
  if (!value) return 0;
  const parsed = Date.parse(value);
  return Number.isNaN(parsed) ? 0 : parsed;
}

function readableDate(value?: string) {
  if (!value) return "No time recorded";
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime())
    ? value
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
        parsed,
      );
}

function readableTime(value: string) {
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime())
    ? value
    : new Intl.DateTimeFormat(undefined, {
        hour: "2-digit",
        minute: "2-digit",
        hourCycle: "h23",
      }).format(parsed);
}

function latestCheckpoint(initiative: InitiativeProjection): Checkpoint | undefined {
  return [...initiative.checkpoints].sort(
    (left, right) => dateValue(right.createdAt) - dateValue(left.createdAt),
  )[0];
}

function timelineForRun(run: RunProjection | undefined) {
  return [...(run?.timeline ?? [])].sort((left, right) => right.sequence - left.sequence);
}

function runState(run: RunProjection, event: TimelineEvent): AmbientEntryState {
  if (event.severity === "error" || run.health === "unavailable" || run.status === "failed") {
    return "paused";
  }
  if (run.status === "running" || run.status === "canceling") return "running";
  return "receipts";
}

function stateLabel(state: AmbientEntryState) {
  if (state === "running") return "observing";
  if (state === "paused") return "needs attention";
  return "receipt retained";
}

function checkpointText(checkpoint: Checkpoint | undefined) {
  return checkpoint?.summary ?? "No retained checkpoint is available.";
}

function initiativeEntry(
  initiative: InitiativeProjection,
  checkpoint: Checkpoint | undefined,
): AmbientLedgerEntry {
  const completed = initiative.currentMilestone.steps.filter(
    (step) => step.status === "completed",
  ).length;
  const state: AmbientEntryState = initiative.blockers.length > 0 ? "paused" : "running";
  const stepEvidence = initiative.currentMilestone.steps
    .map((step) => step.evidence)
    .filter((evidence): evidence is string => Boolean(evidence));
  return {
    id: `initiative-${initiative.id}`,
    occurredAt: initiative.updatedAt,
    source: `initiative · ${initiative.id}`,
    title: initiative.title,
    summary: `${initiative.currentMilestone.title} · ${completed}/${initiative.currentMilestone.steps.length} steps complete`,
    state,
    stateLabel: stateLabel(state),
    evidence:
      initiative.blockers.join(" · ") ||
      stepEvidence.join(" · ") ||
      "Milestone state observed from Jcode.",
    owner: "Jcode initiative store",
    checkpoint: checkpointText(checkpoint),
    logs: [
      ...stepEvidence,
      `${initiative.currentMilestone.title} is ${initiative.currentMilestone.status}.`,
    ].slice(0, 4),
  };
}

function scheduleEntry(
  initiative: InitiativeProjection,
  schedule: ScheduleProjection,
  checkpoint: Checkpoint | undefined,
): AmbientLedgerEntry {
  const state: AmbientEntryState =
    schedule.lastResult === "completed"
      ? "receipts"
      : initiative.blockers.length > 0
        ? "paused"
        : "running";
  return {
    id: `schedule-${schedule.id}`,
    occurredAt: schedule.nextFire ?? initiative.updatedAt,
    source: `schedule · ${schedule.id}`,
    title: `Wake schedule · ${schedule.cadence}`,
    summary: `Next wake ${readableDate(schedule.nextFire)} · ${schedule.timezone}`,
    state,
    stateLabel: stateLabel(state),
    evidence: schedule.evidence ?? `Last result: ${schedule.lastResult ?? "not recorded"}.`,
    owner: "Jcode scheduler",
    checkpoint: checkpointText(checkpoint),
    logs: [
      schedule.evidence ?? "No schedule evidence recorded.",
      `Retry state: ${schedule.retryState ?? "not recorded"}.`,
    ],
  };
}

function checkpointEntry(
  initiative: InitiativeProjection,
  checkpoint: Checkpoint,
): AmbientLedgerEntry {
  return {
    id: `checkpoint-${checkpoint.id}`,
    occurredAt: checkpoint.createdAt,
    source: `checkpoint · ${checkpoint.id}`,
    title: checkpoint.summary,
    summary: `Retained checkpoint · ${readableDate(checkpoint.createdAt)}`,
    state: "receipts",
    stateLabel: stateLabel("receipts"),
    evidence: `Checkpoint ${checkpoint.id} retained by the initiative store.`,
    owner: "Jcode initiative store",
    checkpoint: checkpoint.summary,
    logs: [checkpoint.summary],
  };
}

function timelineEntry(
  initiative: InitiativeProjection,
  run: RunProjection,
  event: TimelineEvent,
  checkpoint: Checkpoint | undefined,
): AmbientLedgerEntry {
  const state = runState(run, event);
  return {
    id: `timeline-${event.id}`,
    occurredAt: event.timestamp,
    source: `${event.source} · ${run.id}`,
    title: event.message,
    summary: `Evidence event ${event.sequence} · ${initiative.title}`,
    state,
    stateLabel: stateLabel(state),
    evidence: `Severity ${event.severity} · sequence ${event.sequence}`,
    owner: run.workers[0]?.label ?? "Jcode runtime",
    checkpoint: checkpointText(checkpoint),
    logs: timelineForRun(run)
      .slice(0, 4)
      .map((item) => item.message),
  };
}

export function buildAmbientLedger(snapshot?: CommandCenterSnapshot): AmbientLedgerEntry[] {
  if (!snapshot) return [];
  const initiatives = snapshot.initiatives.length
    ? snapshot.initiatives
    : snapshot.selectedInitiative
      ? [snapshot.selectedInitiative]
      : [];
  const entries: AmbientLedgerEntry[] = [];

  for (const initiative of initiatives) {
    const checkpoint = latestCheckpoint(initiative);
    const run =
      snapshot.selectedRun?.initiativeId === initiative.id ? snapshot.selectedRun : undefined;
    entries.push(initiativeEntry(initiative, checkpoint));
    for (const schedule of initiative.schedules)
      entries.push(scheduleEntry(initiative, schedule, checkpoint));
    for (const retainedCheckpoint of initiative.checkpoints) {
      entries.push(checkpointEntry(initiative, retainedCheckpoint));
    }
    if (run) {
      for (const event of timelineForRun(run).slice(0, 6)) {
        entries.push(timelineEntry(initiative, run, event, checkpoint));
      }
    }
  }

  return entries.sort((left, right) => dateValue(right.occurredAt) - dateValue(left.occurredAt));
}

function AmbientDrawer(props: {
  open: boolean;
  id: string;
  title: string;
  eyebrow: string;
  describedBy: string;
  drawerRef: (element: HTMLElement) => void;
  onClose: () => void;
  onKeyDown: (event: KeyboardEvent) => void;
  children: JSX.Element;
}) {
  return (
    <>
      <button
        class={`ambient-drawer-backdrop${props.open ? " is-open" : ""}`}
        type="button"
        aria-label="Close drawer"
        aria-hidden={!props.open}
        hidden={!props.open}
        tabIndex={props.open ? 0 : -1}
        onClick={() => props.onClose()}
      />
      <aside
        id={props.id}
        ref={props.drawerRef}
        class={`ambient-drawer${props.open ? " is-open" : ""}`}
        role="dialog"
        aria-modal="true"
        aria-hidden={!props.open}
        hidden={!props.open}
        aria-labelledby={`${props.id}-title`}
        aria-describedby={props.describedBy}
        tabIndex={-1}
        onKeyDown={(event) => props.onKeyDown(event)}
      >
        <header class="ambient-drawer-head">
          <div>
            <p class="eyebrow">{props.eyebrow}</p>
            <h2 id={`${props.id}-title`}>{props.title}</h2>
          </div>
          <button class="drawer-close" type="button" onClick={() => props.onClose()}>
            Close
          </button>
        </header>
        <div class="ambient-drawer-body">{props.children}</div>
      </aside>
    </>
  );
}

export function AmbientActivity(props: {
  snapshot?: CommandCenterSnapshot;
  initialEntryId?: string;
}) {
  const [filter, setFilter] = createSignal<AmbientFilter>("all");
  const [drawer, setDrawer] = createSignal<"create" | AmbientLedgerEntry>();
  const [lastTrigger, setLastTrigger] = createSignal<HTMLElement>();
  let createDrawerElement: HTMLElement | undefined;
  let inspectDrawerElement: HTMLElement | undefined;
  let initialFocus: HTMLElement | undefined;

  const entries = createMemo(() => buildAmbientLedger(props.snapshot));
  const visibleEntries = createMemo(() =>
    entries().filter((entry) => filter() === "all" || entry.state === filter()),
  );
  const schedules = createMemo(() =>
    (props.snapshot?.initiatives ?? []).flatMap((initiative) => initiative.schedules),
  );
  const selectedEntry = createMemo(() => {
    const current = drawer();
    return typeof current === "object" ? current : undefined;
  });

  createEffect(() => {
    const entryId = props.initialEntryId;
    if (!entryId || drawer() || entries().length === 0) return;
    const entry = entries().find((candidate) => candidate.id === entryId);
    if (entry) setDrawer(entry);
  });

  const closeDrawer = () => {
    const trigger = lastTrigger();
    trigger?.focus();
    setDrawer(undefined);
  };

  const openDrawer = (next: "create" | AmbientLedgerEntry, event: MouseEvent) => {
    setLastTrigger(event.currentTarget as HTMLElement);
    const selector = next === "create" ? "#ambient-cycle-objective" : ".drawer-close";
    const drawerId = next === "create" ? "ambient-create-drawer" : "ambient-inspect-drawer";
    document.getElementById(drawerId)?.querySelector<HTMLElement>(selector)?.focus();
    setDrawer(next);
  };

  createEffect(() => {
    const open = drawer();
    if (!open) return;
    queueMicrotask(() => {
      const element = open === "create" ? createDrawerElement : inspectDrawerElement;
      initialFocus =
        element?.querySelector<HTMLElement>(
          open === "create" ? "#ambient-cycle-objective" : ".drawer-close",
        ) ?? undefined;
      initialFocus?.focus();
    });
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeDrawer();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  const trapFocus = (event: KeyboardEvent) => {
    if (event.key !== "Tab") return;
    const drawerElement = event.currentTarget as HTMLElement;
    const focusable = Array.from(
      drawerElement.querySelectorAll<HTMLElement>(
        "button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [href]",
      ),
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  return (
    <section class="page ambient-page" aria-labelledby="ambient-title">
      <header class="page-bar compact-bar">
        <div>
          <p class="eyebrow">{entries().length} evidence events</p>
          <h1 id="ambient-title">Ambient activity</h1>
        </div>
        <div class="page-actions">
          <button
            class="button"
            type="button"
            disabled={entries().length === 0}
            onClick={(event) => openDrawer(entries()[0], event)}
          >
            Latest log
          </button>
          <button
            class="button primary"
            type="button"
            aria-haspopup="dialog"
            aria-controls="ambient-create-drawer"
            onClick={(event) => openDrawer("create", event)}
          >
            New ambient cycle
          </button>
        </div>
      </header>

      <div class="toolbar ambient-toolbar" role="group" aria-label="Filter ambient activity">
        <For each={filterLabels}>
          {([kind, label]) => (
            <button
              class="tab"
              classList={{ active: filter() === kind }}
              type="button"
              aria-pressed={filter() === kind}
              onClick={() => setFilter(kind)}
            >
              {label}
            </button>
          )}
        </For>
        <span class="toolbar-note">
          Next wake ·{" "}
          {schedules()[0]?.nextFire ? readableDate(schedules()[0].nextFire) : "not scheduled"}
        </span>
      </div>

      <ul class="ambient-ledger" role="list" aria-label="Ambient activity ledger">
        <Show
          when={visibleEntries().length > 0}
          fallback={<li class="ambient-empty">No evidence events match this filter.</li>}
        >
          <For each={visibleEntries()}>
            {(entry, index) => (
              <li
                class="ambient-ledger-row"
                data-state={entry.state}
                style={{ "--ambient-index": index() }}
              >
                <time dateTime={entry.occurredAt}>{readableTime(entry.occurredAt)}</time>
                <span class="ambient-source">{entry.source}</span>
                <div class="ambient-entry-copy">
                  <strong>{entry.title}</strong>
                  <p>{entry.summary}</p>
                  <small>{entry.evidence}</small>
                </div>
                <span class={`ambient-state ${entry.state}`}>{entry.stateLabel}</span>
                <button
                  class="inline-action"
                  type="button"
                  aria-label={`Inspect ${entry.title}`}
                  onClick={(event) => openDrawer(entry, event)}
                >
                  Inspect
                </button>
              </li>
            )}
          </For>
        </Show>
      </ul>

      <section class="ambient-schedule" aria-labelledby="ambient-schedule-title">
        <div class="ambient-section-heading">
          <h2 id="ambient-schedule-title">Wake schedule</h2>
          <span>{schedules().length} linked</span>
        </div>
        <Show
          when={schedules().length > 0}
          fallback={<p class="ambient-empty">No authoritative wake schedule is linked.</p>}
        >
          <div class="ambient-schedule-list">
            <For each={schedules()}>
              {(schedule) => (
                <div class="ambient-schedule-row">
                  <strong>{schedule.cadence}</strong>
                  <span>{schedule.timezone}</span>
                  <span>
                    {schedule.nextFire ? readableDate(schedule.nextFire) : "No next wake"}
                  </span>
                  <span>{schedule.lastResult ?? "No result"}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </section>

      <AmbientDrawer
        open={drawer() === "create"}
        id="ambient-create-drawer"
        title="Create ambient cycle"
        eyebrow="Bounded workflow"
        describedBy="ambient-create-description"
        drawerRef={(element) => {
          createDrawerElement = element;
        }}
        onClose={closeDrawer}
        onKeyDown={trapFocus}
      >
        <p id="ambient-create-description" class="ambient-drawer-intro">
          Create remains unavailable until Jcode exposes an authoritative ambient-cycle command.
        </p>
        <form class="ambient-form" onSubmit={(event) => event.preventDefault()}>
          <label for="ambient-cycle-objective">Cycle objective</label>
          <input id="ambient-cycle-objective" type="text" placeholder="What should be observed?" />
          <label for="ambient-cycle-cadence">Wake cadence</label>
          <select id="ambient-cycle-cadence" value="every 30 minutes">
            <option>every 30 minutes</option>
            <option>hourly</option>
            <option>daily</option>
          </select>
          <p class="ambient-contract-note" role="status">
            Unavailable: ambient-cycle create contract is not available in the current transport. No
            cycle will be created.
          </p>
          <button class="button primary" type="submit" disabled>
            Create cycle
          </button>
        </form>
      </AmbientDrawer>

      <AmbientDrawer
        open={Boolean(selectedEntry())}
        id="ambient-inspect-drawer"
        title="Inspect ambient activity"
        eyebrow={selectedEntry()?.source ?? "Activity evidence"}
        describedBy="ambient-inspect-description"
        drawerRef={(element) => {
          inspectDrawerElement = element;
        }}
        onClose={closeDrawer}
        onKeyDown={trapFocus}
      >
        <Show when={selectedEntry()} keyed>
          {(entry) => (
            <>
              <p id="ambient-inspect-description" class="ambient-drawer-intro">
                {entry.summary}
              </p>
              <section class="ambient-inspect-block" aria-labelledby="ambient-latest-logs-title">
                <h3 id="ambient-latest-logs-title">Latest logs</h3>
                <ul class="ambient-log-list">
                  <For each={entry.logs}>{(log) => <li>{log}</li>}</For>
                </ul>
              </section>
              <section class="ambient-inspect-block" aria-labelledby="ambient-evidence-title">
                <h3 id="ambient-evidence-title">Evidence</h3>
                <p>{entry.evidence}</p>
              </section>
              <section class="ambient-inspect-block" aria-labelledby="ambient-checkpoint-title">
                <h3 id="ambient-checkpoint-title">Retained checkpoint</h3>
                <p>{entry.checkpoint}</p>
              </section>
              <section class="ambient-inspect-block" aria-labelledby="ambient-owner-title">
                <h3 id="ambient-owner-title">Owner trail</h3>
                <p>{entry.owner}</p>
                <p>{entry.source}</p>
              </section>
              <section class="ambient-inspect-actions" aria-label="Bounded activity actions">
                <button class="button primary" type="button" disabled>
                  Resume cycle
                </button>
                <p class="ambient-contract-note" role="status">
                  Unavailable: ambient-cycle resume contract is not available in the current
                  transport. No authority is inferred.
                </p>
              </section>
            </>
          )}
        </Show>
      </AmbientDrawer>
    </section>
  );
}
