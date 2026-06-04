// Company Dashboard (UI roadmap Phase 8 — 기업을 플레이 요소로).
//
// A full-screen overlay listing every firm with the books the sim actually keeps:
// this month's revenue and profit, head-count, capital, and a market share read
// off each firm's share of its primary product's revenue. Clicking a firm opens
// its detail — products, scale and margin — so the player can read the corporate
// economy (완료 조건: 기업 경제 확인 가능).
//
// R&D and per-firm investment are not modelled by the engine yet (the design's
// company depth is post-MVP), so they are surfaced as "not yet modelled" rather
// than faked.

import { useEffect, useState } from "react";
import type { CorporationView, WorldView } from "../types";
import { goodLabel } from "../i18n/labels";
import { compact, signed } from "../i18n/format";

interface Props {
  world: WorldView;
  onClose: () => void;
}

const pct1 = (v: number) => `${(v * 100).toFixed(1)}%`;
/** A firm's primary product is its highest-tier output (last in the chain). */
const primaryGood = (c: CorporationView) => c.industries[c.industries.length - 1] ?? "";

export function CompanyDashboard({ world, onClose }: Props) {
  const [openFirm, setOpenFirm] = useState<string | null>(null);

  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const firms = world.corporations;
  const nationName = (id: string) => world.nations.find((n) => n.id === id)?.name ?? id;

  // Market share: a firm's revenue against the revenue of every firm selling the
  // same primary product. Falls back to share of all firms' revenue when no peer
  // sells anything this month.
  const revByGood = new Map<string, number>();
  for (const c of firms) {
    const g = primaryGood(c);
    revByGood.set(g, (revByGood.get(g) ?? 0) + Math.max(0, c.revenue));
  }
  const totalRevenue = firms.reduce((s, c) => s + Math.max(0, c.revenue), 0);
  const shareOf = (c: CorporationView) => {
    const pool = revByGood.get(primaryGood(c)) ?? 0;
    if (pool > 0) return Math.max(0, c.revenue) / pool;
    return totalRevenue > 0 ? Math.max(0, c.revenue) / totalRevenue : 0;
  };

  const totalProfit = firms.reduce((s, c) => s + c.profit, 0);
  const totalEmployees = firms.reduce((s, c) => s + c.employees, 0);

  // Sort by revenue, biggest firms first.
  const rows = [...firms].sort((a, b) => b.revenue - a.revenue);

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label="기업">
        <header className="dash-head">
          <h2>기업</h2>
          <span className="dash-gov">매출 · 이익 · 점유율</span>
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <div className="dash-body">
          <div className="dash-kpis macro">
            <Kpi label="기업 수" value={String(firms.length)} />
            <Kpi label="총매출" value={compact(totalRevenue)} />
            <Kpi label="총이익" value={signed(totalProfit)} />
            <Kpi label="총고용" value={compact(totalEmployees)} />
          </div>

          <div className="firm-table">
            <div className="firm-head">
              <span>기업</span>
              <span className="firm-num">매출</span>
              <span className="firm-num">이익</span>
              <span className="firm-num">직원</span>
              <span className="firm-num">점유율</span>
            </div>
            {rows.map((c) => {
              const open = openFirm === c.name;
              const share = shareOf(c);
              const margin = c.revenue > 0 ? c.profit / c.revenue : 0;
              return (
                <div key={c.name} className={`firm-block ${open ? "open" : ""}`}>
                  <button
                    className="firm-row"
                    onClick={() => setOpenFirm(open ? null : c.name)}
                    aria-expanded={open}
                  >
                    <span className="firm-name">
                      <span className="firm-caret">{open ? "▾" : "▸"}</span>
                      <span>{c.name}</span>
                      <span className="firm-nation">{nationName(c.nationId)}</span>
                    </span>
                    <span className="firm-num">{compact(c.revenue)}</span>
                    <span className={`firm-num ${c.profit < 0 ? "neg" : "pos"}`}>
                      {signed(c.profit)}
                    </span>
                    <span className="firm-num">{compact(c.employees)}</span>
                    <span className="firm-num">{pct1(share)}</span>
                  </button>

                  {open && (
                    <div className="firm-detail">
                      <div className="firm-products">
                        <span className="firm-detail-label">제품</span>
                        <span className="firm-chips">
                          {c.industries.map((g) => (
                            <span key={g} className="firm-chip">
                              {goodLabel(g)}
                            </span>
                          ))}
                        </span>
                      </div>
                      <div className="firm-stats">
                        <Stat label="자본" value={signed(c.capital)} neg={c.capital < 0} />
                        <Stat label="마진율" value={pct1(margin)} neg={margin < 0} />
                        <Stat label="시장 점유율" value={pct1(share)} />
                        <Stat label="공장 위치" value={nationName(c.nationId)} />
                      </div>
                      <p className="firm-deferred">R&amp;D · 투자 시스템은 차기 단계(post-MVP)</p>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
          <p className="dash-note">
            매출·이익은 이번 달 누적(매월 초기화) · 점유율은 동일 제품 시장 내 매출 비중.
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

function Stat({ label, value, neg }: { label: string; value: string; neg?: boolean }) {
  return (
    <div className="firm-stat">
      <span className="firm-stat-label">{label}</span>
      <span className={`firm-stat-value ${neg ? "neg" : ""}`}>{value}</span>
    </div>
  );
}
