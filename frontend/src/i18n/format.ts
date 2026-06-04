// Number formatting helpers for the HUD (UI roadmap Phase 4).
//
// KPIs and money figures get large, so they render compact (Korean 만/억 units);
// plain counts render with locale grouping.

const COMPACT = new Intl.NumberFormat("ko-KR", {
  notation: "compact",
  maximumFractionDigits: 1,
});

/** Compact, Korean-unit number for KPIs and money (e.g. 2.4만, 1.3억). */
export const compact = (n: number): string => COMPACT.format(Math.round(n));

/** Grouped integer (e.g. 1,234,567). */
export const int = (n: number): string => Math.round(n).toLocaleString();

/** Signed compact money, with the sign kept for trade balances. */
export const signed = (n: number): string =>
  `${n < 0 ? "−" : "+"}${compact(Math.abs(n))}`;
