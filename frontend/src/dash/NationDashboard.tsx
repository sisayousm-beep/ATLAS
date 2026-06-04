// Nation Dashboard (UI roadmap Phase 5 — 국가 운영 화면).
//
// A full-screen overlay for running one nation. Five screens — overview, economy,
// population, politics, technology — read off the current world snapshot, and the
// headline figures (GDP, debt, inflation, happiness) plot as time-series charts
// from the per-nation monthly history the store accumulates.

import { useEffect } from "react";
import type { NationSample } from "../store/gameStore";
import type { WorldView } from "../types";
import { goodLabel, governmentLabel, relationLabel } from "../i18n/labels";
import { compact, int, signed } from "../i18n/format";
import { LineChart } from "./LineChart";

export type DashTab = "overview" | "economy" | "population" | "politics" | "technology";

const TABS: { id: DashTab; label: string }[] = [
  { id: "overview", label: "개요" },
  { id: "economy", label: "경제" },
  { id: "population", label: "인구" },
  { id: "politics", label: "정치" },
  { id: "technology", label: "기술" },
];

interface Props {
  world: WorldView;
  history: NationSample[];
  nationId: string;
  tab: DashTab;
  onTab: (t: DashTab) => void;
  /** Switch the dashboard to another nation (e.g. from a relation row). */
  onSelectNation: (id: string) => void;
  onClose: () => void;
}

const swatchOf = (color: number) => `#${color.toString(16).padStart(6, "0")}`;
const pct = (v: number) => `${Math.round(v * 100)}%`;

