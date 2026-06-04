// Market Visualization (UI roadmap Phase 7 — 시장 시스템 시각화).
//
// A full-screen overlay listing every traded good with its live price, the last
// trading day's supply and demand, a trend read off the price-vs-reference gap,
// and a price sparkline from the history the store accumulates — so the player
// can watch the market move (완료 조건: 시장 변화 관찰 가능).

import { useEffect } from "react";
import type { GoodPrice, WorldView } from "../types";
import { goodLabel, tierLabel } from "../i18n/labels";
import { compact } from "../i18n/format";
import { LineChart } from "./LineChart";

interface Props {
  world: WorldView;
  priceHistory: Record<string, number[]>;
  onClose: () => void;
}

type TrendKey = "up" | "down" | "flat";

/** Trend off the price-vs-reference gap: dear (above base) vs cheap (below). */
function trendOf(p: GoodPrice): { key: TrendKey; arrow: string; label: string } {
  if (p.price > p.base * 1.05) return { key: "up", arrow: "▲", label: "상승" };
  if (p.price < p.base * 0.95) return { key: "down", arrow: "▼", label: "하락" };
  return { key: "flat", arrow: "—", label: "안정" };
}

const sparkColor = (key: TrendKey) =>
  key === "up" ? "#e0686d" : key === "down" ? "#5fb98a" : "#7fa6d9";

export function MarketDashboard({ world, priceHistory, onClose }: Props) {
  // Close on Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="dash-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dash" role="dialog" aria-label="시장">
        <header className="dash-head">
          <h2>시장</h2>
          <span className="dash-gov">가격 · 수급 · 추세</span>
          <button className="dash-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        <div className="dash-body">
          <div className="market-table">
            <div className="market-head">
              <span>상품</span>
              <span className="market-num">가격</span>
              <span className="market-num">공급</span>
              <span className="market-num">수요</span>
              <span className="market-num">추세</span>
              <span className="market-num">변화</span>
            </div>
            {world.prices.map((p) => {
              const t = trendOf(p);
              return (
                <div key={p.good} className="market-row">
                  <span className="market-good">
                    <span className={`tier-dot tier-${p.tier}`} title={tierLabel(p.tier)} />
                    {goodLabel(p.good)}
                  </span>
                  <span className="market-num market-price">{p.price.toFixed(1)}</span>
                  <span className="market-num">{compact(p.supply)}</span>
                  <span className="market-num">{compact(p.demand)}</span>
                  <span className={`market-num trend trend-${t.key}`}>
                    {t.arrow} {t.label}
                  </span>
                  <span className="market-spark">
                    <LineChart data={priceHistory[p.good] ?? []} color={sparkColor(t.key)} />
                  </span>
                </div>
              );
            })}
          </div>
          <p className="dash-note">
            공급·수요는 직전 거래일 유통량 · 추세는 기준가 대비 현재가 · 그래프는 가격 추이.
          </p>
        </div>
      </div>
    </div>
  );
}
