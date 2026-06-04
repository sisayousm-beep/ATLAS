import { useEffect, useState } from "react";
import { useGameStore } from "./store/gameStore";
import { AtlasMap, type Selection } from "./map/AtlasMap";
import { Inspector } from "./inspect/Inspector";
import { LeftNav, type HudTab } from "./hud/LeftNav";
import { Panel } from "./hud/Panel";
import { NotificationFeed } from "./hud/NotificationFeed";
import { NationDashboard, type DashTab } from "./dash/NationDashboard";
import { EconomicDashboard } from "./dash/EconomicDashboard";
import { MarketDashboard } from "./dash/MarketDashboard";
import { CompanyDashboard } from "./dash/CompanyDashboard";
import { TradeDashboard } from "./dash/TradeDashboard";
import { PopulationDashboard } from "./dash/PopulationDashboard";
import { MODE_CSS, MODE_LABEL, MODE_ORDER, nationCentroids, routeMode } from "./map/tradeRoutes";
import { compact } from "./i18n/format";

const SPEEDS = [1, 2, 5];

export default function App() {
  const world = useGameStore((s) => s.world);
  const status = useGameStore((s) => s.status);
  const live = useGameStore((s) => s.live);
  const history = useGameStore((s) => s.history);
  const macro = useGameStore((s) => s.macro);
  const priceHistory = useGameStore((s) => s.priceHistory);
  const init = useGameStore((s) => s.init);
  const pause = useGameStore((s) => s.pause);
  const resume = useGameStore((s) => s.resume);
  const setSpeed = useGameStore((s) => s.setSpeed);

  // Phase 3: what the player has selected on the map / legend.
  const [selection, setSelection] = useState<Selection>(null);
  // Phase 4: which left-menu screen the right panel is showing.
  const [tab, setTab] = useState<HudTab>("overview");
  // Phase 5: the nation whose dashboard is open (full-screen), and its sub-tab.
  const [dashNation, setDashNation] = useState<string | null>(null);
  const [dashTab, setDashTab] = useState<DashTab>("overview");
  // Phase 6: whether the world economic dashboard is open.
  const [econOpen, setEconOpen] = useState(false);
  // Phase 7: whether the market dashboard is open.
  const [marketOpen, setMarketOpen] = useState(false);
  // Phase 8: whether the company dashboard is open.
  const [companyOpen, setCompanyOpen] = useState(false);
  // Phase 9: whether the trade network dashboard is open.
  const [tradeOpen, setTradeOpen] = useState(false);
  // Phase 10: whether the population dashboard is open.
  const [popOpen, setPopOpen] = useState(false);
  // 유통.md Trade Overlay: whether trade routes are drawn on the world map. On by
  // default so the network is visible the moment the map loads.
  const [showTrade, setShowTrade] = useState(true);

  useEffect(() => {
    void init();
  }, [init]);

  if (!world) {
    return <div className="app loading">시뮬레이션에 연결하는 중…</div>;
  }

  // Top-bar KPIs focus on the selected nation (or the owner of the selected
  // region); with nothing selected they aggregate the whole world.
  const focusId =
    selection?.kind === "nation"
      ? selection.id
      : selection?.kind === "region"
        ? world.regions.find((r) => r.id === selection.id)?.nationId ?? null
        : null;

  const focusName = focusId
    ? world.nations.find((n) => n.id === focusId)?.name ?? focusId
    : "세계";
  const gdp = focusId
    ? world.economy.find((e) => e.nationId === focusId)?.gdp ?? 0
    : world.economy.reduce((s, e) => s + e.gdp, 0);
  const money = focusId
    ? world.economy.find((e) => e.nationId === focusId)?.treasury ?? 0
    : world.economy.reduce((s, e) => s + e.treasury, 0);
  const population = world.regions
    .filter((r) => !focusId || r.nationId === focusId)
    .reduce((s, r) => s + r.population, 0);

  // Transport modes present on the map this tick, for the overlay legend.
  const tradeCentroids = nationCentroids(world);
  const tradeModes = MODE_ORDER.filter((m) =>
    world.tradeRoutes.some((r) => routeMode(r, tradeCentroids) === m),
  );

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

        <button className="ctrl" onClick={() => setEconOpen(true)}>
          📊 경제
        </button>

        <button className="ctrl" onClick={() => setMarketOpen(true)}>
          📈 시장
        </button>

        <button className="ctrl" onClick={() => setCompanyOpen(true)}>
          🏭 기업
        </button>

        <button className="ctrl" onClick={() => setTradeOpen(true)}>
          🚢 무역
        </button>

        <button className="ctrl" onClick={() => setPopOpen(true)}>
          👥 인구
        </button>

        <button
          className={`ctrl ${showTrade ? "active" : ""}`}
          onClick={() => setShowTrade((v) => !v)}
          aria-pressed={showTrade}
        >
          🧭 교역로
        </button>

        <div className="kpis">
          <span className="kpi-focus">{focusName}</span>
          <Kpi label="GDP" value={compact(gdp)} />
          <Kpi label="국고" value={compact(money)} />
          <Kpi label="인구" value={compact(population)} />
        </div>
      </header>

      <LeftNav active={tab} onSelect={setTab} />

      <main className="stage-wrap">
        <AtlasMap
          world={world}
          selected={selection}
          showTrade={showTrade}
          onSelectRegion={(id) => setSelection({ kind: "region", id })}
          onSelectNation={(id) => setSelection({ kind: "nation", id })}
          onClear={() => setSelection(null)}
        />
        {showTrade && tradeModes.length > 0 && (
          <div className="map-legend" role="img" aria-label="교역로 운송수단 범례">
            <span className="map-legend-title">교역로 · 굵기 = 물동량</span>
            <div className="map-legend-items">
              {tradeModes.map((m) => (
                <span className="map-legend-item" key={m}>
                  <span className="map-legend-swatch" style={{ background: MODE_CSS[m] }} />
                  {MODE_LABEL[m]}
                </span>
              ))}
            </div>
          </div>
        )}
        <div className="map-hint">스크롤: 확대/축소 · 드래그: 이동 · 클릭: 선택</div>
        <NotificationFeed />
        <Inspector
          world={world}
          selection={selection}
          onSelectRegion={(id) => setSelection({ kind: "region", id })}
          onSelectNation={(id) => setSelection({ kind: "nation", id })}
          onOpenDashboard={(id) => setDashNation(id)}
          onClose={() => setSelection(null)}
        />
      </main>

      <Panel
        world={world}
        tab={tab}
        selection={selection}
        onSelectNation={(id) => setSelection({ kind: "nation", id })}
      />

      {dashNation && (
        <NationDashboard
          world={world}
          history={history[dashNation] ?? []}
          nationId={dashNation}
          tab={dashTab}
          onTab={setDashTab}
          onSelectNation={setDashNation}
          onClose={() => setDashNation(null)}
        />
      )}

      {econOpen && (
        <EconomicDashboard world={world} macro={macro} onClose={() => setEconOpen(false)} />
      )}

      {marketOpen && (
        <MarketDashboard
          world={world}
          priceHistory={priceHistory}
          onClose={() => setMarketOpen(false)}
        />
      )}

      {companyOpen && (
        <CompanyDashboard world={world} onClose={() => setCompanyOpen(false)} />
      )}

      {tradeOpen && <TradeDashboard world={world} onClose={() => setTradeOpen(false)} />}

      {popOpen && <PopulationDashboard world={world} onClose={() => setPopOpen(false)} />}
    </div>
  );
}

/** A single top-bar metric: a small label over a big value. */
function Kpi({ label, value }: { label: string; value: string }) {
  return (
    <span className="kpi">
      <span className="kpi-label">{label}</span>
      <span className="kpi-value">{value}</span>
    </span>
  );
}