export function NationDashboard({ world, history, nationId, tab, onTab, onSelectNation, onClose }: Props) {
  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const nation = world.nations.find((n) => n.id === nationId);
  if (!nation) return null;

  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;
  const regions = world.regions.filter((r) => r.nationId === nationId);
  const population = regions.reduce((s, r) => s + r.population, 0);
  const econ = world.economy.find((e) => e.nationId === nationId);
  const fin = world.finance.find((f) => f.nationId === nationId);
  const pol = world.politics.find((p) => p.nationId === nationId);
  const mil = world.military.find((m) => m.nationId === nationId);
  const tech = world.technology.find((t) => t.nationId === nationId);
  const corps = world.corporations.filter((c) => c.nationId === nationId);

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label={`${nation.name} 대시보드`}>
        <header className="dash-head">
          <span className="swatch" style={{ background: swatchOf(nation.color) }} />
          <h2>{nation.name}</h2>
          {pol && <span className="dash-gov">{governmentLabel(pol.government)}</span>}
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <nav className="dash-tabs">
          {TABS.map((t) => (
            <button
              key={t.id}
              className={`dash-tab ${tab === t.id ? "active" : ""}`}
              onClick={() => onTab(t.id)}
            >
              {t.label}
            </button>
          ))}
        </nav>

        <div className="dash-body">
          {tab === "overview" && (
            <>
              <div className="dash-kpis">
                <Kpi label="인구" value={compact(population)} />
                <Kpi label="GDP" value={compact(econ?.gdp ?? 0)} />
                <Kpi label="국고" value={compact(econ?.treasury ?? 0)} />
                <Kpi label="군사력" value={compact(mil?.power ?? 0)} />
                <Kpi label="안정도" value={pct(pol?.stability ?? 0)} />
                <Kpi label="행복도" value={pct(pol?.happiness ?? 0)} />
                <Kpi label="기술" value={(tech?.level ?? 0).toFixed(2)} />
                <Kpi label="물가" value={`${((fin?.inflation ?? 0) * 100).toFixed(1)}%`} />
              </div>
              <ChartCard
                title="GDP"
                value={compact(econ?.gdp ?? 0)}
                data={history.map((h) => h.gdp)}
              />
            </>
          )}

          {tab === "economy" && (
            <>
              <dl className="dash-stats">
                <Stat label="국고" value={compact(econ?.treasury ?? 0)} />
                <Stat label="GDP" value={compact(econ?.gdp ?? 0)} />
                <Stat label="수출" value={compact(econ?.exports ?? 0)} />
                <Stat label="수입" value={compact(econ?.imports ?? 0)} />
                <Stat
                  label="무역수지"
                  value={signed((econ?.exports ?? 0) - (econ?.imports ?? 0))}
                  neg={(econ?.exports ?? 0) - (econ?.imports ?? 0) < 0}
                />
                <Stat label="정책금리" value={`${((fin?.policyRate ?? 0) * 100).toFixed(1)}%`} />
              </dl>

              <div className="chart-grid">
                <ChartCard title="GDP" value={compact(econ?.gdp ?? 0)} data={history.map((h) => h.gdp)} />
                <ChartCard
                  title="부채"
                  value={compact(fin?.debt ?? 0)}
                  data={history.map((h) => h.debt)}
                  color="#e0686d"
                />
                <ChartCard
                  title="물가상승"
                  value={`${((fin?.inflation ?? 0) * 100).toFixed(1)}%`}
                  data={history.map((h) => h.inflation)}
                  color="#c79a4a"
                />
              </div>

              <h3 className="dash-sub">기업 ({corps.length})</h3>
              {corps.map((c) => (
                <div key={c.name} className="corp-row">
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

          {tab === "population" && (
            <>
              <dl className="dash-stats">
                <Stat label="총인구" value={int(population)} />
                <Stat label="지역" value={`${regions.length}`} />
                <Stat label="행복도" value={pct(pol?.happiness ?? 0)} />
              </dl>

              <ChartCard
                title="행복도"
                value={pct(pol?.happiness ?? 0)}
                data={history.map((h) => h.happiness)}
                color="#5b7fb5"
              />

              <h3 className="dash-sub">지역 ({regions.length})</h3>
              {regions.map((r) => (
                <div key={r.id} className="corp-row">
                  <span className="corp-name">
                    {r.name}
                    <span className="corp-industries">GDP {compact(r.gdp)}</span>
                  </span>
                  <span className="corp-capital">{int(r.population)}</span>
                </div>
              ))}
            </>
          )}

          {tab === "politics" && (
            <>
              <dl className="dash-stats">
                <Stat label="정부" value={pol ? governmentLabel(pol.government) : "—"} />
                <Bar label="안정도" value={pol?.stability ?? 0} />
                <Bar label="불안" value={pol?.unrest ?? 0} danger />
                <Bar label="행복도" value={pol?.happiness ?? 0} />
              </dl>

              <h3 className="dash-sub">외교</h3>
              {world.relations
                .filter((r) => r.a === nationId || r.b === nationId)
                .map((r) => {
                  const other = r.a === nationId ? r.b : r.a;
                  return (
                    <button
                      key={`${r.a}-${r.b}`}
                      className="dash-rel"
                      onClick={() => onSelectNation(other)}
                    >
                      <span>{nationName(other)}</span>
                      <span className={`rel-status rel-${r.status}`}>{relationLabel(r.status)}</span>
                    </button>
                  );
                })}
            </>
          )}

          {tab === "technology" && (
            <>
              <dl className="dash-stats">
                <Stat label="기술 수준" value={(tech?.level ?? 0).toFixed(2)} />
                <Stat label="군사력" value={compact(mil?.power ?? 0)} />
              </dl>

              <h3 className="dash-sub">기술 경쟁</h3>
              <TechRace world={world} nationId={nationId} onSelectNation={onSelectNation} />
              <p className="dash-note">기술은 군사력을 끌어올린다 (설계 §12·§17).</p>
            </>
          )}
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

function Stat({ label, value, neg }: { label: string; value: string; neg?: boolean }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd className={neg ? "neg" : ""}>{value}</dd>
    </div>
  );
}

/** A labelled 0..1 bar, reusing the panel's stability colours. */
function Bar({ label, value, danger }: { label: string; value: number; danger?: boolean }) {
  const tone = danger
    ? value > 0.6
      ? "low"
      : value > 0.4
        ? "mid"
        : "high"
    : value > 0.6
      ? "high"
      : value > 0.4
        ? "mid"
        : "low";
  return (
    <div>
      <dt>{label}</dt>
      <dd>
        <span className="stability">
          <span className="stability-track">
            <span className={`stability-fill ${tone}`} style={{ width: pct(value) }} />
          </span>
          {pct(value)}
        </span>
      </dd>
    </div>
  );
}

function ChartCard({
  title,
  value,
  data,
  color,
}: {
  title: string;
  value: string;
  data: number[];
  color?: string;
}) {
  return (
    <div className="chart-card">
      <div className="chart-head">
        <span className="chart-title">{title}</span>
        <span className="chart-value">{value}</span>
      </div>
      <LineChart data={data} color={color} />
    </div>
  );
}

/** Technology league across all nations (the tech race, design §17). */
function TechRace({
  world,
  nationId,
  onSelectNation,
}: {
  world: WorldView;
  nationId: string;
  onSelectNation: (id: string) => void;
}) {
  const ranked = [...world.technology].sort((a, b) => b.level - a.level);
  const top = ranked[0]?.level ?? 1;
  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;
  const swatch = (id: string) => swatchOf(world.nations.find((n) => n.id === id)?.color ?? 0x888888);

  return (
    <>
      {ranked.map((t) => (
        <button
          key={t.nationId}
          className={`gdp-row ${t.nationId === nationId ? "active" : ""}`}
          onClick={() => onSelectNation(t.nationId)}
        >
          <span className="gdp-rank" />
          <span className="swatch" style={{ background: swatch(t.nationId) }} />
          <span className="gdp-name">{nationName(t.nationId)}</span>
          <span className="gdp-value">{t.level.toFixed(2)}</span>
          <span className="gdp-bar-track">
            <span className="gdp-bar-fill" style={{ width: pct(t.level / top) }} />
          </span>
        </button>
      ))}
    </>
  );
}
