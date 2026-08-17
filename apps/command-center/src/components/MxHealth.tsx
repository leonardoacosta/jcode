import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import type {
  MxAdapterState,
  MxCheckStatus,
  MxFailureCategory,
  MxHealthCheck,
  MxHealthProjection,
  MxOverallStatus,
} from "../generated/mx-health-contract";

export interface MxHealthPageProps {
  projection?: MxHealthProjection;
  loading?: boolean;
  refreshing?: boolean;
  error?: unknown;
  onRetry?: () => void;
}

const overallLabels: Record<MxOverallStatus, string> = {
  ok: "Healthy",
  degraded: "Degraded",
  down: "Down",
};

const checkLabels: Record<MxCheckStatus, string> = {
  ok: "OK",
  degraded: "Degraded",
  down: "Down",
  blocked: "Blocked",
};

const adapterLabels: Record<MxAdapterState, string> = {
  live: "Live read",
  stale: "Stale last-known-good",
  unconfigured: "Setup required",
  unauthorized: "Upstream unauthorized",
  unreachable: "Upstream unreachable",
  timeout: "Read timed out",
  invalid_contract: "Invalid contract",
  unavailable: "Unavailable",
};

const failureLabels: Record<MxFailureCategory, string> = {
  unauthorized: "upstream authorization failed",
  unexpected_status: "upstream returned an unsupported status",
  timeout: "the bounded read timed out",
  unreachable: "the upstream could not be reached",
  oversized: "the response exceeded the safety limit",
  invalid_contract: "the response did not match the pinned contract",
};

function humanize(value: string) {
  return value.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function formatAge(value?: string) {
  if (!value) return "Not available";
  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) return value;
  const seconds = Math.max(0, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 10) return "Just now";
  if (seconds < 60) return `${seconds} seconds ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? "" : "s"} ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours} hour${hours === 1 ? "" : "s"} ago`;
}

function errorCategory(error: unknown) {
  const message = error instanceof Error ? error.message : String(error ?? "");
  const match = message.match(/mx_health_(401|403|408|422|502|503|504)/);
  switch (match?.[1]) {
    case "401":
    case "403":
      return "unauthorized";
    case "408":
    case "504":
      return "timeout";
    case "422":
      return "invalid_contract";
    default:
      return "unreachable";
  }
}

function adapterCopy(projection: MxHealthProjection | undefined, error: unknown) {
  if (projection?.adapterState === "unconfigured") {
    return "MX health is not configured for this daemon. Ask an administrator to configure the read-only health adapter.";
  }
  if (projection?.adapterState === "stale" && projection.stale) {
    return `Showing the last validated MX health from ${formatAge(projection.stale.cachedFetchedAt)} because ${failureLabels[projection.stale.currentFailure]}.`;
  }
  const category = projection?.failureCategory ?? errorCategory(error);
  return `Jcode could not obtain a current MX health read because ${failureLabels[category] ?? "the adapter is unavailable"}.`;
}

function impactCopy(checks: MxHealthCheck[], overall?: MxOverallStatus) {
  if (!overall) return "No MX checks are available until the adapter is configured.";
  const down = checks.filter((check) => check.status === "down");
  const blocked = checks.filter((check) => check.status === "blocked");
  if (overall === "ok") return "MX reports all observed layers as healthy.";
  if (down.length > 0 && blocked.length > 0) {
    return `${down.map((check) => check.layer).join(", ")} remain available independently while ${down.map((check) => check.id).join(", ")} is down; ${blocked.map((check) => check.id).join(", ")} are blocked by declared dependencies.`;
  }
  if (down.length > 0) {
    return `${down.map((check) => check.id).join(", ")} report down. Independent healthy checks remain visible without masking the affected layer.`;
  }
  return `${checks
    .filter((check) => check.status === "degraded")
    .map((check) => check.id)
    .join(", ")} report degraded while independent checks remain visible.`;
}

interface TopologyNode {
  check: MxHealthCheck;
  x: number;
  y: number;
}

interface TopologyEdge {
  from: TopologyNode;
  to: TopologyNode;
}

