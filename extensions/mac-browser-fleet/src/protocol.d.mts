export type BrowserKind = "chrome" | "edge";
export type ExtensionAction =
  | "activate_tab"
  | "navigate"
  | "reload"
  | "go_back"
  | "go_forward"
  | "create_tab"
  | "close_tab";

export interface PublicTab {
  tabRef: string;
  windowRef: string;
  nativeWindowId: number;
  nativeTabId: number;
  active: boolean;
  controllable: boolean;
  capabilities: string[];
  title?: string;
  url?: string;
}

export interface PublicWindow {
  windowRef: string;
  nativeWindowId: number;
  focused: boolean;
  tabs: PublicTab[];
}

export interface InventorySnapshot {
  browserKind: BrowserKind;
  displayName: string;
  profileLabel?: string;
  generation: number;
  capabilities: string[];
  windows: PublicWindow[];
}

export interface ExtensionHello {
  type: "hello";
  protocolVersion: number;
  browserKind: BrowserKind;
  extensionVersion: string;
  sessionId: string;
  profileLabel?: string;
}

export interface InventorySnapshotMessage {
  type: "inventory_snapshot";
  snapshot: InventorySnapshot;
}

export interface InventoryDeltaMessage {
  type: "inventory_delta";
  browserKind: BrowserKind;
  fromGeneration: number;
  toGeneration: number;
  addedTabs: PublicTab[];
  updatedTabs: PublicTab[];
  removedTabRefs: string[];
  truncated: boolean;
}

export interface ActionPoll {
  type: "action_poll";
  requestId: string;
}

export type ActionResponse =
  | {
      type: "action_response";
      requestId: string;
      ok: true;
      result: Record<string, unknown>;
    }
  | {
      type: "action_response";
      requestId: string;
      ok: false;
      error: { code: string; message: string };
    };

export interface HelloAck {
  type: "hello_ack";
  protocolVersion: number;
  sessionId: string;
}

export interface InventoryRequest {
  type: "inventory_request";
  requestId: string;
}

export interface ActionIdle {
  type: "action_idle";
}

export interface ActionRequest {
  type: "action_request";
  requestId: string;
  generation: number;
  action: ExtensionAction;
  target: { windowId?: number; tabId?: number };
  payload: Record<string, unknown>;
}

export type ExtensionMessage =
  | ExtensionHello
  | InventorySnapshotMessage
  | InventoryDeltaMessage
  | ActionPoll
  | ActionResponse;

export type NativeHostMessage = HelloAck | InventoryRequest | ActionRequest | ActionIdle;

export declare const PROTOCOL_VERSION: 1;
export declare const MAX_NATIVE_MESSAGE_BYTES: number;
export declare const EXTENSION_MESSAGE_TYPES: readonly ExtensionMessage["type"][];
export declare const NATIVE_HOST_MESSAGE_TYPES: readonly NativeHostMessage["type"][];
export declare const MESSAGE_SCHEMAS: Readonly<
  Record<
    ExtensionMessage["type"] | NativeHostMessage["type"],
    Readonly<{ required: readonly string[]; optional: readonly string[] }>
  >
>;

export declare class ProtocolError extends Error {
  readonly code: string;
  constructor(code: string, message: string);
}

export declare function parseExtensionMessage(message: unknown): ExtensionMessage;
export declare function parseNativeHostMessage(
  message: unknown,
  options?: { expectedGeneration?: number },
): NativeHostMessage;

export declare function createProtocolValidator(): {
  parseExtensionMessage(message: unknown): ExtensionMessage;
  parseNativeHostMessage(
    message: unknown,
    options?: { expectedGeneration?: number },
  ): NativeHostMessage;
  reset(): void;
};
