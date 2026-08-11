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

  return {
    disconnect() {
      reconnectQueued = false;
      cleanupConnection();
    },
    get port() {
      return port;
    },
  };
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
  installMacBrowserFleetBridge(globalThis.chrome);
}
