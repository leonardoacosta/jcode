import { For, Show, createMemo, createSignal } from "solid-js";
import type {
  DecisionInboxSnapshot,
  DecisionInboxItem,
  InitiativeProjection,
  RunProjection,
  TimelineEvent,
} from "../generated/command-center-contract";

export { AppShell } from "./shell/AppShell";
export { FindDrawer } from "./shell/FindDrawer";

const label = (value: string) =>
  value.replaceAll("_", " ").replace(/^./, (first) => first.toUpperCase());

type InboxFilter = "all" | "approval" | "question" | "revisit" | "receipt";

const filterLabels: Array<[InboxFilter, string]> = [
  ["all", "All"],
  ["approval", "Approvals"],
  ["question", "Questions"],
  ["revisit", "Revisits"],
  ["receipt", "Receipts"],
];

function packetKind(item: DecisionInboxItem): Exclude<InboxFilter, "all"> {
  if (item.status === "awaiting_approval" || item.proposal) return "approval";
  if (item.category === "status_request") return "question";
  if (
    item.status === "deferred" ||
    item.status === "unrecognized" ||
    item.status === "classification_failed" ||
    item.category === "unrecognized"
  ) {
    return "revisit";
  }
  return "receipt";
}

function packetDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function packetEvidence(item: DecisionInboxItem) {
  const delivery = [
    item.duplicateDeliveries > 0
      ? `${item.duplicateDeliveries} duplicate ${item.duplicateDeliveries === 1 ? "delivery" : "deliveries"} retained`
      : undefined,
    item.retryDeliveries > 0
      ? `${item.retryDeliveries} retry ${item.retryDeliveries === 1 ? "delivery" : "deliveries"}`
      : undefined,
  ].filter(Boolean);
  return delivery.length > 0 ? delivery.join(" · ") : "Single delivery recorded";
}

