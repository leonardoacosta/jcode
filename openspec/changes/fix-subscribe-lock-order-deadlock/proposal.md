## Why

Session subscribe requests have been observed to stall for more than a minute before swarm registration completes. The server must not hold competing swarm-state locks while it waits for another map, because a lock cycle can leave reconnecting clients and subsequent requests blocked.

## What Changes

- Elect a replacement swarm coordinator from independent snapshots of the swarm index and member map.
- Add lifecycle events around subscribe's swarm-map lock stages so any later stall identifies the blocked stage.
- Add regression coverage for coordinator candidate selection.

## Capabilities

### New Capabilities

- `session-subscription`: subscribe completion remains reliable during concurrent swarm membership cleanup.

### Modified Capabilities

None.

## Impact

The change affects Jcode's local `jcode-app-core` server implementation and its tests. It does not change provider selection, persisted session formats, or client/server protocol fields.
