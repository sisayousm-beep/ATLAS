import { useEffect, useRef } from "react";
import { Application, Container, Graphics, Text, Rectangle } from "pixi.js";
import type { WorldView } from "../types";
import { MODE_COLOR, nationCentroids, routeMode } from "./tradeRoutes";

/** What the player has currently selected on the map (UI roadmap Phase 3). */
export type Selection =
  | { kind: "region"; id: number }
  | { kind: "nation"; id: string }
  | null;

interface Props {
  world: WorldView;
  selected: Selection;
  /** Phase 9+: draw the live trade-route network over the map (유통.md overlay). */
  showTrade: boolean;
  onSelectRegion: (id: number) => void;
  onSelectNation: (id: string) => void;
  onClear: () => void;
}

// The map lives in a fixed world coordinate space; the camera (a Pixi Container)
// scales and pans over it. Decoupling world coords from the screen means a tick
// re-render redraws the contents without disturbing the player's zoom/pan.
const WORLD_W = 1600;
const WORLD_H = 1000;
const GRID = 100;

/**
 * World Map Renderer (UI roadmap Phase 2) + Region/Nation inspection (Phase 3).
 *
 * Draws the world — terrain grid, nation-coloured territories with borders, and
 * region nodes — onto a PixiJS canvas, with a pan/zoom camera. Region nodes are
 * clickable (→ select region), territory discs are clickable (→ select owning
 * nation), and a tap on empty space clears the selection. The Pixi app and its
 * input handlers are built once on mount; only the contents are redrawn when a
 * new world snapshot or a new selection arrives, so the camera survives both.
 */
