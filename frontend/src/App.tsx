import { sampleWorld } from "./data/sampleWorld";
import { AtlasMap } from "./map/AtlasMap";

export default function App() {
  const totalPop = sampleWorld.regions.reduce((s, r) => s + r.population, 0);

  return (
    <div className="app">
      <header className="topbar">
        <h1>Project Atlas</h1>
        <span className="subtitle">Phase 1 · Regions · Nations · Pops</span>
        <span className="stat">World population {totalPop.toLocaleString()}</span>
      </header>

      <main className="stage-wrap">
        <AtlasMap world={sampleWorld} />
      </main>

      <aside className="legend">
        {sampleWorld.nations.map((n) => (
          <div key={n.id} className="legend-row">
            <span
              className="swatch"
              style={{ background: `#${n.color.toString(16).padStart(6, "0")}` }}
            />
            {n.name}
          </div>
        ))}
      </aside>
    </div>
  );
}