function topologyFor(checks: MxHealthCheck[]) {
  const layers = [...new Set(checks.map((check) => check.layer))];
  const layerIndex = new Map(layers.map((layer, index) => [layer, index]));
  const byId = new Map<string, TopologyNode>();
  const nodes: TopologyNode[] = [];
  checks.forEach((check, index) => {
    const layer = layerIndex.get(check.layer) ?? 0;
    const sameLayer = checks
      .slice(0, index)
      .filter((candidate) => candidate.layer === check.layer).length;
    const node = { check, x: 112 + layer * 210, y: 74 + sameLayer * 82 };
    nodes.push(node);
    byId.set(check.id, node);
  });
  const edges: TopologyEdge[] = [];
  for (const node of nodes) {
    for (const dependency of node.check.dependsOn ?? []) {
      const from = byId.get(dependency);
      if (from) edges.push({ from, to: node });
    }
  }
  return {
    layers,
    nodes,
    edges,
    width: Math.max(520, layers.length * 210 + 100),
    height: Math.max(170, Math.max(...nodes.map((node) => node.y), 0) + 78),
  };
}

function statusClass(status: MxCheckStatus | MxOverallStatus | MxAdapterState) {
  return status.replaceAll("_", "-");
}

function CheckStatus(props: { status: MxCheckStatus }) {
  return (
    <span class={`mx-status mx-status-${statusClass(props.status)}`}>
      <span class="mx-status-mark" aria-hidden="true">
        {props.status === "ok" ? "✓" : props.status === "blocked" ? "!" : "×"}
      </span>
      <span>{checkLabels[props.status]}</span>
    </span>
  );
}

