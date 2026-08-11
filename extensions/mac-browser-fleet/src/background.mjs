import { createExtensionActionHandler } from "./action-handler.mjs";
import {
  PROTOCOL_VERSION,
  createProtocolValidator,
  parseExtensionMessage,
} from "./protocol.mjs";
import {
  buildInventorySnapshot,
  diffInventorySnapshots,
} from "./inventory.mjs";

export const ACTION_MESSAGE_TYPES = Object.freeze({
  REQUEST: "action_request",
  RESPONSE: "action_response",
});

export const NATIVE_HOST_NAME = "dev.jcode.mac_browser_fleet";

/// Alarm that wakes a suspended MV3 service worker back into the fleet.
export const KEEPALIVE_ALARM_NAME = "jcode-mac-browser-fleet-keepalive";
const KEEPALIVE_PERIOD_MINUTES = 1;

const DEFAULT_BROWSER_KIND = "chrome";
const DEFAULT_DISPLAY_NAMES = Object.freeze({ chrome: "Google Chrome", edge: "Microsoft Edge" });
const INVENTORY_EVENTS = Object.freeze([
  ["tabs", "onCreated"],
  ["tabs", "onUpdated"],
  ["tabs", "onRemoved"],
  ["tabs", "onActivated"],
  ["windows", "onFocusChanged"],
  ["windows", "onRemoved"],
]);

export function detectBrowserKind(userAgent = globalThis.navigator?.userAgent ?? "") {
  return /\bEdg\//.test(userAgent) ? "edge" : "chrome";
}

function safePost(port, message) {
  parseExtensionMessage(message);
  port.postMessage(message);
}

function requestFailure(requestId, error) {
  return {
    type: ACTION_MESSAGE_TYPES.RESPONSE,
    requestId: typeof requestId === "string" && requestId.length > 0 ? requestId : "unknown",
    ok: false,
    error: {
      code: error?.code === "stale_generation" ? "stale_generation" : "invalid_request",
      message: error?.code === "stale_generation" ? "target generation is stale" : "invalid native request",
    },
  };
}

function removeListeners(removers) {
  for (const remove of removers.splice(0)) remove();
}

function addListener(source, listener, removers) {
  if (!source?.addListener) return;
  source.addListener(listener);
  removers.push(() => source.removeListener?.(listener));
}

async function queryWindows(browserApi) {
  if (typeof browserApi?.windows?.getAll !== "function") {
    throw new TypeError("browser windows.getAll API is required");
  }
  return browserApi.windows.getAll({ populate: true, windowTypes: ["normal"] });
}

function extensionVersion(browserApi) {
  try {
    return browserApi.runtime.getManifest?.().version ?? "0.0.0";
  } catch {
    return "0.0.0";
  }
}

export function installActionMessageHandler(port, browserApi) {
  if (!port?.onMessage?.addListener || typeof port.postMessage !== "function") {
    throw new TypeError("native messaging port is required");
  }

  const handleAction = createExtensionActionHandler(browserApi);
  port.onMessage.addListener(async (message) => {
    if (message?.type !== ACTION_MESSAGE_TYPES.REQUEST) return;

    const response = await handleAction(message);
    port.postMessage({ type: ACTION_MESSAGE_TYPES.RESPONSE, ...response });
  });
}

