// Trade Network Visualization (UI roadmap Phase 9 — 물류와 무역을 시각화).
//
// A full-screen overlay over the cross-border economy: each nation's cumulative
// exports / imports / balance, plus the live trade-route network the logistics
// layer actually clears — nation pairs wired by this month's commerce, drawn on a
// schematic map laid out from the regions' map positions. Line weight tracks the
// trade value, so the player can read where the world's goods flow (완료 조건:
// 세계 경제 흐름 확인 가능).
//
// Transport modes (해운 / 철도 / 도로 / 항공) are not separately modelled by the
// engine — the logistics layer ships on a single infrastructure factor — so each
// lane's tier is a presentation-layer attribute (유통.md): explicit in the sample
// world, inferred from nation separation for live data. The same classification
// drives the colours here and the trade overlay on the world map, so the two read
// alike.

import { useEffect } from "react";
import type { WorldView } from "../types";
import { MODE_CSS, MODE_LABEL, MODE_ORDER, nationCentroids, routeMode } from "../map/tradeRoutes";
import { compact, signed } from "../i18n/format";

interface Props {
  world: WorldView;
  onClose: () => void;
}

const colorHex = (c: number) => `#${c.toString(16).padStart(6, "0")}`;

export function TradeDashboard({ world, onClose }: Props) {
  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;
  const nationColor = (id: string) => world.nations.find((n) => n.id === id)?.color ?? 0x808080;

  const totalExports = world.economy.reduce((s, e) => s + e.exports, 0);
  const totalImports = world.economy.reduce((s, e) => s + e.imports, 0);
  const routes = world.tradeRoutes;
  const monthVolume = routes.reduce((s, r) => s + r.value, 0);
  const maxRoute = routes.reduce((m, r) => Math.max(m, r.value), 0);

  // Transport tier per lane, shared with the map overlay so colours match.
  const centroids = nationCentroids(world);
  const usedModes = MODE_ORDER.filter((m) => routes.some((r) => routeMode(r, centroids) === m));

  // Per-nation trade balance, busiest traders first.
  const rows = [...world.economy].sort(
    (a, b) => b.exports + b.imports - (a.exports + a.imports),
  );

  // Schematic map: place each nation at the centroid of its regions (the same
  // 0..1 layout the Pixi map uses), then normalise into the SVG box so the lanes
  // spread out regardless of how the world is laid out.
  const raw = world.nations
    .map((n) => {
      const regs = world.regions.filter((r) => r.nationId === n.id);
      if (regs.length === 0) return null;
      const x = regs.reduce((s, r) => s + r.x, 0) / regs.length;
      const y = regs.reduce((s, r) => s + r.y, 0) / regs.length;
      return { id: n.id, x, y };
    })
    .filter((p): p is { id: string; x: number; y: number } => p !== null);

  const xs = raw.map((p) => p.x);
  const ys = raw.map((p) => p.y);
  const spread = (v: number, lo: number, hi: number) =>
    hi - lo < 1e-6 ? 50 : 12 + ((v - lo) / (hi - lo)) * 76;
  const pos = new Map(
    raw.map((p) => [
      p.id,
      { x: spread(p.x, Math.min(...xs), Math.max(...xs)), y: spread(p.y, Math.min(...ys), Math.max(...ys)) },
    ]),
  );

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label="무역">
        <header className="dash-head">
          <h2>무역망</h2>
          <span className="dash-gov">수출 · 수입 · 교역로</span>
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <div className="dash-body">
          <div className="dash-kpis">
            <Kpi label="총수출" value={compact(totalExports)} />
            <Kpi label="총수입" value={compact(totalImports)} />
            <Kpi label="교역량(이번 달)" value={compact(monthVolume)} />
            <Kpi label="교역로" value={String(routes.length)} />
          </div>

          <div className="dash-sub">교역 네트워크</div>
          <div className="trade-net">
            <svg viewBox="0 0 100 100" preserveAspectRatio="xMidYMid meet" role="img" aria-label="교역 네트워크">
              {routes.map((r) => {
                const a = pos.get(r.a);
                const b = pos.get(r.b);
                if (!a || !b) return null;
                const w = 0.5 + (maxRoute > 0 ? r.value / maxRoute : 0) * 4;
                return (
                  <line
                    key={`${r.a}-${r.b}`}
                    x1={a.x}
                    y1={a.y}
                    x2={b.x}
                    y2={b.y}
                    stroke={MODE_CSS[routeMode(r, centroids)]}
                    strokeOpacity={0.7}
                    strokeWidth={w}
                    strokeLinecap="round"
                  />
                );
              })}
              {raw.map((p) => {
                const c = pos.get(p.id)!;
                return (
                  <g key={p.id}>
                    <circle cx={c.x} cy={c.y} r={3.4} fill={colorHex(nationColor(p.id))} stroke="#0b1021" strokeWidth={0.8} />
                    <text x={c.x} y={c.y - 5} textAnchor="middle" className="trade-node-label">
                      {nationName(p.id)}
                    </text>
                  </g>
                );
              })}
            </svg>
          </div>

          {usedModes.length > 0 && (
            <div className="trade-legend">
              {usedModes.map((m) => (
                <span className="trade-legend-item" key={m}>
                  <span className="trade-legend-swatch" style={{ background: MODE_CSS[m] }} />
                  {MODE_LABEL[m]}
                </span>
              ))}
            </div>
          )}

          <div className="dash-sub">국가별 무역 수지</div>
          <div className="trade-table">
            <div className="trade-head">
              <span>국가</span>
              <span className="trade-num">수출</span>
              <span className="trade-num">수입</span>
              <span className="trade-num">수지</span>
            </div>
            {rows.map((e) => {
              const balance = e.exports - e.imports;
              return (
                <div key={e.nationId} className="trade-row">
                  <span className="trade-nation">
                    <span className="trade-dot" style={{ background: colorHex(nationColor(e.nationId)) }} />
                    {nationName(e.nationId)}
                  </span>
                  <span className="trade-num">{compact(e.exports)}</span>
                  <span className="trade-num">{compact(e.imports)}</span>
                  <span className={`trade-num ${balance < 0 ? "neg" : "pos"}`}>{signed(balance)}</span>
                </div>
              );
            })}
          </div>

          {routes.length > 0 && (
            <>
              <div className="dash-sub">교역로 (이번 달)</div>
              <div className="route-list">
                {routes.map((r) => {
                  const mode = routeMode(r, centroids);
                  return (
                    <div key={`${r.a}-${r.b}`} className="route-row">
                      <span className="route-pair">
                        {nationName(r.a)} <span className="route-link">↔</span> {nationName(r.b)}
                        <span className="route-mode" style={{ color: MODE_CSS[mode] }}>
                          {MODE_LABEL[mode]}
                        </span>
                      </span>
                      <span className="route-bar">
                        <span
                          className="route-bar-fill"
                          style={{
                            width: `${maxRoute > 0 ? (r.value / maxRoute) * 100 : 0}%`,
                            background: MODE_CSS[mode],
                          }}
                        />
                      </span>
                      <span className="trade-num">{compact(r.value)}</span>
                    </div>
                  );
                })}
              </div>
            </>
          )}

          <p className="dash-note">
            수출·수입은 게임 시작 이후 누적 · 교역로는 이번 달 국경 간 교역액(방향 무관) ·
            색상은 수송 모드(해운/철도/도로/항공), 지도 오버레이와 동일 · 엔진은 단일 인프라
            계수로 운송하므로 모드는 표시용 분류(국가 간 거리 기준 추정).
          </p>
        </div>
      </div>
    </div>
  );
}

function Kpi({ label, value }: { label: string; value: string }) {
  return (
    <div className="dash-kpi">
      <span className="dash-kpi-label">{label}</span>
      <span className="dash-kpi-value">{value}</span>
    </div>
  );
}
