(() => {
  const pathStyles = {
    delivery: { color: "#6e5530", width: 2.2, dash: [] },
    dependency: { color: "#857a5e", width: 1.35, dash: [7, 6] },
    control: { color: "#68483e", width: 2, dash: [13, 4, 2, 4] },
    data: { color: "#496655", width: 3, dash: [] },
    network: { color: "#536966", width: 2.4, dash: [11, 4] },
    identity: { color: "#725a69", width: 1.9, dash: [3, 5] },
    telemetry: { color: "#8a6339", width: 1.9, dash: [2, 6] },
  };
  const roleTints = {
    pipeline: "#b7a276",
    entry: "#b7a276",
    governance: "#b49a91",
    module: "#b6aa86",
    network: "#91a39a",
    compute: "#a7aa98",
    data: "#8fa08a",
    identity: "#ae9aa3",
    messaging: "#b39c8c",
    observability: "#b29d72",
    external: "#aaa58f",
  };
  let paperPattern = null;
  let hatchPattern = null;

  function makePaperPattern(ctx) {
    if (paperPattern) return paperPattern;
    const tile = document.createElement("canvas");
    tile.width = 64;
    tile.height = 64;
    const tileContext = tile.getContext("2d");
    tileContext.clearRect(0, 0, 64, 64);
    for (let index = 0; index < 44; index += 1) {
      const x = (index * 29 + 7) % 64;
      const y = (index * 43 + 11) % 64;
      const alpha = .025 + (index % 5) * .008;
      tileContext.fillStyle = `rgba(70,58,35,${alpha})`;
      tileContext.fillRect(x, y, index % 3 === 0 ? 2 : 1, 1);
    }
    paperPattern = ctx.createPattern(tile, "repeat");
    return paperPattern;
  }

  function makeHatchPattern(ctx) {
    if (hatchPattern) return hatchPattern;
    const tile = document.createElement("canvas");
    tile.width = 10;
    tile.height = 10;
    const tileContext = tile.getContext("2d");
    tileContext.strokeStyle = "rgba(69,59,40,.12)";
    tileContext.lineWidth = .8;
    tileContext.beginPath();
    tileContext.moveTo(-2, 10);
    tileContext.lineTo(10, -2);
    tileContext.moveTo(4, 12);
    tileContext.lineTo(12, 4);
    tileContext.stroke();
    hatchPattern = ctx.createPattern(tile, "repeat");
    return hatchPattern;
  }

  return {
    id: "warm-paper",
    name: "Warm archival paper",
    css: {
      pageBackground: "#d9cfaa",
      text: "#312d22",
      muted: "#716a54",
      hairline: "rgba(77,68,48,.38)",
      controlBackground: "rgba(224,215,181,.93)",
      controlHover: "rgba(87,73,44,.1)",
      focus: "#2f2a1f",
      fontFamily: '"Courier New", Courier, ui-monospace, monospace',
    },
    motion: { stepDuration: 3300 },

    pathStyle(kind) {
      return pathStyles[kind] || pathStyles.dependency;
    },

    drawBackground(ctx, state) {
      const gradient = ctx.createLinearGradient(0, 0, state.width, state.height);
      gradient.addColorStop(0, "#e1d7b3");
      gradient.addColorStop(.55, "#d6cba5");
      gradient.addColorStop(1, "#cfc39a");
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, state.width, state.height);
      ctx.globalAlpha = .85;
      ctx.fillStyle = makePaperPattern(ctx);
      ctx.fillRect(0, 0, state.width, state.height);
      ctx.globalAlpha = 1;
      ctx.strokeStyle = "rgba(83,70,44,.12)";
      ctx.lineWidth = 1;
      for (let y = 18; y < state.height; y += 34) {
        ctx.beginPath();
        ctx.moveTo(0, y + .5);
        ctx.lineTo(state.width, y + .5);
        ctx.stroke();
      }
    },

    drawGround(ctx, ground) {
      ctx.fillStyle = "rgba(226,217,181,.35)";
      ctx.fill(ground);
      ctx.strokeStyle = "rgba(77,68,48,.48)";
      ctx.lineWidth = 1.15;
      ctx.stroke(ground);
    },

    drawGridLine(ctx, path, boundary) {
      ctx.strokeStyle = boundary ? "rgba(82,70,45,.38)" : "rgba(93,80,52,.17)";
      ctx.lineWidth = boundary ? 1 : .75;
      ctx.setLineDash([]);
      ctx.stroke(path);
    },

    drawArea(ctx, path, area, labelPoint) {
      ctx.save();
      ctx.fillStyle = area.status === "held" ? "rgba(74,90,95,.035)" : "rgba(74,90,95,.08)";
      ctx.fill(path);
      ctx.strokeStyle = area.status === "held" ? "rgba(67,86,92,.48)" : "rgba(67,86,92,.78)";
      ctx.lineWidth = 1.25;
      ctx.setLineDash(area.status === "held" ? [6, 4] : []);
      ctx.stroke(path);
      ctx.setLineDash([]);
      ctx.fillStyle = "rgba(58,72,76,.78)";
      ctx.font = "700 9px Courier New, ui-monospace, monospace";
      ctx.textAlign = "center";
      ctx.fillText(area.label.toUpperCase(), labelPoint.x, labelPoint.y - 10);
      ctx.restore();
    },

    drawZone(ctx, zone, point) {
      ctx.save();
      ctx.fillStyle = "rgba(74,64,44,.56)";
      ctx.font = "700 9px Courier New, ui-monospace, monospace";
      ctx.textAlign = "center";
      ctx.fillText(zone.label.toUpperCase(), point.x, point.y + 28);
      ctx.restore();
    },

    drawRoute(ctx, path, item) {
      const style = this.pathStyle(item.kind);
      ctx.save();
      ctx.lineCap = item.kind === "data" ? "round" : "square";
      ctx.lineJoin = "round";
      ctx.setLineDash(style.dash);
      ctx.strokeStyle = "rgba(231,221,185,.72)";
      ctx.lineWidth = style.width + 3;
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
      ctx.strokeStyle = style.color;
      ctx.lineWidth = 1.6;
      ctx.beginPath();
      ctx.moveTo(-size, -size * .48);
      ctx.lineTo(0, 0);
      ctx.lineTo(-size, size * .48);
      ctx.stroke();
      ctx.restore();
    },

    drawPathLabel() {},

    drawFace(ctx, path, side, node) {
      const tint = roleTints[node.role] || roleTints.module;
      ctx.save();
      ctx.fillStyle = tint;
      ctx.globalAlpha = side === "roof" ? .42 : side === "left" ? .25 : .32;
      ctx.fill(path);
      if (side !== "roof") {
        ctx.globalAlpha = side === "left" ? .55 : .34;
        ctx.fillStyle = makeHatchPattern(ctx);
        ctx.fill(path);
      }
      ctx.globalAlpha = node.status === "held" ? .48 : .92;
      ctx.strokeStyle = node.status === "held" ? "#786f59" : "#514936";
      ctx.lineWidth = side === "roof" ? 1.1 : .85;
      ctx.setLineDash(node.status === "held" ? [4, 4] : []);
      ctx.stroke(path);
      ctx.restore();
    },

    drawNodeLabel(ctx, node, point) {
      ctx.save();
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.font = "700 9px Courier New, ui-monospace, monospace";
      const codeWidth = ctx.measureText(node.code).width + 12;
      ctx.fillStyle = "rgba(224,215,180,.9)";
      ctx.fillRect(point.x - codeWidth / 2, point.y - 12, codeWidth, 16);
      ctx.strokeStyle = "rgba(69,60,42,.82)";
      ctx.lineWidth = .9;
      ctx.strokeRect(point.x - codeWidth / 2 + .5, point.y - 11.5, codeWidth - 1, 15);
      ctx.fillStyle = "#342f23";
      ctx.fillText(node.code, point.x, point.y - 4);
      ctx.restore();
    },

    drawPayload(ctx, payload) {
      ctx.save();
      ctx.fillStyle = payload.payload?.kind === "telemetry" ? "#80582f" : "#3e3a2b";
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 4.4, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "rgba(62,55,38,.66)";
      ctx.lineWidth = 1;
      ctx.setLineDash([1, 3]);
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 8.3, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    },

    drawSelection(ctx, path, kind, mode) {
      ctx.save();
      ctx.strokeStyle = mode === "hover" ? "rgba(48,43,31,.58)" : "rgba(45,39,27,.9)";
      ctx.lineWidth = kind === "path" ? 6 : 2;
      ctx.setLineDash(mode === "hover" ? [2, 4] : [8, 3]);
      ctx.stroke(path);
      ctx.restore();
    },
  };
})()