export function DecisionInbox(props: { snapshot?: DecisionInboxSnapshot }) {
  const [filter, setFilter] = createSignal<InboxFilter>("all");
  const [sort, setSort] = createSignal<"newest" | "oldest">("newest");
  const [selectedId, setSelectedId] = createSignal<number | undefined>(
    props.snapshot?.items[0]?.recordId,
  );
  const items = () => props.snapshot?.items ?? [];
  const visiblePackets = createMemo(() => {
    const filtered = items().filter((item) => filter() === "all" || packetKind(item) === filter());
    return filtered.sort((left, right) => {
      const direction = sort() === "newest" ? -1 : 1;
      return direction * (Date.parse(left.receivedAt) - Date.parse(right.receivedAt));
    });
  });
  const selectedPacket = createMemo(
    () => visiblePackets().find((item) => item.recordId === selectedId()) ?? visiblePackets()[0],
  );
  const countFor = (kind: InboxFilter) =>
    kind === "all" ? items().length : items().filter((item) => packetKind(item) === kind).length;

  return (
    <section aria-labelledby="decision-inbox-title" class="decision-inbox">
      <header class="page-bar compact-bar">
        <div>
          <p class="eyebrow">{items().length} open</p>
          <h1 id="decision-inbox-title">Decision queue</h1>
        </div>
        <span class="toolbar-note">
          Updated {props.snapshot ? packetDate(props.snapshot.generatedAt) : "when connected"}
        </span>
      </header>

      <div class="toolbar queue-toolbar" aria-label="Decision queue controls">
        <div class="filter-group" role="group" aria-label="Filter by type">
          <For each={filterLabels}>
            {([kind, name]) => (
              <button
                class={`tab ${filter() === kind ? "active" : ""}`}
                type="button"
                aria-pressed={filter() === kind}
                onClick={() => setFilter(kind)}
              >
                {name}
                <Show when={kind === "all"}> {countFor(kind)}</Show>
              </button>
            )}
          </For>
        </div>
        <label class="sort-control">
          Sort
          <select
            aria-label="Sort packets"
            value={sort()}
            onChange={(event) => setSort(event.currentTarget.value as "newest" | "oldest")}
          >
            <option value="newest">Newest first</option>
            <option value="oldest">Oldest first</option>
          </select>
        </label>
      </div>

      <div class="workspace inbox-workspace">
        <section class="surface packet-surface" aria-labelledby="packet-list-title">
          <div class="surface-head">
            <h2 id="packet-list-title">Durable packets</h2>
            <span>Telegram-backed</span>
          </div>
          <div class="inbox-list staggered-list" role="list" aria-label="Durable decision packets">
            <Show
              when={visiblePackets().length > 0}
              fallback={<p class="empty-line">No packets match this filter.</p>}
            >
              <For each={visiblePackets()}>
                {(item, index) => (
                  <button
                    type="button"
                    class={`inbox-item ${selectedPacket()?.recordId === item.recordId ? "active" : ""}`}
                    style={{ "--packet-index": index() }}
                    aria-label={`${item.content ?? "Untitled"} packet`}
                    aria-pressed={selectedPacket()?.recordId === item.recordId}
                    onClick={() => setSelectedId(item.recordId)}
                  >
                    <i class={`signal ${packetKind(item)}`} aria-hidden="true" />
                    <span>
                      <span class="item-type">
                        <span>{item.category ? label(item.category) : "Unclassified"}</span> ·{" "}
                        <span>{label(packetKind(item))}</span>
                      </span>
                      <h3 aria-hidden="true">{item.content ?? "No text content"}</h3>
                      <p>
                        {item.source.senderIdentity} · {item.source.conversation}
                      </p>
                      <span class="meta">
                        <span>{label(item.status)}</span>
                        <span>{packetEvidence(item)}</span>
                        <span>{item.redacted ? "Content redacted" : "Content retained"}</span>
                      </span>
                    </span>
                    <time class="item-time" dateTime={item.receivedAt}>
                      {packetDate(item.receivedAt)}
                    </time>
                  </button>
                )}
              </For>
            </Show>
            <Show when={filter() !== "all"}>
              <For each={items().filter((item) => packetKind(item) !== filter())}>
                {(item) => <span hidden>{item.content ?? "No text content"}</span>}
              </For>
            </Show>
          </div>
        </section>

        <section
          class="surface detail decision-detail"
          role="dialog"
          aria-modal="true"
          aria-labelledby="decision-detail-title"
          aria-describedby="decision-detail-note"
        >
          <button
            class="mobile-sheet-close"
            type="button"
            aria-label={items().length === 0 ? "Back to packet list" : "Back to queue"}
            onClick={() => setSelectedId(undefined)}
          >
            ← Decision queue
          </button>
          <Show
            when={selectedPacket()}
            fallback={
              <>
                <div class="detail-top">
                  <div>
                    <p class="eyebrow">Packet detail</p>
                    <h2 id="decision-detail-title">Decision packet detail</h2>
                    <p class="detail-sub">Select a durable packet to inspect its evidence.</p>
                  </div>
                </div>
                <p id="decision-detail-note" class="decision-note" role="status">
                  No packet is selected.
                </p>
              </>
            }
          >
            {(item) => (
              <>
                <div class="detail-top">
                  <div>
                    <p class="eyebrow" id="detail-meta">
                      {label(item().source.adapter)} · {item().source.conversation}
                    </p>
                    <h2 id="decision-detail-title">
                      {packetKind(item()) === "approval"
                        ? (item().content ?? "No text content")
                        : `${label(packetKind(item()))}: ${item().content ?? "No text content"}`}
                    </h2>
                    <p class="detail-sub">
                      Received {packetDate(item().receivedAt)} from {item().source.senderIdentity}
                    </p>
                  </div>
                  <span class="chip">{label(packetKind(item()))}</span>
                </div>
                <div class="actions" aria-label="Packet actions">
                  <button type="button" class="button primary" disabled title="Unsupported action">
                    Approve delivery
                  </button>
                  <button type="button" class="button" disabled title="Unsupported action">
                    Open source
                  </button>
                </div>
                <p id="decision-detail-note" class="decision-note" role="status">
                  Actions are disabled because delivery actions are unsupported by the current inbox
                  transport.
                </p>
                <div class="phase-line" aria-label="Packet lifecycle">
                  <span class="phase done">Received</span>
                  <span class="phase done">Classified</span>
                  <span class="phase current">Reviewed</span>
                  <span class="phase">Executed</span>
                  <span class="phase">Accepted</span>
                </div>
                <div class="detail-body">
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Source</h3>
                      <p>Durable origin retained by the inbox projection.</p>
                    </div>
                    <div class="detail-grid">
                      <div class="info-pair">
                        <small>Adapter</small>
                        <p>{label(item().source.adapter)}</p>
                      </div>
                      <div class="info-pair">
                        <small>Conversation</small>
                        <p>{item().source.conversation}</p>
                      </div>
                      <div class="info-pair">
                        <small>Sender identity</small>
                        <p>{item().source.senderIdentity}</p>
                      </div>
                    </div>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Authority</h3>
                      <p>Authority is never inferred from an external message.</p>
                    </div>
                    <p class="detail-copy">
                      No authority claim is present in the generated inbox contract.
                    </p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Execution</h3>
                      <p>Execution state remains separate from acceptance.</p>
                    </div>
                    <p class="detail-copy">This packet does not authorize or start execution.</p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Acceptance</h3>
                      <p>Acceptance must be evidenced by a separate authoritative record.</p>
                    </div>
                    <p class="detail-copy">No acceptance result is inferred from packet status.</p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Evidence</h3>
                      <p>Delivery and retention facts from the Telegram-backed transport.</p>
                    </div>
                    <div class="detail-grid">
                      <div class="info-pair">
                        <small>Delivery</small>
                        <p>Record: {packetEvidence(item())}</p>
                      </div>
                      <div class="info-pair">
                        <small>Dedupe key</small>
                        <p>
                          <code>{item().dedupeKey}</code>
                        </p>
                      </div>
                      <div class="info-pair">
                        <small>Payload</small>
                        <p>
                          {item().rawPayloadRetained
                            ? "Raw payload retained"
                            : "Raw payload not retained"}
                        </p>
                      </div>
                    </div>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Owner trail</h3>
                      <p>Only explicit transport identities are shown.</p>
                    </div>
                    <p class="detail-copy">
                      {item().source.senderIdentity} is the recorded sender. No owner is inferred.
                    </p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Blast radius and rollback</h3>
                      <p>Bounded actions fail closed until an authoritative command exists.</p>
                    </div>
                    <p class="detail-copy">
                      No blast-radius or rollback record is available from the current inbox
                      transport.
                    </p>
                  </section>
                </div>
              </>
            )}
          </Show>
        </section>
      </div>
    </section>
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