export function installMacBrowserFleetBridge(browserApi, options = {}) {
  if (typeof browserApi?.runtime?.connectNative !== "function") {
    throw new TypeError("native messaging runtime API is required");
  }

  const browserKind = options.browserKind ?? detectBrowserKind();
  const displayName = options.displayName ?? DEFAULT_DISPLAY_NAMES[browserKind];
  const hostName = options.hostName ?? NATIVE_HOST_NAME;
  const reconnect = options.reconnect !== false;
  const sessionId = options.sessionId ?? crypto.randomUUID();
  const profileLabel = options.profileLabel;
  const policy = options.policy ?? {};
  const maxChanges = options.maxChanges;
  const actionPollIntervalMs = options.actionPollIntervalMs ?? 1000;
  const handleAction = createExtensionActionHandler(browserApi);

  let port;
  let currentSnapshot;
  let generation = 0;
  let connected = false;
  let reconnectQueued = false;
  let inventoryQueued = false;
  let actionPollTimer;
  let actionPollCounter = 0;
  const removers = [];
  let validator = createProtocolValidator();

  const buildSnapshot = async () => buildInventorySnapshot({
    browserKind,
    displayName,
    profileLabel,
    generation: ++generation,
    windows: await queryWindows(browserApi),
    policy,
  });

  const postInitialSnapshot = async () => {
    if (!connected || !port) return;
    currentSnapshot = await buildSnapshot();
    if (connected && port) safePost(port, { type: "inventory_snapshot", snapshot: currentSnapshot });
  };

  const scheduleInventoryUpdate = () => {
    if (!connected || inventoryQueued) return;
    inventoryQueued = true;
    queueMicrotask(async () => {
      inventoryQueued = false;
      if (!connected || !port || !currentSnapshot) return;
      const nextSnapshot = await buildSnapshot();
      if (!connected || !port) return;
      const delta = diffInventorySnapshots(currentSnapshot, nextSnapshot, { maxChanges });
      currentSnapshot = nextSnapshot;
      safePost(port, { type: "inventory_delta", browserKind, ...delta });
    });
  };

  const installInventoryListeners = () => {
    removeListeners(removers);
    for (const [area, eventName] of INVENTORY_EVENTS) {
      addListener(browserApi?.[area]?.[eventName], scheduleInventoryUpdate, removers);
    }
  };

  const pollForAction = () => {
    if (!connected || !port) return;
    actionPollCounter += 1;
    safePost(port, { type: "action_poll", requestId: `poll-${sessionId}-${actionPollCounter}` });
  };

  const startActionPolling = () => {
    if (!(actionPollIntervalMs > 0)) return;
    clearInterval(actionPollTimer);
    actionPollTimer = setInterval(pollForAction, actionPollIntervalMs);
    actionPollTimer.unref?.();
  };

  const stopActionPolling = () => {
    clearInterval(actionPollTimer);
    actionPollTimer = undefined;
  };

  const cleanupConnection = () => {
    connected = false;
    port = undefined;
    currentSnapshot = undefined;
    generation = 0;
    inventoryQueued = false;
    stopActionPolling();
    removeListeners(removers);
  };

  const connect = () => {
    validator = createProtocolValidator();
    port = browserApi.runtime.connectNative(hostName);
    connected = true;
    reconnectQueued = false;
    installInventoryListeners();
    startActionPolling();

    safePost(port, {
      type: "hello",
      protocolVersion: PROTOCOL_VERSION,
      browserKind,
      extensionVersion: extensionVersion(browserApi),
      sessionId,
      ...(profileLabel === undefined ? {} : { profileLabel }),
    });

    port.onMessage.addListener(async (message) => {
      if (!connected || !port) return;
      let parsed;
      try {
        parsed = validator.parseNativeHostMessage(message, {
          expectedGeneration: currentSnapshot?.generation,
        });
      } catch (error) {
        if (message?.type === ACTION_MESSAGE_TYPES.REQUEST) {
          safePost(port, requestFailure(message?.requestId, error));
        }
        return;
      }

      if (parsed.type === "inventory_request") {
        if (!currentSnapshot) await postInitialSnapshot();
        else safePost(port, { type: "inventory_snapshot", snapshot: currentSnapshot });
        return;
      }
      if (parsed.type === ACTION_MESSAGE_TYPES.REQUEST) {
        const response = await handleAction(parsed);
        if (connected && port) safePost(port, { type: ACTION_MESSAGE_TYPES.RESPONSE, ...response });
        return;
      }
    });

    port.onDisconnect?.addListener(() => {
      cleanupConnection();
      if (reconnect && !reconnectQueued) {
        reconnectQueued = true;
        queueMicrotask(connect);
      }
    });

    queueMicrotask(postInitialSnapshot);
  };

  connect();

  // MV3 tears the service worker down when it goes idle. An open native port
  // keeps it alive, but once the browser suspends the worker nothing restarts
  // it and this browser silently disappears from the fleet. A periodic alarm is
  // the supported wakeup, and each tick re-establishes a dropped connection.
  let alarmListener;
  if (typeof browserApi?.alarms?.create === "function") {
    browserApi.alarms.create(KEEPALIVE_ALARM_NAME, {
      periodInMinutes: KEEPALIVE_PERIOD_MINUTES,
    });
    alarmListener = (alarm) => {
      if (alarm?.name !== KEEPALIVE_ALARM_NAME) return;
      if (!connected || !port) {
        reconnectQueued = false;
        connect();
      }
    };
    browserApi.alarms.onAlarm?.addListener?.(alarmListener);
  }

  return {
    disconnect() {
      reconnectQueued = false;
      if (alarmListener) {
        browserApi.alarms?.onAlarm?.removeListener?.(alarmListener);
        browserApi.alarms?.clear?.(KEEPALIVE_ALARM_NAME);
      }
      cleanupConnection();
    },
    get port() {
      return port;
    },
  };
}

const PROFILE_LABEL_STORAGE_KEY = "jcodeMacBrowserFleetProfileLabel";

/// Stable per-profile label used to key this browser profile in the fleet.
///
/// Chromium runs one extension instance per profile, and the broker keys
/// extension sources by (browserKind, profileLabel). Without a distinct label
/// every profile of a browser collapses onto one source key and the profiles
/// evict each other instead of joining the fleet side by side.
export async function resolveProfileLabel({ storage, identityEmail } = {}) {
  if (typeof identityEmail === "string" && identityEmail.trim().length > 0) {
    return identityEmail.trim();
  }
  const local = storage?.local;
  if (!local) return undefined;
  const existing = await local.get(PROFILE_LABEL_STORAGE_KEY);
  const stored = existing?.[PROFILE_LABEL_STORAGE_KEY];
  if (typeof stored === "string" && stored.length > 0) return stored;
  const generated = `profile-${crypto.randomUUID().slice(0, 8)}`;
  await local.set({ [PROFILE_LABEL_STORAGE_KEY]: generated });
  return generated;
}

export function connectActionNativeHost(browserApi, hostName = NATIVE_HOST_NAME) {
  if (typeof browserApi?.runtime?.connectNative !== "function") {
    throw new TypeError("native messaging runtime API is required");
  }

  const port = browserApi.runtime.connectNative(hostName);
  installActionMessageHandler(port, browserApi);
  return port;
}

if (globalThis.chrome?.runtime?.connectNative) {
  const api = globalThis.chrome;
  // MV3 service workers forbid top-level await, so the async profile-label
  // lookup has to run inside a function. Bridge installation is deferred by
  // exactly one microtask-driven promise, which the broker tolerates because
  // the first snapshot is posted after connect() anyway.
  void (async () => {
    const identityEmail = await new Promise((resolve) => {
      try {
        if (typeof api.identity?.getProfileUserInfo !== "function") {
          resolve(undefined);
          return;
        }
        api.identity.getProfileUserInfo((info) => resolve(info?.email));
      } catch {
        resolve(undefined);
      }
    });
    const profileLabel = await resolveProfileLabel({
      storage: api.storage,
      identityEmail,
    });
    installMacBrowserFleetBridge(api, { profileLabel });
  })();
}