export function AtlasMap({ world, selected, showTrade, onSelectRegion, onSelectNation, onClear }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const appRef = useRef<Application | null>(null);
  const cameraRef = useRef<Container | null>(null);
  const drawRef = useRef<() => void>(() => {});
  const fitRef = useRef(1);
  // Latest world + selection + callbacks, read by draw() so the mount-once setup
  // always sees fresh data.
  const worldRef = useRef(world);
  worldRef.current = world;
  const selectedRef = useRef(selected);
  selectedRef.current = selected;
  const showTradeRef = useRef(showTrade);
  showTradeRef.current = showTrade;
  const onSelectRegionRef = useRef(onSelectRegion);
  onSelectRegionRef.current = onSelectRegion;
  const onSelectNationRef = useRef(onSelectNation);
  onSelectNationRef.current = onSelectNation;
  const onClearRef = useRef(onClear);
  onClearRef.current = onClear;

  // Mount once: build the Pixi app, the camera, and the pan/zoom handlers.
  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    let disposed = false;
    let teardown: (() => void) | null = null;

    // Drag state, shared between the pan handlers and the click handlers so a
    // pan-drag never registers as a selection click.
    let dragging = false;
    let moved = false;
    let lastX = 0;
    let lastY = 0;
    let downX = 0;
    let downY = 0;

    const draw = () => {
      const camera = cameraRef.current;
      if (!camera) return;
      camera.removeChildren();
      const w = worldRef.current;
      const sel = selectedRef.current;
      const nationColor = new Map(w.nations.map((n) => [n.id, n.color]));

      // --- terrain: a faint grid + world border, so panning has a frame of reference ---
      const terrain = new Graphics();
      for (let x = 0; x <= WORLD_W; x += GRID) terrain.moveTo(x, 0).lineTo(x, WORLD_H);
      for (let y = 0; y <= WORLD_H; y += GRID) terrain.moveTo(0, y).lineTo(WORLD_W, y);
      terrain.stroke({ color: 0x141b33, width: 1 });
      terrain.rect(0, 0, WORLD_W, WORLD_H).stroke({ color: 0x223056, width: 2 });
      camera.addChild(terrain);

      // --- nation territory: a coloured disc + border under each region; click → select nation ---
      for (const region of w.regions) {
        const color = nationColor.get(region.nationId) ?? 0x999999;
        const nationSel = sel?.kind === "nation" && sel.id === region.nationId;
        const land = new Graphics();
        land
          .circle(region.x * WORLD_W, region.y * WORLD_H, 130)
          .fill({ color, alpha: nationSel ? 0.22 : 0.13 })
          .stroke({ color, width: nationSel ? 3 : 2, alpha: nationSel ? 0.7 : 0.32 });
        land.eventMode = "static";
        land.cursor = "pointer";
        land.on("pointertap", (e) => {
          if (moved) return;
          e.stopPropagation();
          onSelectNationRef.current(region.nationId);
        });
        camera.addChild(land);
      }

      // --- trade routes (유통.md Trade Overlay): a line per nation-pair lane, drawn
      // between the nations' region centroids. Colour = transport mode, thickness ∝
      // this month's volume. Sits over the territory but under the region nodes, and
      // is non-interactive so it never steals a node/territory click. ---
      if (showTradeRef.current && w.tradeRoutes.length > 0) {
        const centroids = nationCentroids(w);
        const maxValue = w.tradeRoutes.reduce((m, r) => Math.max(m, r.value), 0);
        const routes = new Container();
        routes.eventMode = "none";
        for (const r of w.tradeRoutes) {
          const a = centroids.get(r.a);
          const b = centroids.get(r.b);
          if (!a || !b) continue;
          const color = MODE_COLOR[routeMode(r, centroids)];
          const width = 2 + (maxValue > 0 ? r.value / maxValue : 0) * 14;
          const lane = new Graphics();
          lane
            .moveTo(a.x * WORLD_W, a.y * WORLD_H)
            .lineTo(b.x * WORLD_W, b.y * WORLD_H)
            .stroke({ color, width, alpha: 0.72, cap: "round" });
          routes.addChild(lane);
        }
        camera.addChild(routes);
      }

      // --- region nodes: nation-coloured, sized by population, labelled; click → select region ---
      for (const region of w.regions) {
        const color = nationColor.get(region.nationId) ?? 0x999999;
        const radius = 14 + Math.sqrt(region.population) / 30;
        const regionSel = sel?.kind === "region" && sel.id === region.id;

        const node = new Container();
        node.x = region.x * WORLD_W;
        node.y = region.y * WORLD_H;
        node.eventMode = "static";
        node.cursor = "pointer";
        node.on("pointertap", (e) => {
          if (moved) return;
          e.stopPropagation();
          onSelectRegionRef.current(region.id);
        });

        const g = new Graphics();
        g.circle(0, 0, radius)
          .fill({ color, alpha: 0.9 })
          .stroke({ color: 0xffffff, width: 1.5, alpha: 0.45 });
        if (regionSel) {
          g.circle(0, 0, radius + 5).stroke({ color: 0xffffff, width: 2.5, alpha: 0.9 });
        }
        node.addChild(g);

        const label = new Text({
          text: `${region.name}\n${region.population.toLocaleString()}`,
          style: {
            fill: 0xe6e9f5,
            fontSize: 12,
            align: "center",
            fontFamily: "system-ui, sans-serif",
          },
        });
        label.anchor.set(0.5, 0);
        label.y = radius + 4;
        node.addChild(label);

        camera.addChild(node);
      }
    };
    drawRef.current = draw;

    (async () => {
      const app = new Application();
      await app.init({ background: "#0b1021", antialias: true, resizeTo: host });
      if (disposed) {
        app.destroy(true);
        return;
      }
      appRef.current = app;
      host.appendChild(app.canvas);

      const camera = new Container();
      app.stage.addChild(camera);
      cameraRef.current = camera;

      // Scale so the whole world fits, centred — the opening view.
      const fit = () => Math.min(app.screen.width / WORLD_W, app.screen.height / WORLD_H);
      const applyFit = () => {
        const f = fit();
        fitRef.current = f;
        camera.scale.set(f);
        camera.x = (app.screen.width - WORLD_W * f) / 2;
        camera.y = (app.screen.height - WORLD_H * f) / 2;
      };
      applyFit();

      // The stage needs a hit area covering the canvas so pointer events fire on
      // empty space (not just on region nodes), which is what makes panning work.
      app.stage.eventMode = "static";
      app.stage.hitArea = new Rectangle(0, 0, app.screen.width, app.screen.height);

      // --- pan: drag the camera ---
      app.stage.on("pointerdown", (e) => {
        dragging = true;
        moved = false;
        downX = lastX = e.global.x;
        downY = lastY = e.global.y;
        app.canvas.style.cursor = "grabbing";
      });
      const endDrag = () => {
        dragging = false;
        app.canvas.style.cursor = "grab";
      };
      app.stage.on("pointerup", endDrag);
      app.stage.on("pointerupoutside", endDrag);
      app.stage.on("globalpointermove", (e) => {
        if (!dragging) return;
        const gx = e.global.x;
        const gy = e.global.y;
        if (!moved && Math.hypot(gx - downX, gy - downY) > 4) moved = true;
        camera.x += gx - lastX;
        camera.y += gy - lastY;
        lastX = gx;
        lastY = gy;
      });

      // --- tap on empty space clears the selection (node/disc taps stop propagation) ---
      app.stage.on("pointertap", () => {
        if (!moved) onClearRef.current();
      });

      // --- zoom: wheel, anchored on the cursor ---
      const onWheel = (ev: WheelEvent) => {
        ev.preventDefault();
        const rect = app.canvas.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;
        const s = camera.scale.x;
        const factor = ev.deltaY < 0 ? 1.1 : 1 / 1.1;
        const s2 = Math.max(fitRef.current * 0.6, Math.min(fitRef.current * 8, s * factor));
        // Keep the world point under the cursor fixed while scaling.
        const wx = (px - camera.x) / s;
        const wy = (py - camera.y) / s;
        camera.scale.set(s2);
        camera.x = px - wx * s2;
        camera.y = py - wy * s2;
      };
      app.canvas.addEventListener("wheel", onWheel, { passive: false });
      app.canvas.style.cursor = "grab";

      const onResize = () => {
        app.stage.hitArea = new Rectangle(0, 0, app.screen.width, app.screen.height);
        fitRef.current = fit();
      };
      window.addEventListener("resize", onResize);

      teardown = () => {
        app.canvas.removeEventListener("wheel", onWheel);
        window.removeEventListener("resize", onResize);
      };

      draw();
    })();

    return () => {
      disposed = true;
      teardown?.();
      const app = appRef.current;
      if (app) {
        app.destroy(true, { children: true });
        appRef.current = null;
        cameraRef.current = null;
      }
    };
  }, []);

  // Redraw the contents whenever a new world snapshot or selection arrives. The
  // camera keeps its transform, so the player's pan/zoom is untouched.
  useEffect(() => {
    drawRef.current();
  }, [world, selected, showTrade]);

  return <div ref={hostRef} className="atlas-map" />;
}
