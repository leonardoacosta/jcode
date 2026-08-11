import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_NATIVE_MESSAGE_BYTES,
  PROTOCOL_VERSION,
  ProtocolError,
  createProtocolValidator,
  parseExtensionMessage,
  parseNativeHostMessage,
} from "../src/protocol.mjs";

const snapshot = {
  browserKind: "chrome",
  displayName: "Chrome",
  profileLabel: "Default",
  generation: 7,
  capabilities: ["activate_tab", "navigate", "close_tab"],
  windows: [
    {
      windowRef: "win_a1",
      focused: true,
      tabs: [
        {
          tabRef: "tab_b2",
          windowRef: "win_a1",
          active: true,
          controllable: true,
          capabilities: ["activate_tab", "navigate"],
          title: "Example",
          url: "https://example.com/path",
        },
      ],
    },
  ],
};

test("accepts every extension-to-native-host message shape", () => {
  const messages = [
    {
      type: "hello",
      protocolVersion: PROTOCOL_VERSION,
      browserKind: "chrome",
      extensionVersion: "1.0.0",
      sessionId: "session-1",
      profileLabel: "Default",
    },
    { type: "inventory_snapshot", snapshot },
    {
      type: "inventory_delta",
      browserKind: "chrome",
      fromGeneration: 7,
      toGeneration: 8,
      addedTabs: [],
      updatedTabs: [],
      removedTabRefs: ["tab_old"],
      truncated: false,
    },
    { type: "action_response", requestId: "req-1", ok: true, result: {} },
    {
      type: "action_response",
      requestId: "req-2",
      ok: false,
      error: { code: "unsupported_action", message: "action is unavailable" },
    },
  ];

  for (const message of messages) {
    assert.deepEqual(parseExtensionMessage(message), message);
  }
});

test("accepts every native-host-to-extension message shape", () => {
  const messages = [
    { type: "hello_ack", protocolVersion: PROTOCOL_VERSION, sessionId: "session-1" },
    {
      type: "action_request",
      requestId: "req-1",
      generation: 7,
      action: "navigate",
      target: { windowId: 3, tabId: 9 },
      payload: { url: "https://example.com" },
    },
    { type: "inventory_request", requestId: "req-2" },
  ];

  for (const message of messages) {
    assert.deepEqual(parseNativeHostMessage(message), message);
  }
});

test("fails closed for malformed messages with bounded secret-safe errors", () => {
  for (const value of [null, [], { type: "unknown" }, { type: "hello", protocolVersion: 1 }]) {
    assert.throws(
      () => parseExtensionMessage(value),
      (error) =>
        error instanceof ProtocolError &&
        error.code === "invalid_message" &&
        !error.message.includes(JSON.stringify(value)),
    );
  }
});

test("rejects unsupported protocol versions", () => {
  assert.throws(
    () =>
      parseExtensionMessage({
        type: "hello",
        protocolVersion: PROTOCOL_VERSION + 1,
        browserKind: "edge",
        extensionVersion: "1.0.0",
        sessionId: "session-1",
      }),
    (error) => error instanceof ProtocolError && error.code === "unsupported_version",
  );
});

test("rejects stale target generations", () => {
  assert.throws(
    () =>
      parseNativeHostMessage(
        {
          type: "action_request",
          requestId: "req-stale",
          generation: 6,
          action: "reload",
          target: { tabId: 9 },
          payload: {},
        },
        { expectedGeneration: 7 },
      ),
    (error) => error instanceof ProtocolError && error.code === "stale_generation",
  );
});

test("rejects duplicate request IDs within a protocol session", () => {
  const validator = createProtocolValidator();
  const message = { type: "inventory_request", requestId: "req-duplicate" };

  validator.parseNativeHostMessage(message);
  assert.throws(
    () => validator.parseNativeHostMessage(message),
    (error) => error instanceof ProtocolError && error.code === "duplicate_request_id",
  );
});

test("rejects messages over the native messaging payload limit", () => {
  const oversized = {
    type: "action_response",
    requestId: "req-large",
    ok: true,
    result: { text: "x".repeat(MAX_NATIVE_MESSAGE_BYTES) },
  };

  assert.throws(
    () => parseExtensionMessage(oversized),
    (error) => error instanceof ProtocolError && error.code === "message_too_large",
  );
});

test("rejects unknown fields and unsafe numeric identifiers", () => {
  assert.throws(() =>
    parseNativeHostMessage({
      type: "inventory_request",
      requestId: "req-1",
      secret: "must-not-cross-the-extension-boundary",
    }),
  );
  assert.throws(() =>
    parseNativeHostMessage({
      type: "action_request",
      requestId: "req-2",
      generation: 1,
      action: "activate_tab",
      target: { tabId: -1, windowId: 2 },
      payload: {},
    }),
  );
});
