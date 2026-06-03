import { useEffect, useRef } from "react";
import { Application, Container, Graphics, Text } from "pixi.js";
import type { WorldView } from "../types";

interface Props {
  world: WorldView;
}

/** Renders the world as nation-coloured region nodes on a PixiJS canvas. */
export function AtlasMap({ world }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    let app: Application | null = null;
    let disposed = false;
    const nationColor = new Map(world.nations.map((n) => [n.id, n.color]));

    const draw = () => {
      if (!app) return;
      const { width, height } = app.screen;
      app.stage.removeChildren();

      for (const region of world.regions) {
        const color = nationColor.get(region.nationId) ?? 0x999999;
        const radius = 14 + Math.sqrt(region.population) / 30;

        const node = new Container();
        node.x = region.x * width;
        node.y = region.y * height;

        const g = new Graphics();
        g.circle(0, 0, radius)
          .fill({ color, alpha: 0.85 })
          .stroke({ color: 0xffffff, width: 1.5, alpha: 0.4 });
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

        app.stage.addChild(node);
      }
    };

    (async () => {
      const a = new Application();
      await a.init({ background: "#0b1021", antialias: true, resizeTo: host });
      if (disposed) {
        a.destroy(true);
        return;
      }
      app = a;
      host.appendChild(a.canvas);
      draw();
    })();

    const onResize = () => draw();
    window.addEventListener("resize", onResize);

    return () => {
      disposed = true;
      window.removeEventListener("resize", onResize);
      if (app) {
        app.destroy(true, { children: true });
        app = null;
      }
    };
  }, [world]);

  return <div ref={hostRef} className="atlas-map" />;
}
