import { sampleWorld } from "./data/sampleWorld";
import { AtlasMap } from "./map/AtlasMap";

const nationColor = (id: string) =>
  sampleWorld.nations.find((n) => n.id === id)?.color ?? 0x888888;

const nationName = (id: string) =>
  sampleWorld.nations.find((n) => n.id === id)?.name ?? id;

const swatch = (id: string) => `#${nationColor(id).toString(16).padStart(6, "0")}`;

export default function App() {
  const totalPop = sampleWorld.regions.reduce((s, r) => s + r.population, 0);

  return (
    <div className="app">
      <header className="topbar">
        <h1>Project Atlas</h1>
        <span className="subtitle">Phase 5 · Politics · Diplomacy</span>
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
            <span className="swatch" style={{ background: swatch(c.nationId) }} title={c.nationId} />
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

        <h2 className="panel-title">Finance · Central Bank</h2>
        {sampleWorld.crisis && <div className="crisis-banner">⚠ Financial crisis</div>}
        {sampleWorld.finance.map((f) => (
          <div key={f.nationId} className="corp-row">
            <span className="swatch" style={{ background: swatch(f.nationId) }} title={f.nationId} />
            <span className="corp-name">
              {nationName(f.nationId)}
              <span className="corp-industries">
                rate {(f.policyRate * 100).toFixed(1)}% · infl {(f.inflation * 100).toFixed(1)}%
              </span>
            </span>
            <span className={`corp-capital ${f.debt > 0 ? "loss" : "profit"}`}>
              {f.debt > 0 ? `−${f.debt.toLocaleString()}` : "—"}
            </span>
          </div>
        ))}

        <h2 className="panel-title">Politics</h2>
        {sampleWorld.politics.map((p) => (
          <div key={p.nationId} className="corp-row">
            <span className="swatch" style={{ background: swatch(p.nationId) }} title={p.nationId} />
            <span className="corp-name">
              {nationName(p.nationId)}
              <span className="corp-industries">{p.government}</span>
            </span>
            <span className="stability">
              <span className="stability-track">
                <span
                  className={`stability-fill ${p.unrest > 0.6 ? "low" : p.unrest > 0.4 ? "mid" : "high"}`}
                  style={{ width: `${Math.round(p.stability * 100)}%` }}
                />
              </span>
              {Math.round(p.stability * 100)}%
            </span>
          </div>
        ))}

        <h2 className="panel-title">Diplomacy</h2>
        {sampleWorld.relations.map((r) => (
          <div key={`${r.a}-${r.b}`} className="rel-row">
            <span className="rel-pair">
              <span className="swatch swatch-sm" style={{ background: swatch(r.a) }} title={r.a} />
              {nationName(r.a)} · {nationName(r.b)}
              <span className="swatch swatch-sm" style={{ background: swatch(r.b) }} title={r.b} />
            </span>
            <span className={`rel-status rel-${r.status}`}>{r.status}</span>
          </div>
        ))}
      </aside>
    </div>
  );
}
