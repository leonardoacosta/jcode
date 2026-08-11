export const PROTOCOL_VERSION = 1;
export const MAX_NATIVE_MESSAGE_BYTES = 1024 * 1024;

export const EXTENSION_MESSAGE_TYPES = Object.freeze([
  "hello",
  "inventory_snapshot",
  "inventory_delta",
  "action_poll",
  "action_response",
]);

export const NATIVE_HOST_MESSAGE_TYPES = Object.freeze([
  "hello_ack",
  "inventory_request",
  "action_request",
  "action_idle",
]);

export const MESSAGE_SCHEMAS = Object.freeze({
  hello: Object.freeze({
    required: ["type", "protocolVersion", "browserKind", "extensionVersion", "sessionId"],
    optional: ["profileLabel"],
  }),
  hello_ack: Object.freeze({
    required: ["type", "protocolVersion", "sessionId"],
    optional: [],
  }),
  inventory_request: Object.freeze({ required: ["type", "requestId"], optional: [] }),
  action_poll: Object.freeze({ required: ["type", "requestId"], optional: [] }),
  action_idle: Object.freeze({ required: ["type"], optional: [] }),
  inventory_snapshot: Object.freeze({ required: ["type", "snapshot"], optional: [] }),
  inventory_delta: Object.freeze({
    required: [
      "type",
      "browserKind",
      "fromGeneration",
      "toGeneration",
      "addedTabs",
      "updatedTabs",
      "removedTabRefs",
      "truncated",
    ],
    optional: [],
  }),
  action_request: Object.freeze({
    required: ["type", "requestId", "generation", "action", "target", "payload"],
    optional: [],
  }),
  action_response: Object.freeze({
    required: ["type", "requestId", "ok"],
    optional: ["result", "error"],
  }),
});

/**
 * @typedef {"chrome" | "edge"} BrowserKind
 * @typedef {{type:"hello", protocolVersion:number, browserKind:BrowserKind, extensionVersion:string, sessionId:string, profileLabel?:string}} ExtensionHello
 * @typedef {{type:"inventory_request", requestId:string}} InventoryRequest
 * @typedef {{type:"action_poll", requestId:string}} ActionPoll
 * @typedef {{type:"action_request", requestId:string, generation:number, action:string, target:{windowId?:number, tabId?:number}, payload:Record<string, unknown>}} ActionRequest
 * @typedef {{type:"action_response", requestId:string, ok:boolean, result?:Record<string, unknown>, error?:{code:string, message:string}}} ActionResponse
 * @typedef {ExtensionHello | ActionPoll | ActionResponse | ({type:"inventory_snapshot", snapshot:Record<string, unknown>}) | ({type:"inventory_delta"} & Record<string, unknown>)} ExtensionMessage
 * @typedef {InventoryRequest | ActionRequest | {type:"action_idle"} | {type:"hello_ack", protocolVersion:number, sessionId:string}} NativeHostMessage
 */

export class ProtocolError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "ProtocolError";
    this.code = code;
  }
}

const encoder = new TextEncoder();
const ACTIONS = new Set([
  "activate_tab",
  "navigate",
  "reload",
  "go_back",
  "go_forward",
  "create_tab",
  "close_tab",
]);

