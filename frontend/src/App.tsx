import { sampleWorld } from "./data/sampleWorld";
import { AtlasMap } from "./map/AtlasMap";

const nationColor = (id: string) =>
  sampleWorld.nations.find((n) => n.id === id)?.color ?? 0x888888;

export default function App() {
  const totalPop = sampleWorld.regions.reduce((s, r) => s + r.population, 0);

  return (
    <div className="app">
      <header className="topbar">
        <h1>Project Atlas</h1>
        <span className="subtitle">Phase 3 · Corporations · Trade · Market</span>
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

        <h2 className="panel-title">Corporations</h2>
        {sampleWorld.corporations.map((c) => (
          <div key={c.name} className="corp-row">
            <span
              className="swatch"
              style={{ background: `#${nationColor(c.nationId).toString(16).padStart(6, "0")}` }}
              title={c.nationId}
            />
            <span className="corp-name">
              {c.name}
              <span className="corp-industries">{c.industries.join(", ")}</span>
            </span>
            <span className={`corp-capital ${c.capital < 0 ? "loss" : "profit"}`}>
              {c.capital < 0 ? "−" : ""}
              {Math.abs(c.capital).toLocaleString()}
            </span>
          </div>
        ))}
      </aside>
    </div>
  );
}
