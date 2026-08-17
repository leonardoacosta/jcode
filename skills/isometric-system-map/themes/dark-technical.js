(() => {
  const roleColors = {
    pipeline: "#8b7cff",
    entry: "#8b7cff",
    governance: "#d48cff",
    module: "#4f82d8",
    network: "#2daac1",
    compute: "#4779d2",
    data: "#2c9a79",
    identity: "#b27ee8",
    messaging: "#d96f9f",
    observability: "#d69b4d",
    external: "#778397",
  };
  const pathStyles = {
    delivery: { color: "#7c9cff", width: 2.4, dash: [] },
    dependency: { color: "#7d8798", width: 1.45, dash: [7, 7] },
    control: { color: "#c77dff", width: 2.1, dash: [14, 4, 2, 4] },
    data: { color: "#35c89b", width: 3.2, dash: [] },
    network: { color: "#22b8cf", width: 2.6, dash: [12, 4] },
    identity: { color: "#d187ff", width: 2, dash: [3, 5] },
    telemetry: { color: "#e6aa5d", width: 2, dash: [2, 6] },
  };
  let scanPattern = null;

  function pattern(ctx) {
    if (scanPattern) return scanPattern;
    const tile = document.createElement("canvas");
    tile.width = 12;
    tile.height = 12;
    const tileContext = tile.getContext("2d");
    tileContext.strokeStyle = "rgba(255,255,255,.07)";
    tileContext.lineWidth = 1;
    tileContext.beginPath();
    tileContext.moveTo(0, 3.5);
    tileContext.lineTo(12, 3.5);
    tileContext.stroke();
    scanPattern = ctx.createPattern(tile, "repeat");
    return scanPattern;
  }

  return {
    id: "dark-technical",
    name: "Dark technical linework",
    css: {
      pageBackground: "#07090f",
      text: "#eef2fb",
      muted: "#8b96a9",
      hairline: "rgba(128,145,171,.35)",
      controlBackground: "rgba(8,11,18,.88)",
      controlHover: "rgba(101,126,168,.16)",
      focus: "#d9e5ff",
      fontFamily: 'ui-monospace, "Cascadia Code", "SFMono-Regular", Consolas, monospace',
    },
    motion: { stepDuration: 2800 },

    pathStyle(kind) {
      return pathStyles[kind] || pathStyles.dependency;
    },

    drawBackground(ctx, state) {
      const gradient = ctx.createRadialGradient(
        state.width * .53,
        state.height * .47,
        10,
        state.width * .53,
        state.height * .47,
        state.width * .72,
      );
      gradient.addColorStop(0, "#111725");
      gradient.addColorStop(.55, "#090d16");
      gradient.addColorStop(1, "#05070c");
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, state.width, state.height);
      ctx.save();
      ctx.globalAlpha = .22;
      ctx.strokeStyle = "#1b263b";
      ctx.lineWidth = 1;
      for (let y = 0; y < state.height; y += 5) {
        ctx.beginPath();
        ctx.moveTo(0, y + .5);
        ctx.lineTo(state.width, y + .5);
        ctx.stroke();
      }
      ctx.restore();
    },

    drawGround(ctx, ground) {
      ctx.fillStyle = "rgba(8,13,22,.72)";
      ctx.fill(ground);
      ctx.strokeStyle = "rgba(105,130,171,.62)";
      ctx.lineWidth = 1.1;
      ctx.stroke(ground);
    },

    drawGridLine(ctx, path, boundary) {
      ctx.strokeStyle = boundary ? "rgba(111,137,177,.46)" : "rgba(78,99,133,.25)";
      ctx.lineWidth = boundary ? 1 : .8;
      ctx.setLineDash([]);
      ctx.stroke(path);
    },

    drawZone(ctx, zone, point) {
      ctx.save();
      ctx.fillStyle = "rgba(142,161,190,.52)";
      ctx.font = "600 9px ui-monospace, monospace";
      ctx.textAlign = "center";
      ctx.letterSpacing = "1px";
      ctx.fillText(zone.label.toUpperCase(), point.x, point.y + 28);
      ctx.restore();
    },

    drawRoute(ctx, path, item) {
      const style = this.pathStyle(item.kind);
      ctx.save();
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.setLineDash(style.dash);
      ctx.strokeStyle = "rgba(3,6,11,.9)";
      ctx.lineWidth = style.width + 3.5;
      ctx.stroke(path);
      ctx.strokeStyle = style.color;
      ctx.lineWidth = style.width;
      ctx.stroke(path);
      ctx.restore();
    },

    drawArrow(ctx, points, item) {
      if (points.length < 2) return;
      const style = this.pathStyle(item.kind);
      const end = points[points.length - 1];
      const before = points[points.length - 2];
      const angle = Math.atan2(end.y - before.y, end.x - before.x);
      const size = 7 + style.width;
      ctx.save();
      ctx.translate(end.x, end.y);
      ctx.rotate(angle);
      ctx.fillStyle = style.color;
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(-size, -size * .48);
      ctx.lineTo(-size, size * .48);
      ctx.closePath();
      ctx.fill();
      ctx.restore();
    },

    drawPathLabel() {},

    drawFace(ctx, path, side, node, massIndex) {
      const base = roleColors[node.role] || roleColors.module;
      ctx.save();
      ctx.fillStyle = base;
      ctx.globalAlpha = side === "roof" ? .46 : side === "left" ? .26 : .34;
      ctx.fill(path);
      ctx.globalAlpha = side === "roof" ? .2 : .11;
      ctx.fillStyle = pattern(ctx);
      ctx.fill(path);
      ctx.globalAlpha = node.status === "held" ? .42 : .9;
      ctx.strokeStyle = node.status === "held" ? "#a7afbd" : base;
      ctx.lineWidth = side === "roof" ? 1.25 : .9;
      ctx.setLineDash(node.status === "held" ? [4, 4] : []);
      ctx.stroke(path);
      if (side === "roof" && massIndex === 0) {
        ctx.globalAlpha = .2;
        ctx.shadowColor = base;
        ctx.shadowBlur = 10;
        ctx.stroke(path);
      }
      ctx.restore();
    },

    drawNodeLabel(ctx, node, point) {
      const accent = roleColors[node.role] || roleColors.module;
      ctx.save();
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.font = "700 9px ui-monospace, monospace";
      const codeWidth = ctx.measureText(node.code).width + 12;
      ctx.fillStyle = "rgba(5,8,14,.88)";
      ctx.fillRect(point.x - codeWidth / 2, point.y - 12, codeWidth, 16);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 1;
      ctx.strokeRect(point.x - codeWidth / 2 + .5, point.y - 11.5, codeWidth - 1, 15);
      ctx.fillStyle = "#f2f5fb";
      ctx.fillText(node.code, point.x, point.y - 4);
      ctx.restore();
    },

    drawPayload(ctx, payload) {
      const color = payload.payload?.kind === "telemetry" ? "#e6aa5d" : "#9bb8ff";
      ctx.save();
      ctx.shadowColor = color;
      ctx.shadowBlur = 12;
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 4.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.shadowBlur = 0;
      ctx.strokeStyle = "rgba(255,255,255,.82)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 7.5, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    },

    drawSelection(ctx, path, kind, mode) {
      ctx.save();
      ctx.strokeStyle = mode === "hover" ? "rgba(255,255,255,.62)" : "rgba(169,194,255,.92)";
      ctx.lineWidth = kind === "path" ? 7 : 2.2;
      ctx.setLineDash(mode === "hover" ? [3, 4] : []);
      ctx.stroke(path);
      ctx.restore();
    },
  };
})()