function fail(code, message) {
  throw new ProtocolError(code, message);
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isString(value) {
  return typeof value === "string" && value.length > 0 && value.length <= 256;
}

function isId(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function assertBounded(message) {
  let json;
  try {
    json = JSON.stringify(message);
  } catch {
    fail("invalid_message", "message must be JSON serializable");
  }
  if (json === undefined) fail("invalid_message", "message must be JSON serializable");
  if (encoder.encode(json).byteLength > MAX_NATIVE_MESSAGE_BYTES) {
    fail("message_too_large", "message exceeds the native messaging payload limit");
  }
}

function assertShape(message, type) {
  const schema = MESSAGE_SCHEMAS[type];
  const allowed = new Set([...schema.required, ...schema.optional]);
  if (Object.keys(message).some((key) => !allowed.has(key))) {
    fail("invalid_message", "message contains unsupported fields");
  }
  if (schema.required.some((key) => !(key in message))) {
    fail("invalid_message", "message is missing required fields");
  }
}

function assertVersion(version) {
  if (version !== PROTOCOL_VERSION) {
    fail("unsupported_version", "protocol version is not supported");
  }
}

function assertBrowserKind(value) {
  if (value !== "chrome" && value !== "edge") {
    fail("invalid_message", "browser kind is invalid");
  }
}

function assertStringArray(value) {
  if (!Array.isArray(value) || value.some((item) => !isString(item))) {
    fail("invalid_message", "expected a bounded string array");
  }
}

function assertTab(tab) {
  if (!isObject(tab) || !isString(tab.tabRef) || !isString(tab.windowRef)) {
    fail("invalid_message", "tab metadata is invalid");
  }
  if (!isId(tab.nativeTabId) || !isId(tab.nativeWindowId)) fail("invalid_message", "native tab metadata is invalid");
  if (typeof tab.active !== "boolean" || typeof tab.controllable !== "boolean") {
    fail("invalid_message", "tab state is invalid");
  }
  assertStringArray(tab.capabilities);
  if (tab.title !== undefined && typeof tab.title !== "string") fail("invalid_message", "tab title is invalid");
  if (tab.url !== undefined && typeof tab.url !== "string") fail("invalid_message", "tab URL is invalid");
}

function assertSnapshot(snapshot) {
  if (!isObject(snapshot)) fail("invalid_message", "inventory snapshot is invalid");
  assertBrowserKind(snapshot.browserKind);
  if (!isString(snapshot.displayName) || !isId(snapshot.generation)) fail("invalid_message", "inventory identity is invalid");
  if (snapshot.profileLabel !== undefined && !isString(snapshot.profileLabel)) fail("invalid_message", "profile label is invalid");
  assertStringArray(snapshot.capabilities);
  if (!Array.isArray(snapshot.windows)) fail("invalid_message", "inventory windows are invalid");
  for (const window of snapshot.windows) {
    if (!isObject(window) || !isString(window.windowRef) || !isId(window.nativeWindowId) || typeof window.focused !== "boolean" || !Array.isArray(window.tabs)) {
      fail("invalid_message", "inventory window is invalid");
    }
    for (const tab of window.tabs) assertTab(tab);
  }
}

function validateCommon(message, allowedTypes) {
  assertBounded(message);
  if (!isObject(message) || !isString(message.type) || !allowedTypes.includes(message.type)) {
    fail("invalid_message", "message type is invalid");
  }
  assertShape(message, message.type);
}

/** @returns {ExtensionMessage} */
export function parseExtensionMessage(message) {
  validateCommon(message, EXTENSION_MESSAGE_TYPES);
  switch (message.type) {
    case "hello":
      assertVersion(message.protocolVersion);
      assertBrowserKind(message.browserKind);
      if (!isString(message.extensionVersion) || !isString(message.sessionId)) fail("invalid_message", "extension identity is invalid");
      if (message.profileLabel !== undefined && !isString(message.profileLabel)) fail("invalid_message", "profile label is invalid");
      break;
    case "inventory_snapshot":
      assertSnapshot(message.snapshot);
      break;
    case "inventory_delta":
      assertBrowserKind(message.browserKind);
      if (!isId(message.fromGeneration) || !isId(message.toGeneration) || message.toGeneration <= message.fromGeneration) fail("invalid_message", "inventory generations are invalid");
      if (!Array.isArray(message.addedTabs) || !Array.isArray(message.updatedTabs)) fail("invalid_message", "inventory tab changes are invalid");
      for (const tab of [...message.addedTabs, ...message.updatedTabs]) assertTab(tab);
      assertStringArray(message.removedTabRefs);
      if (typeof message.truncated !== "boolean") fail("invalid_message", "inventory truncation state is invalid");
      break;
    case "action_response":
      if (!isString(message.requestId) || typeof message.ok !== "boolean") fail("invalid_message", "action response identity is invalid");
      if (message.ok) {
        if (!isObject(message.result) || message.error !== undefined) fail("invalid_message", "successful action response is invalid");
      } else if (!isObject(message.error) || !isString(message.error.code) || !isString(message.error.message) || message.result !== undefined) {
        fail("invalid_message", "failed action response is invalid");
      }
      break;
    case "action_poll":
      if (!isString(message.requestId)) fail("invalid_message", "request ID is invalid");
      break;
  }
  return message;
}

/** @returns {NativeHostMessage} */
export function parseNativeHostMessage(message, { expectedGeneration } = {}) {
  validateCommon(message, NATIVE_HOST_MESSAGE_TYPES);
  switch (message.type) {
    case "hello_ack":
      assertVersion(message.protocolVersion);
      if (!isString(message.sessionId)) fail("invalid_message", "session identity is invalid");
      break;
    case "inventory_request":
      if (!isString(message.requestId)) fail("invalid_message", "request ID is invalid");
      break;
    case "action_request":
      if (!isString(message.requestId) || !isId(message.generation) || !ACTIONS.has(message.action)) fail("invalid_message", "action request identity is invalid");
      if (expectedGeneration !== undefined && message.generation !== expectedGeneration) fail("stale_generation", "target generation is stale");
      if (!isObject(message.target) || !isObject(message.payload)) fail("invalid_message", "action request body is invalid");
      if (message.target.tabId !== undefined && !isId(message.target.tabId)) fail("invalid_message", "tab ID is invalid");
      if (message.target.windowId !== undefined && !isId(message.target.windowId)) fail("invalid_message", "window ID is invalid");
      if (message.target.tabId === undefined && message.target.windowId === undefined) fail("invalid_message", "action target is required");
      break;
    case "action_idle":
      break;
  }
  return message;
}

export function createProtocolValidator() {
  const requestIds = new Set();
  return {
    parseExtensionMessage,
    parseNativeHostMessage(message, options) {
      const parsed = parseNativeHostMessage(message, options);
      if (parsed.type === "inventory_request" || parsed.type === "action_request") {
        if (requestIds.has(parsed.requestId)) fail("duplicate_request_id", "request ID was already used");
        requestIds.add(parsed.requestId);
      }
      return parsed;
    },
    reset() {
      requestIds.clear();
    },
  };
}
