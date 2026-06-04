// Economic Dashboard (UI roadmap Phase 6 — 경제를 게임의 주인공으로).
//
// A full-screen overlay for reading the world economy at a glance. Five macro
// figures — GDP, employment, inflation, money supply, interest rate — show their
// current value off the live snapshot and plot as time-series from the macro
// history the store accumulates. A range selector scopes every chart to the last
// 1 / 5 / 10 / 50 years.

import { useEffect, useState } from "react";
import type { MacroSample } from "../store/gameStore";
import type { WorldView } from "../types";
import { compact } from "../i18n/format";
import { LineChart } from "./LineChart";

interface Props {
  world: WorldView;
  macro: MacroSample[];
  onClose: () => void;
}

const RANGES: { id: string; label: string; months: number }[] = [
  { id: "1y", label: "1년", months: 12 },
  { id: "5y", label: "5년", months: 60 },
  { id: "10y", label: "10년", months: 120 },
  { id: "50y", label: "50년", months: 600 },
];

const mean = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0);
const pct1 = (v: number) => `${(v * 100).toFixed(1)}%`;

export function EconomicDashboard({ world, macro, onClose }: Props) {
  const [range, setRange] = useState(RANGES[2]); // default 10년

  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  // Current values come straight off the live world so the KPIs are exact even
  // before the first month tick lands in history.
  const gdp = world.economy.reduce((s, e) => s + e.gdp, 0);
  const employment = world.corporations.reduce((s, c) => s + c.employees, 0);
  const inflation = mean(world.finance.map((f) => f.inflation));
  const moneySupply = world.finance.reduce((s, f) => s + f.moneySupply, 0);
  const interestRate = mean(world.finance.map((f) => f.policyRate));
  // Unemployment is the world's labour-force-weighted 실업률 (design §8).
  const laborForce = world.labor.reduce((s, l) => s + l.laborForce, 0);
  const unemployment = laborForce > 0
    ? world.labor.reduce((s, l) => s + l.unemployed, 0) / laborForce
    : 0;

  const slice = (pick: (m: MacroSample) => number) => macro.slice(-range.months).map(pick);

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label="세계 경제 대시보드">
        <header className="dash-head">
          <h2>세계 경제</h2>
          <span className="dash-gov">실물 · 금융 지표</span>
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <div className="dash-body">
          <div className="dash-kpis macro">
            <Kpi label="GDP" value={compact(gdp)} />
            <Kpi label="고용" value={compact(employment)} />
            <Kpi label="실업률" value={pct1(unemployment)} />
            <Kpi label="물가" value={pct1(inflation)} />
            <Kpi label="통화량" value={compact(moneySupply)} />
            <Kpi label="금리" value={pct1(interestRate)} />
          </div>

          <div className="dash-range">
            {RANGES.map((r) => (
              <button
                key={r.id}
                className={`range-btn ${r.id === range.id ? "active" : ""}`}
                onClick={() => setRange(r)}
              >
                {r.label}
              </button>
            ))}
          </div>

          <div className="chart-grid">
            <ChartCard title="GDP" value={compact(gdp)} data={slice((m) => m.gdp)} />
            <ChartCard
              title="고용"
              value={compact(employment)}
              data={slice((m) => m.employment)}
              color="#5b7fb5"
            />
            <ChartCard
              title="실업률"
              value={pct1(unemployment)}
              data={slice((m) => m.unemployment)}
              color="#d98c5b"
            />
            <ChartCard
              title="물가"
              value={pct1(inflation)}
              data={slice((m) => m.inflation)}
              color="#c79a4a"
            />
            <ChartCard
              title="통화량"
              value={compact(moneySupply)}
              data={slice((m) => m.moneySupply)}
              color="#7fa6d9"
            />
            <ChartCard
              title="금리"
              value={pct1(interestRate)}
              data={slice((m) => m.interestRate)}
              color="#e0686d"
            />
          </div>
          <p className="dash-note">
            고용은 기업 고용 인원 합계 · 실업률은 노동력 가중 평균 · 물가/금리는 국가 평균.
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
