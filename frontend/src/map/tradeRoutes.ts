// Shared trade-route presentation, used by both the map overlay (유통.md
// "Trade Overlay") and the trade dashboard so the two stay in lockstep.
//
// The engine ships cross-border commerce on a single infrastructure factor — it
// does not yet model 도로/철도/해운/항공 as separate networks — so the transport
// tier shown here is a presentation-layer attribute: taken from the route when the
// data carries one (the hand-authored sample world does), else inferred from how
// far apart the two nations sit. Colours follow the 유통.md palette.

import type { TradeRouteView, TransportMode, WorldView } from "../types";

/** Pixi-friendly 0xRRGGBB colour per mode (유통.md palette). */
export const MODE_COLOR: Record<TransportMode, number> = {
  road: 0x8b93b8, // 도로 — 회색
  rail: 0xd9a441, // 철도 — 노란색
  sea: 0x4f86d6, // 해운 — 파란색
  air: 0x6fc6e8, // 항공 — 하늘색
};

/** CSS colour per mode (same palette, for SVG / HTML legends). */
export const MODE_CSS: Record<TransportMode, string> = {
  road: "#8b93b8",
  rail: "#d9a441",
  sea: "#4f86d6",
  air: "#6fc6e8",
};

/** Korean label per mode, for legends. */
export const MODE_LABEL: Record<TransportMode, string> = {
  road: "도로",
  rail: "철도",
  sea: "해운",
  air: "항공",
};

/** The order legends list the modes in: heaviest tier first (유통.md Tier 3→1). */
export const MODE_ORDER: TransportMode[] = ["sea", "rail", "road", "air"];

export interface Point {
  x: number;
  y: number;
}

/** Each nation placed at the centroid of its regions, in the map's 0..1 space —
 * the anchor a lane is drawn from. */
export function nationCentroids(world: WorldView): Map<string, Point> {
  const out = new Map<string, Point>();
  for (const n of world.nations) {
    const regs = world.regions.filter((r) => r.nationId === n.id);
    if (regs.length === 0) continue;
    const x = regs.reduce((s, r) => s + r.x, 0) / regs.length;
    const y = regs.reduce((s, r) => s + r.y, 0) / regs.length;
    out.set(n.id, { x, y });
  }
  return out;
}

/** Infer a transport tier from the gap between two nations when the route carries
 * no explicit mode: neighbours ship by 도로, a nation apart by 철도, the far-flung
 * by 해운 (유통.md Tier 1→3 by distance). */
function inferMode(dist: number): TransportMode {
  if (dist < 0.22) return "road";
  if (dist < 0.5) return "rail";
  return "sea";
}

/** The transport mode to draw a route in: explicit when the data sets it, else
 * inferred from the two nations' separation. */
export function routeMode(route: TradeRouteView, centroids: Map<string, Point>): TransportMode {
  if (route.mode) return route.mode;
  const a = centroids.get(route.a);
  const b = centroids.get(route.b);
  if (!a || !b) return "rail";
  return inferMode(Math.hypot(a.x - b.x, a.y - b.y));
}
