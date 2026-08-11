const ORDINARY_TAB_CAPABILITIES = Object.freeze([
  "activate_tab",
  "close_tab",
  "navigate",
]);

const CONTROLLABLE_PROTOCOLS = new Set(["http:", "https:"]);
const DEFAULT_MAX_CHANGES = 256;

function opaqueRef(prefix, parts) {
  const input = parts.join("\u0000");
  let hash = 0x811c9dc5;
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return `${prefix}_${(hash >>> 0).toString(36)}`;
}

function safeUrl(rawUrl, policy) {
  if (typeof rawUrl !== "string") return undefined;

  let parsed;
  try {
    parsed = new URL(rawUrl);
  } catch {
    return undefined;
  }

  if (!CONTROLLABLE_PROTOCOLS.has(parsed.protocol)) return undefined;

  parsed.username = "";
  parsed.password = "";
  parsed.search = "";
  parsed.hash = "";
  if (policy.hidePath) parsed.pathname = "/";

  const rendered = parsed.toString();
  return policy.hidePath ? rendered.replace(/\/$/, "") : rendered;
}

function publicTab(browserIdentity, windowRef, tab, policy) {
  const tabRef = opaqueRef("tab", [browserIdentity, String(tab.id)]);
  const url = safeUrl(tab.url, policy);
  const controllable = url !== undefined;

  return {
    tabRef,
    windowRef,
    active: tab.active === true,
    controllable,
    capabilities: controllable ? [...ORDINARY_TAB_CAPABILITIES] : [],
    ...(policy.hideTitle || typeof tab.title !== "string"
      ? {}
      : { title: tab.title }),
    ...(url === undefined ? {} : { url }),
  };
}

export function buildInventorySnapshot({
  browserKind,
  displayName,
  profileLabel,
  generation,
  windows,
  policy = {},
}) {
  if (!Number.isSafeInteger(generation) || generation < 0) {
    throw new TypeError(
      "inventory generation must be a non-negative safe integer",
    );
  }
  if (browserKind !== "chrome" && browserKind !== "edge") {
    throw new TypeError("browser kind must be chrome or edge");
  }
  if (!Array.isArray(windows)) {
    throw new TypeError("inventory windows must be an array");
  }

  const browserIdentity = `${browserKind}:${profileLabel ?? "default"}`;
  const publicWindows = windows
    .filter((window) => window && window.incognito !== true)
    .map((window) => {
      const windowRef = opaqueRef("win", [browserIdentity, String(window.id)]);
      return {
        windowRef,
        focused: window.focused === true,
        tabs: Array.isArray(window.tabs)
          ? window.tabs.map((tab) =>
              publicTab(browserIdentity, windowRef, tab, policy),
            )
          : [],
      };
    });

  return {
    browserKind,
    displayName,
    ...(profileLabel === undefined ? {} : { profileLabel }),
    generation,
    capabilities: [...ORDINARY_TAB_CAPABILITIES],
    windows: publicWindows,
  };
}

function tabsByRef(snapshot) {
  return new Map(
    snapshot.windows.flatMap((window) =>
      window.tabs.map((tab) => [tab.tabRef, tab]),
    ),
  );
}

function sameTab(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function diffInventorySnapshots(
  previous,
  current,
  { maxChanges = DEFAULT_MAX_CHANGES } = {},
) {
  if (current.generation <= previous.generation) {
    throw new Error("inventory delta requires a newer inventory generation");
  }
  if (!Number.isSafeInteger(maxChanges) || maxChanges < 0) {
    throw new TypeError("maxChanges must be a non-negative safe integer");
  }

  const before = tabsByRef(previous);
  const after = tabsByRef(current);
  const changes = [];

  for (const [tabRef, tab] of after) {
    const oldTab = before.get(tabRef);
    if (oldTab === undefined) changes.push(["added", tab]);
    else if (!sameTab(oldTab, tab)) changes.push(["updated", tab]);
  }
  for (const tabRef of before.keys()) {
    if (!after.has(tabRef)) changes.push(["removed", tabRef]);
  }

  const emitted = changes.slice(0, maxChanges);
  return {
    fromGeneration: previous.generation,
    toGeneration: current.generation,
    addedTabs: emitted
      .filter(([kind]) => kind === "added")
      .map(([, tab]) => tab),
    updatedTabs: emitted
      .filter(([kind]) => kind === "updated")
      .map(([, tab]) => tab),
    removedTabRefs: emitted
      .filter(([kind]) => kind === "removed")
      .map(([, tabRef]) => tabRef),
    truncated: emitted.length < changes.length,
  };
}
