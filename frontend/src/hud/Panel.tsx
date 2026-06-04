// Right-hand panel (UI roadmap Phase 4 — Strategic HUD).
//
// Renders the content for whichever left-menu tab is active. The sections are the
// ones the earlier phases already surfaced, regrouped under the four screens:
//   overview — nations ranked by GDP (design §17), plus crisis/war banners
//   economy  — market prices, central banks, corporations
//   trade    — per-nation trade balance
//   politics — stability, diplomacy, military

import type { Selection } from "../map/AtlasMap";
import type { WorldView } from "../types";
import { goodLabel, governmentLabel, relationLabel, tierLabel } from "../i18n/labels";
import { compact, signed } from "../i18n/format";
import type { HudTab } from "./LeftNav";

interface Props {
  world: WorldView;
  tab: HudTab;
  selection: Selection;
  onSelectNation: (id: string) => void;
}

const swatchOf = (color: number) => `#${color.toString(16).padStart(6, "0")}`;

export function Panel({ world, tab, selection, onSelectNation }: Props) {
  const nationColor = (id: string) => world.nations.find((n) => n.id === id)?.color ?? 0x888888;
  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;
  const swatch = (id: string) => swatchOf(nationColor(id));

  return (
    <aside className="panel">
      {tab === "overview" && (
        <Overview
          world={world}
          selection={selection}
          onSelectNation={onSelectNation}
          swatch={swatch}
        />
      )}

      {tab === "economy" && (
        <>
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
                {f.debt > 0 ? `−${compact(f.debt)}` : "—"}
              </span>
            </div>
          ))}

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
                {compact(Math.abs(c.capital))}
              </span>
            </div>
          ))}
        </>
      )}

      {tab === "trade" && (
        <>
          <h2 className="panel-title">무역수지</h2>
          {world.economy.map((e) => {
            const net = e.exports - e.imports;
            return (
              <div key={e.nationId} className="corp-row">
                <span className="swatch" style={{ background: swatch(e.nationId) }} title={e.nationId} />
                <span className="corp-name">
                  {nationName(e.nationId)}
                  <span className="corp-industries">
                    수출 {compact(e.exports)} · 수입 {compact(e.imports)}
                  </span>
                </span>
                <span className={`corp-capital ${net < 0 ? "loss" : "profit"}`}>{signed(net)}</span>
              </div>
            );
          })}
        </>
      )}

      {tab === "politics" && (
        <>
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
                <span className="corp-industries">병력 {compact(m.strength)}</span>
              </span>
              <span className={`mil-power ${m.atWar ? "at-war" : ""}`}>
                {m.atWar ? "⚔ " : ""}
                {compact(m.power)}
              </span>
            </div>
          ))}
        </>
      )}
    </aside>
  );
}

/** Overview: the GDP league table (design §17 economic victory) + alert banners. */
function Overview({
  world,
  selection,
  onSelectNation,
  swatch,
}: {
  world: WorldView;
  selection: Selection;
  onSelectNation: (id: string) => void;
  swatch: (id: string) => string;
}) {
  const ranked = [...world.economy].sort((a, b) => b.gdp - a.gdp);
  const top = ranked[0]?.gdp ?? 1;
  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;

  return (
    <>
      {world.crisis && <div className="crisis-banner">⚠ 금융 위기</div>}
      {world.wars.length > 0 ? (
        world.wars.map((w) => (
          <div key={`${w.aggressor}-${w.defender}`} className="war-banner">
            ⚔ {nationName(w.aggressor)} → {nationName(w.defender)}
          </div>
        ))
      ) : (
        <div className="war-peace">평화 상태</div>
      )}

      <h2 className="panel-title">GDP 순위</h2>
      {ranked.map((e, i) => (
        <button
          key={e.nationId}
          className={`gdp-row ${
            selection?.kind === "nation" && selection.id === e.nationId ? "active" : ""
          }`}
          onClick={() => onSelectNation(e.nationId)}
        >
          <span className="gdp-rank">{i + 1}</span>
          <span className="swatch" style={{ background: swatch(e.nationId) }} />
          <span className="gdp-name">{nationName(e.nationId)}</span>
          <span className="gdp-value">{compact(e.gdp)}</span>
          <span className="gdp-bar-track">
            <span className="gdp-bar-fill" style={{ width: `${Math.round((e.gdp / top) * 100)}%` }} />
          </span>
        </button>
      ))}
    </>
  );
}
