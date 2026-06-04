// Notification feed source (UI roadmap Phase 4).
//
// The engine pushes a fresh world snapshot each tick but no event stream, so the
// feed is derived here: compare two consecutive snapshots and raise an entry for
// anything notable that changed — wars igniting or ending, the financial-crisis
// flag flipping, and coups/revolutions.

import type { GameNotification, WorldSnapshot } from "../types";
import { governmentLabel } from "../i18n/labels";

type Raw = Omit<GameNotification, "id">;

export function diffWorld(prev: WorldSnapshot, next: WorldSnapshot): Raw[] {
  const out: Raw[] = [];
  const { year, month } = next.clock;
  const name = (id: string) => next.nations.find((n) => n.id === id)?.name ?? id;
  const key = (w: { aggressor: string; defender: string }) => `${w.aggressor}>${w.defender}`;

  const before = new Set(prev.wars.map(key));
  const after = new Set(next.wars.map(key));
  for (const w of next.wars) {
    if (!before.has(key(w))) {
      out.push({ year, month, tone: "bad", text: `⚔ ${name(w.aggressor)} → ${name(w.defender)} 전쟁 발발` });
    }
  }
  for (const w of prev.wars) {
    if (!after.has(key(w))) {
      out.push({ year, month, tone: "good", text: `🕊 ${name(w.aggressor)} · ${name(w.defender)} 종전` });
    }
  }

  if (next.crisis && !prev.crisis) out.push({ year, month, tone: "bad", text: "⚠ 세계 금융 위기 발생" });
  if (!next.crisis && prev.crisis) out.push({ year, month, tone: "good", text: "✓ 금융 위기 진정" });

  const govBefore = new Map(prev.politics.map((p) => [p.nationId, p.government]));
  for (const p of next.politics) {
    const was = govBefore.get(p.nationId);
    if (was && was !== p.government) {
      out.push({
        year,
        month,
        tone: "bad",
        text: `🏛 ${name(p.nationId)} 정변: ${governmentLabel(was)} → ${governmentLabel(p.government)}`,
      });
    }
  }

  return out;
}
