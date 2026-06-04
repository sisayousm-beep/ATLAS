import { useEffect, useRef } from "react";
import { Application, Container, Graphics, Text, Rectangle } from "pixi.js";
import type { WorldView } from "../types";

interface Props {
  world: WorldView;
}

// The map lives in a fixed world coordinate space; the camera (a Pixi Container)
// scales and pans over it. Decoupling world coords from the screen means a tick
// re-render redraws the contents without disturbing the player's zoom/pan.
const WORLD_W = 1600;
const WORLD_H = 1000;
const GRID = 100;

/**
 * World Map Renderer (UI roadmap Phase 2).
 *
 * Draws the world — terrain grid, nation-coloured territories with borders, and
 * region nodes — onto a PixiJS canvas, with a pan/zoom camera. The Pixi app and
 * its input handlers are built once on mount; only the contents are redrawn when
 * a new world snapshot arrives, so the camera survives every tick.
 */
export function AtlasMap({ world }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const appRef = useRef<Application | null>(null);
  const cameraRef = useRef<Container | null>(null);
  const drawRef = useRef<() => void>(() => {});
  const fitRef = useRef(1);
  // The latest world, read by draw() so the mount-once setup always sees fresh data.
  const worldRef = useRef(world);
  worldRef.current = world;

  // Mount once: build the Pixi app, the camera, and the pan/zoom handlers.
  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    let disposed = false;
    let teardown: (() => void) | null = null;

    const draw = () => {
      const camera = cameraRef.current;
      if (!camera) return;
      camera.removeChildren();
      const w = worldRef.current;
      const nationColor = new Map(w.nations.map((n) => [n.id, n.color]));

      // --- terrain: a faint grid + world border, so panning has a frame of reference ---
      const terrain = new Graphics();
      for (let x = 0; x <= WORLD_W; x += GRID) terrain.moveTo(x, 0).lineTo(x, WORLD_H);
      for (let y = 0; y <= WORLD_H; y += GRID) terrain.moveTo(0, y).lineTo(WORLD_W, y);
      terrain.stroke({ color: 0x141b33, width: 1 });
      terrain.rect(0, 0, WORLD_W, WORLD_H).stroke({ color: 0x223056, width: 2 });
      camera.addChild(terrain);

      // --- nation territory: a coloured disc + border under each region ---
      for (const region of w.regions) {
        const color = nationColor.get(region.nationId) ?? 0x999999;
        const land = new Graphics();
        land
          .circle(region.x * WORLD_W, region.y * WORLD_H, 130)
          .fill({ color, alpha: 0.13 })
          .stroke({ color, width: 2, alpha: 0.32 });
        camera.addChild(land);
      }

      // --- region nodes: nation-coloured, sized by population, labelled ---
      for (const region of w.regions) {
        const color = nationColor.get(region.nationId) ?? 0x999999;
        const radius = 14 + Math.sqrt(region.population) / 30;

        const node = new Container();
        node.x = region.x * WORLD_W;
        node.y = region.y * WORLD_H;

        const g = new Graphics();
        g.circle(0, 0, radius)
          .fill({ color, alpha: 0.9 })
          .stroke({ color: 0xffffff, width: 1.5, alpha: 0.45 });
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
      let dragging = false;
      let lastX = 0;
      let lastY = 0;
      app.stage.on("pointerdown", (e) => {
        dragging = true;
        lastX = e.global.x;
        lastY = e.global.y;
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
        camera.x += e.global.x - lastX;
        camera.y += e.global.y - lastY;
        lastX = e.global.x;
        lastY = e.global.y;
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

  // Redraw the contents whenever a new world snapshot arrives. The camera keeps
  // its transform, so the player's pan/zoom is untouched.
  useEffect(() => {
    drawRef.current();
  }, [world]);

  return <div ref={hostRef} className="atlas-map" />;
}