export function MxHealthPage(props: MxHealthPageProps) {
  const [selectedId, setSelectedId] = createSignal<string>();
  const [announcement, setAnnouncement] = createSignal("");
  let lastFingerprint = "";
  const health = () => props.projection?.health;
  const checks = () => health()?.checks ?? [];
  const selected = createMemo(
    () => checks().find((check) => check.id === selectedId()) ?? checks()[0],
  );
  const topology = createMemo(() => topologyFor(checks()));
  const overall = () => health()?.overall;
  const state = () => props.projection?.adapterState;

  createEffect(() => {
    const currentChecks = checks();
    if (
      currentChecks.length > 0 &&
      (!selectedId() || !currentChecks.some((check) => check.id === selectedId()))
    ) {
      setSelectedId(currentChecks[0].id);
    }
    const fingerprint = `${props.projection?.fetchedAt ?? ""}:${state() ?? ""}:${overall() ?? ""}`;
    if (lastFingerprint && fingerprint !== lastFingerprint) {
      setAnnouncement(
        `MX health refreshed: ${overall() ? overallLabels[overall()!] : adapterLabels[state() ?? "unavailable"]}.`,
      );
    }
    lastFingerprint = fingerprint;
  });

  const selectCheck = (id: string) => setSelectedId(id);
  const moveSelection = (currentId: string, delta: number) => {
    const index = checks().findIndex((check) => check.id === currentId);
    if (index < 0 || checks().length === 0) return;
    const next = checks()[(index + delta + checks().length) % checks().length];
    setSelectedId(next.id);
    document
      .querySelector<HTMLButtonElement>(`[data-mx-check-id="${CSS.escape(next.id)}"]`)
      ?.focus();
  };

  return (
    <section class="page mx-page" aria-labelledby="mx-health-title">
      <header class="page-bar mx-page-bar">
        <div>
          <p class="eyebrow">Read-only operator view</p>
          <h1 id="mx-health-title">MX health</h1>
          <p class="mx-intro">
            The committed MX health authority, projected through the Jcode daemon.
          </p>
        </div>
        <Show when={props.projection?.adapterState !== "unconfigured"}>
          <button class="button" type="button" onClick={() => props.onRetry?.()}>
            Retry read
          </button>
        </Show>
      </header>

      <div class="sr-only" role="status" aria-live="polite">
        {announcement()}
      </div>

      <Show when={props.loading && !props.projection}>
        <section class="state-card mx-state-card" aria-live="polite">
          <p class="eyebrow">Connecting</p>
          <h2>Loading MX health</h2>
          <p>Requesting the authenticated Jcode health projection.</p>
        </section>
      </Show>

      <Show when={!props.loading || props.projection}>
        <Show when={state() === "unconfigured"}>
          <section class="state-card mx-state-card" aria-live="polite">
            <p class="eyebrow">Setup required</p>
            <h2>MX health is not configured</h2>
            <p>{adapterCopy(props.projection, props.error)}</p>
          </section>
        </Show>

        <Show when={state() !== "unconfigured" && !health()}>
          <section class="state-card mx-state-card" aria-live="polite" role="alert">
            <p class="eyebrow">{adapterLabels[state() ?? "unavailable"]}</p>
            <h2>MX health read unavailable</h2>
            <p>{adapterCopy(props.projection, props.error)}</p>
            <button class="button primary" type="button" onClick={() => props.onRetry?.()}>
              Retry read
            </button>
          </section>
        </Show>

        <Show when={health()}>
          {(snapshot) => (
            <>
              <Show when={state() === "stale"}>
                <section class="mx-notice" role="status">
                  <strong>Stale last-known-good</strong>
                  <span>{adapterCopy(props.projection, props.error)}</span>
                </section>
              </Show>

              <section class="mx-summary surface" aria-labelledby="mx-summary-title">
                <div class="surface-head">
                  <h2 id="mx-summary-title">Overall state</h2>
                  <span>
                    {props.refreshing ? "Refreshing read" : "Authoritative MX projection"}
                  </span>
                </div>
                <div class="mx-summary-body">
                  <div class={`mx-overall mx-overall-${statusClass(snapshot().overall)}`}>
                    <span class="mx-overall-mark" aria-hidden="true">
                      {snapshot().overall === "ok"
                        ? "✓"
                        : snapshot().overall === "down"
                          ? "×"
                          : "!"}
                    </span>
                    <div>
                      <p class="eyebrow">MX reports</p>
                      <p class="mx-overall-label">{overallLabels[snapshot().overall]}</p>
                      <p class="mx-impact">{impactCopy(snapshot().checks, snapshot().overall)}</p>
                    </div>
                  </div>
                  <dl class="mx-freshness">
                    <div>
                      <dt>MX generated</dt>
                      <dd>{formatAge(snapshot().generatedAt)}</dd>
                    </div>
                    <div>
                      <dt>Jcode fetched</dt>
                      <dd>{formatAge(props.projection?.fetchedAt)}</dd>
                    </div>
                    <div>
                      <dt>Adapter</dt>
                      <dd>{adapterLabels[state() ?? "unavailable"]}</dd>
                    </div>
                  </dl>
                </div>
              </section>

              <section class="mx-topology surface" aria-labelledby="mx-topology-title">
                <div class="surface-head">
                  <h2 id="mx-topology-title">Dependency topology</h2>
                  <span>Edges follow declared depends_on</span>
                </div>
                <div class="mx-topology-scroll">
                  <svg
                    class="mx-topology-svg"
                    role="img"
                    aria-label="MX health dependency topology"
                    aria-describedby="mx-topology-description"
                    viewBox={`0 0 ${topology().width} ${topology().height}`}
                  >
                    <title id="mx-topology-name">MX health dependency topology</title>
                    <desc id="mx-topology-description">
                      {topology().layers.join(", ")} layers with {topology().nodes.length} checks.
                      Use the semantic check list below to inspect details.
                    </desc>
                    <For each={topology().edges}>
                      {(edge) => (
                        <path
                          class="mx-topology-edge"
                          aria-hidden="true"
                          d={`M ${edge.from.x + 62} ${edge.from.y} C ${edge.from.x + 150} ${edge.from.y}, ${edge.to.x - 150} ${edge.to.y}, ${edge.to.x - 62} ${edge.to.y}`}
                        />
                      )}
                    </For>
                    <For each={topology().nodes}>
                      {(node) => (
                        <g
                          class={`mx-topology-node mx-topology-${statusClass(node.check.status)}`}
                          data-check-id={node.check.id}
                          aria-hidden="true"
                          transform={`translate(${node.x - 62} ${node.y - 24})`}
                        >
                          <rect width="124" height="48" rx="8" />
                          <circle cx="16" cy="24" r="6" />
                          <text x="29" y="21">
                            {node.check.layer}
                          </text>
                          <text x="29" y="36">
                            {node.check.status}
                          </text>
                        </g>
                      )}
                    </For>
                  </svg>
                </div>
              </section>

              <div class="mx-content-grid">
                <section class="mx-checks surface" aria-labelledby="mx-checks-title">
                  <div class="surface-head">
                    <h2 id="mx-checks-title">Checks by layer</h2>
                    <span>{checks().length} checks</span>
                  </div>
                  <ul class="mx-check-list" aria-label="MX health checks">
                    <For each={snapshot().checks}>
                      {(check) => (
                        <li>
                          <button
                            ref={(element) => {
                              element.dataset.mxCheckId = check.id;
                            }}
                            class={`mx-check-button ${selected()?.id === check.id ? "selected" : ""}`}
                            type="button"
                            data-mx-check-id={check.id}
                            aria-pressed={selected()?.id === check.id}
                            onClick={() => selectCheck(check.id)}
                            onKeyDown={(event) => {
                              if (event.key === "ArrowDown" || event.key === "ArrowRight") {
                                event.preventDefault();
                                moveSelection(check.id, 1);
                              }
                              if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
                                event.preventDefault();
                                moveSelection(check.id, -1);
                              }
                            }}
                          >
                            <span class="sr-only">{semanticCheckLabel(check)}</span>
                            <span class="mx-check-copy">
                              <strong>{check.id}</strong>
                              <span>{check.layer}</span>
                              <small>{check.summary}</small>
                            </span>
                            <CheckStatus status={check.status} />
                          </button>
                        </li>
                      )}
                    </For>
                  </ul>
                </section>

                <section
                  class="mx-details surface"
                  aria-labelledby="mx-details-title"
                  aria-live="polite"
                >
                  <div class="surface-head">
                    <h2 id="mx-details-title">Check details</h2>
                    <span>{selected()?.id ?? "Select a check"}</span>
                  </div>
                  <Show
                    when={selected()}
                    fallback={
                      <p class="empty-line">Select a check to inspect its committed fields.</p>
                    }
                  >
                    {(check) => (
                      <div class="mx-detail-body">
                        <div class="mx-detail-status">
                          <p class="eyebrow">Current status</p>
                          <CheckStatus status={check().status} />
                        </div>
                        <dl class="mx-detail-list">
                          <div>
                            <dt>Check ID</dt>
                            <dd>
                              <code>{check().id}</code>
                            </dd>
                          </div>
                          <div>
                            <dt>Layer</dt>
                            <dd>{check().layer}</dd>
                          </div>
                          <div>
                            <dt>Reason code</dt>
                            <dd>
                              <code>{check().reasonCode}</code>
                            </dd>
                          </div>
                          <div>
                            <dt>Summary</dt>
                            <dd>{check().summary}</dd>
                          </div>
                          <div>
                            <dt>Dependencies</dt>
                            <dd>
                              <Show when={check().dependsOn?.length} fallback="None declared">
                                <ul class="mx-dependency-list">
                                  <For each={check().dependsOn}>
                                    {(dependency) => (
                                      <li>
                                        <code>{dependency}</code>
                                      </li>
                                    )}
                                  </For>
                                </ul>
                              </Show>
                            </dd>
                          </div>
                        </dl>
                      </div>
                    )}
                  </Show>
                </section>
              </div>

              <section class="mx-legend" aria-labelledby="mx-legend-title">
                <h2 id="mx-legend-title">Legend</h2>
                <ul>
                  <li>
                    <CheckStatus status="ok" /> Independent healthy check
                  </li>
                  <li>
                    <CheckStatus status="degraded" /> Degraded check
                  </li>
                  <li>
                    <CheckStatus status="down" /> Down check
                  </li>
                  <li>
                    <CheckStatus status="blocked" /> Blocked by a declared dependency
                  </li>
                </ul>
              </section>
            </>
          )}
        </Show>
      </Show>
    </section>
  );
}

export function mxStatusLabel(status: MxCheckStatus | MxOverallStatus) {
  return status in checkLabels
    ? checkLabels[status as MxCheckStatus]
    : overallLabels[status as MxOverallStatus];
}

export function mxAdapterLabel(state: MxAdapterState) {
  return adapterLabels[state];
}

export function topologyChecks(checks: MxHealthCheck[]) {
  return topologyFor(checks);
}

export function semanticCheckLabel(check: MxHealthCheck) {
  return `${check.id}: ${checkLabels[check.status]}`;
}

export function mxHumanize(value: string) {
  return humanize(value);
}
