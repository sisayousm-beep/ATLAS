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

interface GameState {
  world: WorldSnapshot | null;
  status: SimStatus;
  /** Recent feed entries, newest first (Phase 4 notification feed). */
  notifications: GameNotification[];
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

export const useGameStore = create<GameState>((set, get) => ({
  world: null,
  status: { paused: false, speed: 1 },
  notifications: [],
  live: false,
  ready: false,

  init: async () => {
    if (adapter) return; // guard against React StrictMode's double-invoke
    adapter = createAdapter();

    const [status, world] = await Promise.all([adapter.getStatus(), adapter.getWorld()]);
    set({ status, world, live: adapter.live, ready: true });
    bus.emit("world", world);
    bus.emit("tick", world.clock);

    await adapter.onWorld((w) => {
      const prev = get().world;
      set({ world: w });
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
