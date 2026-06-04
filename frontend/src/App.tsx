import { useEffect, useState } from "react";
import { useGameStore } from "./store/gameStore";
import { AtlasMap, type Selection } from "./map/AtlasMap";
import { Inspector } from "./inspect/Inspector";
import { goodLabel, governmentLabel, relationLabel, tierLabel } from "./i18n/labels";

const SPEEDS = [1, 2, 5];

export default function App() {
  const world = useGameStore((s) => s.world);
  const status = useGameStore((s) => s.status);
  const live = useGameStore((s) => s.live);
  const init = useGameStore((s) => s.init);
  const pause = useGameStore((s) => s.pause);
  const resume = useGameStore((s) => s.resume);
  const setSpeed = useGameStore((s) => s.setSpeed);

  // Phase 3: what the player has selected on the map / legend.
  const [selection, setSelection] = useState<Selection>(null);

  useEffect(() => {
    void init();
  }, [init]);

  if (!world) {
    return <div className="app loading">시뮬레이션에 연결하는 중…</div>;
  }

  const nationColor = (id: string) => world.nations.find((n) => n.id === id)?.color ?? 0x888888;
  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;
  const swatch = (id: string) => `#${nationColor(id).toString(16).padStart(6, "0")}`;

  const totalPop = world.regions.reduce((s, r) => s + r.population, 0);

  return (
    <div className="app">
      <header className="topbar">
        <h1>프로젝트 아틀라스</h1>
        <span className="clock">
          {world.clock.year}년 {world.clock.month}월 {world.clock.dayOfMonth}일
        </span>

        <div className="loop-controls">
          <button className="ctrl" onClick={status.paused ? resume : pause}>
            {status.paused ? "▶ 재개" : "⏸ 일시정지"}
          </button>
          {SPEEDS.map((s) => (
            <button
              key={s}
              className={`speed ${status.speed === s ? "active" : ""}`}
              onClick={() => setSpeed(s)}
            >
              {s}×
            </button>
          ))}
        </div>

        <span className={`mode ${live ? "live" : "mock"}`}>{live ? "● 엔진" : "○ 데모"}</span>
        <span className="stat">세계 인구 {totalPop.toLocaleString()}</span>
      </header>

      <main className="stage-wrap">
        <AtlasMap
          world={world}
          selected={selection}
          onSelectRegion={(id) => setSelection({ kind: "region", id })}
          onSelectNation={(id) => setSelection({ kind: "nation", id })}
          onClear={() => setSelection(null)}
        />
        <div className="map-hint">스크롤: 확대/축소 · 드래그: 이동 · 클릭: 선택</div>
        <Inspector
          world={world}
          selection={selection}
          onSelectRegion={(id) => setSelection({ kind: "region", id })}
          onSelectNation={(id) => setSelection({ kind: "nation", id })}
          onClose={() => setSelection(null)}
        />
      </main>

      <aside className="legend">
        <h2 className="panel-title">국가</h2>
        {world.nations.map((n) => (
          <button
            key={n.id}
            className={`legend-row legend-nation ${
              selection?.kind === "nation" && selection.id === n.id ? "active" : ""
            }`}
            onClick={() => setSelection({ kind: "nation", id: n.id })}
          >
            <span
              className="swatch"
              style={{ background: `#${n.color.toString(16).padStart(6, "0")}` }}
            />
            {n.name}
          </button>
        ))}

        <h2 className="panel-title">시장</h2>
        {world.prices.map((p) => {
          const trend = p.price > p.base * 1.05 ? "up" : p.price < p.base * 0.95 ? "down" : "flat";
          const arrow = trend === "up" ? "▲" : trend === "down" ? "▼" : "—";
          return (
            <div key={p.good} className="price-row">
              <span className={`tier-dot tier-${p.tier}`} title={tierLabel(p.tier)} />
              <span className="good-name">{goodLabel(p.good)}</span>
              <span className="good-price">{p.price.toFixed(1)}</span>
              <span className={`trend trend-${trend}`}>{arrow}</span>
            </div>
          );
        })}

        <h2 className="panel-title">기업</h2>
        {world.corporations.map((c) => (
          <div key={c.name} className="corp-row">
            <span className="swatch" style={{ background: swatch(c.nationId) }} title={c.nationId} />
            <span className="corp-name">
              {c.name}
              <span className="corp-industries">{c.industries.map(goodLabel).join(", ")}</span>
            </span>
            <span className={`corp-capital ${c.capital < 0 ? "loss" : "profit"}`}>
              {c.capital < 0 ? "−" : ""}
              {Math.abs(c.capital).toLocaleString(undefined, { maximumFractionDigits: 0 })}
            </span>
          </div>
        ))}

        <h2 className="panel-title">금융 · 중앙은행</h2>
        {world.crisis && <div className="crisis-banner">⚠ 금융 위기</div>}
        {world.finance.map((f) => (
          <div key={f.nationId} className="corp-row">
            <span className="swatch" style={{ background: swatch(f.nationId) }} title={f.nationId} />
            <span className="corp-name">
              {nationName(f.nationId)}
              <span className="corp-industries">
                금리 {(f.policyRate * 100).toFixed(1)}% · 물가 {(f.inflation * 100).toFixed(1)}%
              </span>
            </span>
            <span className={`corp-capital ${f.debt > 0 ? "loss" : "profit"}`}>
              {f.debt > 0 ? `−${f.debt.toLocaleString(undefined, { maximumFractionDigits: 0 })}` : "—"}
            </span>
          </div>
        ))}

        <h2 className="panel-title">정치</h2>
        {world.politics.map((p) => (
          <div key={p.nationId} className="corp-row">
            <span className="swatch" style={{ background: swatch(p.nationId) }} title={p.nationId} />
            <span className="corp-name">
              {nationName(p.nationId)}
              <span className="corp-industries">{governmentLabel(p.government)}</span>
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

        <h2 className="panel-title">외교</h2>
        {world.relations.map((r) => (
          <div key={`${r.a}-${r.b}`} className="rel-row">
            <span className="rel-pair">
              <span className="swatch swatch-sm" style={{ background: swatch(r.a) }} title={r.a} />
              {nationName(r.a)} · {nationName(r.b)}
              <span className="swatch swatch-sm" style={{ background: swatch(r.b) }} title={r.b} />
            </span>
            <span className={`rel-status rel-${r.status}`}>{relationLabel(r.status)}</span>
          </div>
        ))}

        <h2 className="panel-title">군사 · 전쟁</h2>
        {world.wars.length > 0 ? (
          world.wars.map((w) => (
            <div key={`${w.aggressor}-${w.defender}`} className="war-banner">
              ⚔ {nationName(w.aggressor)} → {nationName(w.defender)}
            </div>
          ))
        ) : (
          <div className="war-peace">평화 상태</div>
        )}
        {world.military.map((m) => (
          <div key={m.nationId} className="corp-row">
            <span className="swatch" style={{ background: swatch(m.nationId) }} title={m.nationId} />
            <span className="corp-name">
              {nationName(m.nationId)}
              <span className="corp-industries">
                병력 {m.strength.toLocaleString(undefined, { maximumFractionDigits: 0 })}
              </span>
            </span>
            <span className={`mil-power ${m.atWar ? "at-war" : ""}`}>
              {m.atWar ? "⚔ " : ""}
              {m.power.toLocaleString(undefined, { maximumFractionDigits: 0 })}
            </span>
          </div>
        ))}
      </aside>
    </div>
  );
}
