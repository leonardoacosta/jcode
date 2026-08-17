(() => {
  const azureBlue = "#0078d4";
  const families = {
    compute: { stroke: "#c8460e", fill: "#fde6d4" },
    data: { stroke: "#107c10", fill: "#d7ebd7" },
    identity: { stroke: "#886200", fill: "#fbeec7" },
    integration: { stroke: "#5933a3", fill: "#ece4f7" },
    network: { stroke: "#006da3", fill: "#d4e9f6" },
    monitor: { stroke: "#3a3d99", fill: "#dadcf2" },
    governance: { stroke: "#5a6470", fill: "#ebedf0" },
    devops: { stroke: "#a02763", fill: "#f6d8e6" },
  };
  const resourceTypeFamilies = {
    "Microsoft.Web/sites": "compute",
    "Microsoft.Web/serverFarms": "compute",
    "Microsoft.App/containerApps": "compute",
    "Microsoft.App/managedEnvironments": "compute",
    "Microsoft.ContainerService/managedClusters": "compute",
    "Microsoft.Compute/virtualMachines": "compute",
    "Microsoft.Sql/servers": "data",
    "Microsoft.Sql/servers/databases": "data",
    "Microsoft.Sql/servers/elasticPools": "data",
    "Microsoft.DocumentDB/databaseAccounts": "data",
    "Microsoft.Storage/storageAccounts": "data",
    "Microsoft.Cache/Redis": "data",
    "Microsoft.KeyVault/vaults": "identity",
    "Microsoft.ManagedIdentity/userAssignedIdentities": "identity",
    "Microsoft.AppConfiguration/configurationStores": "identity",
    "Microsoft.ApiManagement/service": "integration",
    "Microsoft.ApiManagement/service/apis": "integration",
    "Microsoft.ApiManagement/service/groups": "integration",
    "Microsoft.ApiManagement/service/products": "integration",
    "Microsoft.Cdn/profiles": "integration",
    "Microsoft.Network/applicationGateways": "integration",
    "Microsoft.ServiceBus/namespaces": "integration",
    "Microsoft.EventGrid/topics": "integration",
    "Microsoft.Network/virtualNetworks": "network",
    "Microsoft.Network/virtualNetworks/subnets": "network",
    "Microsoft.Network/virtualNetworks/virtualNetworkPeerings": "network",
    "Microsoft.Network/privateEndpoints": "network",
    "Microsoft.Network/privateDnsZones": "network",
    "Microsoft.Network/networkSecurityGroups": "network",
    "Microsoft.Network/azureFirewalls": "network",
    "Microsoft.Insights/components": "monitor",
    "Microsoft.Insights/metricAlerts": "monitor",
    "Microsoft.Insights/privateLinkScopes": "monitor",
    "Microsoft.OperationalInsights/workspaces": "monitor",
    "Microsoft.Resources/resourceGroups": "governance",
    "Microsoft.Resources/subscriptions/resourceGroups": "governance",
    "Microsoft.ContainerRegistry/registries": "devops",
    "Azure DevOps pipeline": "devops",
  };
  const roleFamilies = {
    entry: "devops",
    pipeline: "devops",
    governance: "governance",
    module: "governance",
    network: "network",
    compute: "compute",
    data: "data",
    identity: "identity",
    messaging: "integration",
    observability: "monitor",
    external: "governance",
  };
  const pathStyles = {
    dependency: { color: azureBlue, width: 2, dash: [], arrow: "filled" },
    network: { color: "#235dc1", width: 2.2, dash: [], arrow: "both" },
    data: { color: "#107c10", width: 2.4, dash: [], arrow: "filled" },
    identity: { color: "#886200", width: 2, dash: [5, 3], arrow: "open" },
    control: { color: "#5933a3", width: 2, dash: [2, 3], arrow: "filled" },
    telemetry: { color: "#3a3d99", width: 2, dash: [5, 3], arrow: "open" },
    delivery: { color: "#a02763", width: 2.4, dash: [], arrow: "filled" },
  };
  const trafficColors = ["#0078d4", "#5933a3", "#107c10", "#c8460e"];

  function familyFor(node) {
    return resourceTypeFamilies[node.resource_type] || roleFamilies[node.role] || "governance";
  }

  function colorsFor(node) {
    return families[familyFor(node)];
  }

  function rectanglesOverlap(first, second) {
    return first.left < second.right && first.right > second.left
      && first.top < second.bottom && first.bottom > second.top;
  }

  function resolveLabelOverlap(rect, state) {
    const existing = state.labelPlates || [];
    const offsets = [0, -24, 24, -48, 48, -72, 72];
    for (const offset of offsets) {
      const candidate = {
        left: rect.left,
        right: rect.right,
        top: rect.top + offset,
        bottom: rect.bottom + offset,
        offset,
      };
      if (!existing.some(other => rectanglesOverlap(candidate, other))) {
        existing.push(candidate);
        state.labelPlates = existing;
        return candidate;
      }
    }
    existing.push(rect);
    state.labelPlates = existing;
    return rect;
  }

  function arrowHead(ctx, point, angle, style, reverse = false) {
    const size = 7 + style.width;
    ctx.save();
    ctx.translate(point.x, point.y);
    ctx.rotate(angle + (reverse ? Math.PI : 0));
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(-size, -size * .48);
    ctx.lineTo(-size, size * .48);
    if (style.arrow === "open") {
      ctx.strokeStyle = style.color;
      ctx.lineWidth = style.width;
      ctx.stroke();
    } else {
      ctx.closePath();
      ctx.fillStyle = style.color;
      ctx.fill();
    }
    ctx.restore();
  }

  return {
    id: "azure-topology",
    name: "Azure topology resource blocks",
    css: {
      pageBackground: "#ffffff",
      text: "#212121",
      muted: "#57606a",
      hairline: "rgba(87,96,106,.35)",
      controlBackground: "rgba(255,255,255,.94)",
      controlHover: "#eef5fb",
      focus: azureBlue,
      fontFamily: '"Segoe UI", system-ui, sans-serif',
    },
    azureBlue: "#0078d4",
    motion: { stepDuration: 3000 },

    pathStyle(kind) {
      return pathStyles[kind] || pathStyles.dependency;
    },

    iconColor(node) {
      return colorsFor(node).stroke;
    },

    iconWorldSize(mass) {
      const iconWorldSize = Math.min(mass.width, mass.depth) * .78;
      return iconWorldSize;
    },

    drawBackground(ctx, state) {
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, state.width, state.height);
      const wash = ctx.createRadialGradient(
        state.width * .52,
        state.height * .5,
        20,
        state.width * .52,
        state.height * .5,
        state.width * .7,
      );
      wash.addColorStop(0, "rgba(0,120,212,.045)");
      wash.addColorStop(1, "rgba(255,255,255,0)");
      ctx.fillStyle = wash;
      ctx.fillRect(0, 0, state.width, state.height);
    },

    drawGround(ctx, ground) {
      ctx.fillStyle = "rgba(250,252,255,.92)";
      ctx.fill(ground);
      ctx.strokeStyle = "rgba(0,120,212,.62)";
      ctx.lineWidth = 1.25;
      ctx.stroke(ground);
    },

    drawGridLine(ctx, path, boundary) {
      ctx.strokeStyle = boundary ? "rgba(0,120,212,.48)" : "rgba(87,96,106,.17)";
      ctx.lineWidth = boundary ? 1 : .75;
      ctx.setLineDash([]);
      ctx.stroke(path);
    },

    drawTrafficLayer(ctx, path, layer, index, labelPoint) {
      const color = trafficColors[index] || azureBlue;
      ctx.save();
      ctx.globalAlpha = .08;
      ctx.fillStyle = color;
      ctx.fill(path);
      ctx.globalAlpha = .72;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.15;
      ctx.setLineDash([3, 5]);
      ctx.stroke(path);
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
      ctx.fillStyle = color;
      ctx.font = '700 9px "Segoe UI", system-ui, sans-serif';
      ctx.textAlign = "center";
      ctx.fillText(`${index + 1} · ${layer.label.toUpperCase()}`, labelPoint.x, labelPoint.y + 4);
      ctx.restore();
    },

    drawTrafficDirection(ctx, points, traffic) {
      if (points.length < 2) return;
      const [start, end] = points;
      const angle = Math.atan2(end.y - start.y, end.x - start.x);
      const midpoint = { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 };
      ctx.save();
      ctx.strokeStyle = "rgba(0,120,212,.56)";
      ctx.lineWidth = 2;
      ctx.setLineDash([8, 5]);
      ctx.beginPath();
      ctx.moveTo(start.x, start.y);
      ctx.lineTo(end.x, end.y);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.translate(end.x, end.y);
      ctx.rotate(angle);
      ctx.fillStyle = azureBlue;
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(-11, -5);
      ctx.lineTo(-11, 5);
      ctx.closePath();
      ctx.fill();
      ctx.restore();
      ctx.save();
      ctx.fillStyle = "rgba(255,255,255,.92)";
      ctx.fillRect(midpoint.x - 52, midpoint.y - 20, 104, 15);
      ctx.fillStyle = azureBlue;
      ctx.font = '700 8px "Cascadia Code", Consolas, monospace';
      ctx.textAlign = "center";
      ctx.fillText(traffic.label, midpoint.x, midpoint.y - 10);
      ctx.restore();
    },

    drawArea(ctx, path, area, labelPoint) {
      ctx.save();
      ctx.fillStyle = area.status === "held" ? "rgba(0,120,212,.035)" : "rgba(0,120,212,.075)";
      ctx.fill(path);
      ctx.strokeStyle = area.status === "held" ? "rgba(0,109,163,.56)" : "rgba(0,109,163,.88)";
      ctx.lineWidth = 1.5;
      ctx.setLineDash(area.status === "held" ? [6, 4] : []);
      ctx.stroke(path);
      ctx.setLineDash([]);
      ctx.fillStyle = "#006da3";
      ctx.font = '700 10px "Segoe UI", system-ui, sans-serif';
      ctx.textAlign = "center";
      ctx.fillText(area.label.toUpperCase(), labelPoint.x, labelPoint.y - 10);
      ctx.restore();
    },

    drawZone(ctx, zone, point) {
      ctx.save();
      ctx.fillStyle = "#57606a";
      ctx.font = '600 10px "Segoe UI", system-ui, sans-serif';
      ctx.textAlign = "center";
      ctx.fillText(zone.label.toUpperCase(), point.x, point.y + 28);
      ctx.restore();
    },

    drawRoute(ctx, path, item) {
      const style = this.pathStyle(item.kind);
      ctx.save();
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.setLineDash(item.evidence_level === "inferred" ? [10, 5] : item.evidence_level === "held" ? [3, 5] : style.dash);
      ctx.globalAlpha = item.evidence_level === "held" ? .48 : 1;
      ctx.strokeStyle = "rgba(255,255,255,.94)";
      ctx.lineWidth = style.width + 4;
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
      arrowHead(ctx, end, Math.atan2(end.y - before.y, end.x - before.x), style);
      if (style.arrow === "both") {
        const start = points[0];
        const after = points[1];
        arrowHead(ctx, start, Math.atan2(after.y - start.y, after.x - start.x), style, true);
      }
    },

    drawPathLabel(ctx, points, item) {
      if (points.length < 2) return;
      let best = null;
      for (let index = 1; index < points.length; index += 1) {
        const start = points[index - 1];
        const end = points[index];
        const length = Math.hypot(end.x - start.x, end.y - start.y);
        if (!best || length > best.length) best = { start, end, length };
      }
      if (!best || best.length < 72) return;
      const evidenceLabel = item.evidence_level === "inferred"
        ? "INFERRED"
        : item.evidence_level === "held"
          ? "HELD · NOT DEPLOYED"
          : "";
      const label = evidenceLabel || item.kind.toUpperCase();
      const x = (best.start.x + best.end.x) / 2;
      const y = (best.start.y + best.end.y) / 2 - 7;
      ctx.save();
      ctx.font = '600 8px "Cascadia Code", Consolas, monospace';
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const width = ctx.measureText(label).width + 8;
      ctx.fillStyle = "rgba(255,255,255,.92)";
      ctx.fillRect(x - width / 2, y - 7, width, 14);
      ctx.fillStyle = "#57606a";
      ctx.fillText(label, x, y);
      ctx.restore();
    },

    drawFace(ctx, path, side, node) {
      const colors = colorsFor(node);
      ctx.save();
      ctx.fillStyle = colors.fill;
      ctx.fill(path);
      if (side !== "roof") {
        ctx.fillStyle = side === "left" ? "rgba(33,33,33,.10)" : "rgba(33,33,33,.055)";
        ctx.fill(path);
      }
      ctx.globalAlpha = node.status === "held" ? .58 : 1;
      ctx.strokeStyle = colors.stroke;
      ctx.lineWidth = side === "roof" ? 1.35 : 1;
      ctx.setLineDash(node.status === "held" ? [5, 4] : []);
      ctx.stroke(path);
      ctx.restore();
    },

    drawNodeLabel(ctx, node, point, state) {
      const colors = colorsFor(node);
      const label = node.label || node.code;
      const resourceType = node.resource_type || node.role;
      const status = node.status || "unknown";
      ctx.save();
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.font = '700 10px "Segoe UI", system-ui, sans-serif';
      const labelWidth = ctx.measureText(label).width;
      ctx.font = '600 7px "Segoe UI", system-ui, sans-serif';
      const typeWidth = ctx.measureText(resourceType).width;
      const statusWidth = ctx.measureText(status).width;
      const labelBoxWidth = Math.max(labelWidth, typeWidth, statusWidth) + 14;
      // Keep full labels visible and move their plates instead of abbreviating.
      const labelRect = resolveLabelOverlap({
        left: point.x - labelBoxWidth / 2,
        right: point.x + labelBoxWidth / 2,
        top: point.y - 18,
        bottom: point.y + 17,
      }, state);
      const labelOffset = labelRect.offset || 0;
      const labelPoint = { x: point.x, y: point.y + labelOffset };
      ctx.fillStyle = "rgba(255,255,255,.95)";
      ctx.fillRect(labelPoint.x - labelBoxWidth / 2, labelPoint.y - 18, labelBoxWidth, 35);
      ctx.strokeStyle = colors.stroke;
      ctx.lineWidth = 1;
      ctx.strokeRect(labelPoint.x - labelBoxWidth / 2 + .5, labelPoint.y - 17.5, labelBoxWidth - 1, 34);
      ctx.fillStyle = "#212121";
      ctx.font = '700 10px "Segoe UI", system-ui, sans-serif';
      ctx.fillText(node.label || node.code, labelPoint.x, labelPoint.y - 10);
      ctx.fillStyle = "#57606a";
      ctx.font = '600 7px "Segoe UI", system-ui, sans-serif';
      ctx.fillText(node.resource_type || node.role, labelPoint.x, labelPoint.y + 1);
      ctx.fillStyle = colors.stroke;
      ctx.fillText(status, labelPoint.x, labelPoint.y + 11);
      ctx.restore();
    },

    drawPayload(ctx, payload) {
      const style = this.pathStyle(payload.path?.kind || "dependency");
      ctx.save();
      ctx.fillStyle = "#ffffff";
      ctx.strokeStyle = style.color;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = style.color;
      ctx.beginPath();
      ctx.arc(payload.point.x, payload.point.y, 2.1, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    },

    drawSelection(ctx, path, kind, mode) {
      ctx.save();
      ctx.strokeStyle = mode === "hover" ? "rgba(0,120,212,.58)" : "rgba(0,120,212,.94)";
      ctx.lineWidth = kind === "path" ? 7 : 2.5;
      ctx.setLineDash(mode === "hover" ? [4, 4] : []);
      ctx.stroke(path);
      ctx.restore();
    },
  };
})()
