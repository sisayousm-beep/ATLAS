import { sampleWorld } from "./data/sampleWorld";
import { AtlasMap } from "./map/AtlasMap";

export default function App() {
  const totalPop = sampleWorld.regions.reduce((s, r) => s + r.population, 0);

  return (
    <div className="app">
      <header className="topbar">
        <h1>Project Atlas</h1>
        <span className="subtitle">Phase 2 · Production · Market · Prices</span>
        <span className="stat">World population {totalPop.toLocaleString()}</span>
      </header>

      <main className="stage-wrap">
        <AtlasMap world={sampleWorld} />
      </main>

      <aside className="legend">
        <h2 className="panel-title">Nations</h2>
        {sampleWorld.nations.map((n) => (
          <div key={n.id} className="legend-row">
            <span
              className="swatch"
              style={{ background: `#${n.color.toString(16).padStart(6, "0")}` }}
            />
            {n.name}
          </div>
        ))}

        <h2 className="panel-title">Market</h2>
        {sampleWorld.prices.map((p) => {
          const trend = p.price > p.base * 1.05 ? "up" : p.price < p.base * 0.95 ? "down" : "flat";
          const arrow = trend === "up" ? "▲" : trend === "down" ? "▼" : "—";
          return (
            <div key={p.good} className="price-row">
              <span className={`tier-dot tier-${p.tier}`} title={p.tier} />
              <span className="good-name">{p.good}</span>
              <span className="good-price">{p.price.toFixed(1)}</span>
              <span className={`trend trend-${trend}`}>{arrow}</span>
            </div>
          );
        })}
      </aside>
    </div>
  );
}
