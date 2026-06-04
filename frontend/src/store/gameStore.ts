// Global game state (UI roadmap Phase 1), backed by Zustand.
//
// Holds the latest world snapshot and the loop status, and exposes the loop
// controls. It owns the single adapter instance and subscribes to its `world`
// feed, re-broadcasting onto the event bus so views can react either way.

import { create } from "zustand";
import type { GameNotification, SimStatus, WorldSnapshot } from "../types";
import { createAdapter, type SimAdapter } from "../api/adapter";
import { bus } from "../bus/eventBus";
import { diffWorld } from "../bus/notifications";

/** Most recent notifications kept in the feed. */
const FEED_LIMIT = 30;
/** Months of per-nation history kept for the Phase 5 dashboard charts (~10y). */
const HISTORY_LIMIT = 120;
/** Months of world macro history kept for the Phase 6 dashboard (50y range). */
const MACRO_LIMIT = 600;

/** One month's headline figures for a nation (Phase 5 dashboard charts). */
export interface NationSample {
  year: number;
  month: number;
  gdp: number;
  debt: number;
  inflation: number;
  happiness: number;
}

/** One month's world-economy aggregates (Phase 6 economic dashboard).
 * GDP / employment / money supply are world sums; inflation and the interest
 * rate are simple means across nations. Employment is the head-count employed
 * by firms (no unemployment series exists in the engine). */
export interface MacroSample {
  year: number;
  month: number;
  gdp: number;
  employment: number;
  inflation: number;
  moneySupply: number;
  interestRate: number;
}

const mean = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0);

interface GameState {
  world: WorldSnapshot | null;
  status: SimStatus;
  /** Recent feed entries, newest first (Phase 4 notification feed). */
  notifications: GameNotification[];
  /** Per-nation monthly history, oldest first (Phase 5 dashboard charts). */
  history: Record<string, NationSample[]>;
  /** World macro history, oldest first (Phase 6 economic dashboard). */
  macro: MacroSample[];
  /** True when driven by the real engine, false on the browser mock. */
  live: boolean;
  ready: boolean;
  init: () => Promise<void>;
  pause: () => void;
  resume: () => void;
  setSpeed: (speed: number) => void;
}

let adapter: SimAdapter | null = null;
let notifId = 1;
/** Last month appended to history, so each month is recorded once. */
let lastMonthKey: number | null = null;

/** Append this month's figures, once per game month, to both the per-nation
 * history (Phase 5) and the world macro history (Phase 6). Returns the previous
 * series unchanged when the month is unchanged, so charts don't churn. */
function record(
  hist: Record<string, NationSample[]>,
  macro: MacroSample[],
  world: WorldSnapshot,
): { history: Record<string, NationSample[]>; macro: MacroSample[] } {
  const key = world.clock.year * 12 + world.clock.month;
  if (key === lastMonthKey) return { history: hist, macro };
  lastMonthKey = key;

  const next: Record<string, NationSample[]> = {};
  for (const n of world.nations) {
    const econ = world.economy.find((e) => e.nationId === n.id);
    const fin = world.finance.find((f) => f.nationId === n.id);
    const pol = world.politics.find((p) => p.nationId === n.id);
    const point: NationSample = {
      year: world.clock.year,
      month: world.clock.month,
      gdp: econ?.gdp ?? 0,
      debt: fin?.debt ?? 0,
      inflation: fin?.inflation ?? 0,
      happiness: pol?.happiness ?? 0,
    };
    next[n.id] = [...(hist[n.id] ?? []), point].slice(-HISTORY_LIMIT);
  }

  const macroPoint: MacroSample = {
    year: world.clock.year,
    month: world.clock.month,
    gdp: world.economy.reduce((s, e) => s + e.gdp, 0),
    employment: world.corporations.reduce((s, c) => s + c.employees, 0),
    inflation: mean(world.finance.map((f) => f.inflation)),
    moneySupply: world.finance.reduce((s, f) => s + f.moneySupply, 0),
    interestRate: mean(world.finance.map((f) => f.policyRate)),
  };

  return { history: next, macro: [...macro, macroPoint].slice(-MACRO_LIMIT) };
}

export const useGameStore = create<GameState>((set, get) => ({
  world: null,
  status: { paused: false, speed: 1 },
  notifications: [],
  history: {},
  macro: [],
  live: false,
  ready: false,

  init: async () => {
    if (adapter) return; // guard against React StrictMode's double-invoke
    adapter = createAdapter();

    const [status, world] = await Promise.all([adapter.getStatus(), adapter.getWorld()]);
    set({ status, world, live: adapter.live, ready: true, ...record({}, [], world) });
    bus.emit("world", world);
    bus.emit("tick", world.clock);

    await adapter.onWorld((w) => {
      const prev = get().world;
      set({ world: w, ...record(get().history, get().macro, w) });
      if (prev) {
        const raised = diffWorld(prev, w).map((r) => ({ ...r, id: notifId++ }));
        if (raised.length > 0) {
          set((s) => ({ notifications: [...raised, ...s.notifications].slice(0, FEED_LIMIT) }));
        }
      }
      bus.emit("world", w);
      bus.emit("tick", w.clock);
    });
  },

  pause: () => {
    adapter?.setPaused(true);
    set((s) => ({ status: { ...s.status, paused: true } }));
  },
  resume: () => {
    adapter?.setPaused(false);
    set((s) => ({ status: { ...s.status, paused: false } }));
  },
  setSpeed: (speed) => {
    adapter?.setSpeed(speed);
    set((s) => ({ status: { ...s.status, speed } }));
  },
}));
