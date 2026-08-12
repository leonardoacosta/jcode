#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIAGRAM_DIR="$ROOT/docs/diagrams"
DIAGRAM_FILE="orca-command-center-architecture.html"
ARTIFACT_DIR="$ROOT/artifacts"
SCREENSHOT="$ARTIFACT_DIR/command-center-architecture-mobile.png"
PORT="${PORT:-49352}"
HOST="127.0.0.1"

if [[ ! -f "$DIAGRAM_DIR/$DIAGRAM_FILE" ]]; then
  echo "missing diagram: $DIAGRAM_DIR/$DIAGRAM_FILE" >&2
  exit 1
fi

mkdir -p "$ARTIFACT_DIR"

python3 -m http.server "$PORT" --bind "$HOST" --directory "$DIAGRAM_DIR" >/tmp/jcode-command-center-diagram-server.log 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1 || true
  wait "$SERVER_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for _ in {1..50}; do
  if python3 - "$HOST" "$PORT" <<'PY' >/dev/null 2>&1
import socket, sys
host, port = sys.argv[1], int(sys.argv[2])
with socket.create_connection((host, port), timeout=0.2):
    pass
PY
  then
    break
  fi
  sleep 0.1
done

NODE_PATH="$ROOT/apps/command-center/node_modules" \
CHROMIUM_EXECUTABLE="${CHROMIUM_EXECUTABLE:-/usr/bin/chromium}" \
DIAGRAM_URL="http://$HOST:$PORT/$DIAGRAM_FILE" \
SCREENSHOT="$SCREENSHOT" \
node <<'NODE'
const { chromium } = require('@playwright/test');
const executablePath = process.env.CHROMIUM_EXECUTABLE;
const url = process.env.DIAGRAM_URL;
const screenshot = process.env.SCREENSHOT;

(async () => {
  const browser = await chromium.launch({ executablePath, args: ['--no-sandbox'] });
  try {
    const page = await browser.newPage({ viewport: { width: 393, height: 852 }, deviceScaleFactor: 1, isMobile: true });
    await page.goto(url, { waitUntil: 'networkidle' });
    await page.screenshot({ path: screenshot, fullPage: true });
    const result = await page.evaluate(() => {
      const scrollWidth = Math.ceil(document.documentElement.scrollWidth);
      const innerWidth = window.innerWidth;
      const fixed = [...document.querySelectorAll('*')].filter((el) => getComputedStyle(el).position === 'fixed');
      const headings = [...document.querySelectorAll('h1,h2,h3')];
      const intersections = [];
      for (const fixedEl of fixed) {
        const a = fixedEl.getBoundingClientRect();
        if (a.width === 0 || a.height === 0) continue;
        for (const heading of headings) {
          const b = heading.getBoundingClientRect();
          const intersects = a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
          if (intersects) intersections.push({ fixed: fixedEl.className || fixedEl.tagName, heading: heading.textContent.trim() });
        }
      }
      return { scrollWidth, innerWidth, intersections };
    });
    if (result.scrollWidth !== result.innerWidth) {
      throw new Error(`mobile horizontal overflow: scrollWidth=${result.scrollWidth} innerWidth=${result.innerWidth}`);
    }
    if (result.intersections.length) {
      throw new Error(`fixed element intersects section heading: ${JSON.stringify(result.intersections)}`);
    }
    console.log(`PASS command center architecture mobile smoke: ${screenshot}`);
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
NODE
