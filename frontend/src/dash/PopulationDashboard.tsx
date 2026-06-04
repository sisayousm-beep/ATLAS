// Population Dashboard (UI roadmap Phase 10 — POP 시스템 시각화).
//
// A full-screen overlay over the world's people: every population class with its
// head-count, share, income (average wealth), education (literacy) and happiness —
// the indicators the engine already carries on each pop — plus world employment
// read off the labour market. So the player can analyse who the population is and
// how it lives (완료 조건: 인구 변화 분석 가능).

import { useEffect } from "react";
import type { WorldView } from "../types";
import { professionLabel } from "../i18n/labels";
import { compact, int } from "../i18n/format";

interface Props {
  world: WorldView;
  onClose: () => void;
}

const pct = (v: number) => `${Math.round(v * 100)}%`;

export function PopulationDashboard({ world, onClose }: Props) {
  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const classes = world.population;
  const totalPop = classes.reduce((s, c) => s + c.size, 0);
  const maxSize = classes.reduce((m, c) => Math.max(m, c.size), 0);

  // World employment, labour-force-weighted (실업률/고용률, design §8).
  const force = world.labor.reduce((s, l) => s + l.laborForce, 0);
  const employed = world.labor.reduce((s, l) => s + l.employed, 0);
  const unemployed = world.labor.reduce((s, l) => s + l.unemployed, 0);
  const employRate = force > 0 ? employed / force : 0;
  const unemployRate = force > 0 ? unemployed / force : 0;

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label="인구">
        <header className="dash-head">
          <h2>인구</h2>
          <span className="dash-gov">계층 · 소득 · 교육 · 행복</span>
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <div className="dash-body">
          <div className="dash-kpis">
            <Kpi label="총인구" value={compact(totalPop)} />
            <Kpi label="계층" value={String(classes.length)} />
            <Kpi label="고용률" value={pct(employRate)} />
            <Kpi label="실업률" value={pct(unemployRate)} />
          </div>

          <div className="dash-sub">계층별 구성</div>
          <div className="pop-table">
            <div className="pop-head">
              <span>계층</span>
              <span className="pop-share-col">비중</span>
              <span className="pop-num">인구</span>
              <span className="pop-num">소득</span>
              <span className="pop-num">교육</span>
              <span className="pop-num">행복</span>
            </div>
            {classes.map((c) => {
              const share = totalPop > 0 ? c.size / totalPop : 0;
              return (
                <div key={c.profession} className="pop-row">
                  <span className="pop-class">{professionLabel(c.profession)}</span>
                  <span className="pop-share">
                    <span className="pop-bar">
                      <span
                        className="pop-bar-fill"
                        style={{ width: `${maxSize > 0 ? (c.size / maxSize) * 100 : 0}%` }}
                      />
                    </span>
                    <span className="pop-share-pct">{pct(share)}</span>
                  </span>
                  <span className="pop-num" title={int(c.size)}>
                    {compact(c.size)}
                  </span>
                  <span className="pop-num">{c.income.toFixed(1)}</span>
                  <span className="pop-num">{pct(c.literacy)}</span>
                  <span className="pop-num">{pct(c.happiness)}</span>
                </div>
              );
            })}
          </div>

          <p className="dash-note">
            소득은 계층 평균 부 · 교육은 문해율 · 행복·교육·소득은 인구 가중 평균 ·
            고용률·실업률은 노동력 기준(§8).
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
