import { createReadStream, realpathSync } from "node:fs";
import { access, stat } from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const MIME_TYPES = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
};

const thisDirectory = path.dirname(fileURLToPath(import.meta.url));

function resolvePublicFile(publicDir, pathname) {
  const relativePath = pathname === "/" ? "index.html" : pathname.slice(1);
  if (!relativePath || relativePath.includes("\\") || relativePath.split("/").includes("..")) {
    return null;
  }

  const filePath = path.resolve(publicDir, relativePath);
  const publicRoot = path.resolve(publicDir);
  return filePath === publicRoot || filePath.startsWith(`${publicRoot}${path.sep}`)
    ? filePath
    : null;
}

async function serveStatic(request, response, publicDir) {
  const pathname = new URL(request.url ?? "/", "http://command-center").pathname;
  const requestedFile = resolvePublicFile(publicDir, pathname);
  let filePath = requestedFile;

  if (filePath) {
    try {
      const info = await stat(filePath);
      if (!info.isFile()) filePath = null;
    } catch {
      filePath = null;
    }
  }

  if (!filePath) {
    if (path.extname(pathname)) {
      response.writeHead(404);
      response.end("Not found");
      return;
    }
    filePath = path.join(publicDir, "index.html");
    try {
      await access(filePath);
    } catch {
      response.writeHead(404);
      response.end("Not found");
      return;
    }
  }

  response.writeHead(200, {
    "content-type": MIME_TYPES[path.extname(filePath)] ?? "application/octet-stream",
  });
  createReadStream(filePath).pipe(response);
}

function isCommandCenterApi(pathname) {
  return pathname === "/api/command-center" || pathname.startsWith("/api/command-center/");
}

async function proxyApi(request, response, apiUrl) {
  const target = new URL(request.url ?? "/", apiUrl);
  const headers = { ...request.headers, host: target.host };
  delete headers.connection;
  if (target.pathname === "/api/command-center/bootstrap") delete headers.origin;

  try {
    const upstream = await fetch(target, {
      method: request.method,
      headers,
      body: ["GET", "HEAD"].includes(request.method ?? "") ? undefined : request,
      duplex: "half",
      redirect: "manual",
    });
    const responseHeaders = Object.fromEntries(upstream.headers);
    response.writeHead(upstream.status, responseHeaders);
    if (upstream.body) {
      for await (const chunk of upstream.body) response.write(chunk);
    }
    response.end();
  } catch {
    response.writeHead(502, { "content-type": "text/plain; charset=utf-8" });
    response.end("Jcode Command Center API is unavailable");
  }
}

export function createCommandCenterServer({
  publicDir = path.join(thisDirectory, "dist"),
  apiUrl,
} = {}) {
  if (!apiUrl) throw new Error("apiUrl is required");

  return http.createServer((request, response) => {
    const pathname = new URL(request.url ?? "/", "http://command-center").pathname;
    if (pathname === "/healthz") {
      response.writeHead(200, { "content-type": "application/json; charset=utf-8" });
      response.end(JSON.stringify({ status: "ok" }));
      return;
    }
    if (isCommandCenterApi(pathname)) {
      void proxyApi(request, response, apiUrl);
      return;
    }
    if (pathname.startsWith("/api/")) {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end("Not found");
      return;
    }
    void serveStatic(request, response, publicDir);
  });
}

function parseBind(value) {
  const separator = value.lastIndexOf(":");
  const host = value.slice(0, separator);
  const port = Number(value.slice(separator + 1));
  if (!host || !Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`invalid JCODE_COMMAND_CENTER_UI_BIND: ${value}`);
  }
  return { host, port };
}

if (
  process.argv[1] &&
  realpathSync.native(process.argv[1]) === realpathSync.native(fileURLToPath(import.meta.url))
) {
  const bind = parseBind(process.env.JCODE_COMMAND_CENTER_UI_BIND ?? "127.0.0.1:43119");
  const server = createCommandCenterServer({
    publicDir: process.env.JCODE_COMMAND_CENTER_PUBLIC_DIR ?? path.join(thisDirectory, "public"),
    apiUrl: process.env.JCODE_COMMAND_CENTER_API_URL ?? "http://127.0.0.1:43118",
  });
  server.listen(bind.port, bind.host, () => {
    console.log(`Jcode Command Center listening on http://${bind.host}:${bind.port}`);
  });
  const shutdown = () => server.close(() => process.exit(0));
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);
}
