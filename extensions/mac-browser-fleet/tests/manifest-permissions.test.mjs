import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const root = new URL("../", import.meta.url);

async function readJson(path) {
  return JSON.parse(await readFile(new URL(path, root), "utf8"));
}

const expectedPermissions = ["alarms", "nativeMessaging", "scripting", "storage", "tabs"];
const expectedHostPermissions = ["http://*/*", "https://*/*"];
const forbiddenPermissions = [
  "bookmarks",
  "browsingData",
  "clipboardRead",
  "clipboardWrite",
  "contentSettings",
  "cookies",
  "debugger",
  "downloads",
  "geolocation",
  "history",
  "management",
  "privacy",
  "proxy",
  "sessions",
  "topSites",
  "webRequest",
];

for (const browser of ["chrome", "edge"]) {
  test(`${browser} manifest is a minimal MV3 native-messaging extension`, async () => {
    const manifest = await readJson(`manifests/${browser}.json`);

    assert.equal(manifest.manifest_version, 3);
    assert.equal(manifest.version, "0.1.0");
    assert.match(manifest.name, /Jcode Mac Browser Fleet/);
    assert.equal(manifest.background.service_worker, "src/background.mjs");
    assert.equal(manifest.background.type, "module");
    assert.equal(manifest.incognito, "not_allowed");
    assert.deepEqual(manifest.permissions, expectedPermissions);
    assert.deepEqual(manifest.host_permissions, expectedHostPermissions);
    assert.equal(manifest.optional_permissions, undefined);
    assert.equal(manifest.optional_host_permissions, undefined);

    for (const permission of forbiddenPermissions) {
      assert.equal(
        manifest.permissions.includes(permission),
        false,
        `${browser} must not request ${permission}`,
      );
    }
  });
}

test("Chrome and Edge differ only in browser identity fields", async () => {
  const chrome = await readJson("manifests/chrome.json");
  const edge = await readJson("manifests/edge.json");
  const omitIdentity = ({ name, description, ...manifest }) => manifest;

  assert.deepEqual(omitIdentity(chrome), omitIdentity(edge));
  assert.notEqual(chrome.name, edge.name);
  assert.notEqual(chrome.description, edge.description);
});

test("permission asset exactly accounts for every requested grant", async () => {
  const permissions = await readJson("config/permissions.json");
  const chrome = await readJson("manifests/chrome.json");

  assert.deepEqual(permissions.permissions, expectedPermissions);
  assert.deepEqual(permissions.host_permissions, expectedHostPermissions);
  assert.deepEqual(Object.keys(permissions.rationale).sort(), expectedPermissions.toSorted());
  assert.deepEqual(permissions.denied_permissions, forbiddenPermissions);
  assert.equal(permissions.incognito_allowed, false);
  assert.equal(permissions.restricted_schemes_allowed, false);
  assert.deepEqual(chrome.permissions, permissions.permissions);
  assert.deepEqual(chrome.host_permissions, permissions.host_permissions);
});
