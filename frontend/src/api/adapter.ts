// The Simulation API Adapter (UI roadmap Phase 1).
//
// One interface, two implementations:
//   • TauriAdapter — talks to the real `atlas-sim` engine hosted in the Tauri shell
//     via commands + the `world` event. This is the desktop app.
//   • MockAdapter — a self-contained TS fallback for browser-only dev (`npm run dev`),
//     where there is no engine: it advances a clock over the static sample world.
//
// `createAdapter()` picks the right one, so nothing above this file knows or cares
// which is in play.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { SimStatus, WorldSnapshot } from "../types";
import { sampleWorld } from "../data/sampleWorld";

export interface SimAdapter {
  getWorld(): Promise<WorldSnapshot>;
  getStatus(): Promise<SimStatus>;
  setPaused(paused: boolean): Promise<void>;
  setSpeed(speed: number): Promise<void>;
  /** Subscribe to world snapshots pushed each tick; resolves to an unsubscribe fn. */
  onWorld(cb: (world: WorldSnapshot) => void): Promise<() => void>;
  /** True for the real engine, false for the browser mock. */
  readonly live: boolean;
}

/** Whether we are running inside the Tauri webview (vs. a plain browser). */
function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

class TauriAdapter implements SimAdapter {
  readonly live = true;
  getWorld() {
    return invoke<WorldSnapshot>("get_world");
  }
  getStatus() {
    return invoke<SimStatus>("get_status");
  }
  setPaused(paused: boolean) {
    return invoke<void>("set_paused", { paused });
  }
  setSpeed(speed: number) {
    return invoke<void>("set_speed", { speed });
  }
  async onWorld(cb: (world: WorldSnapshot) => void) {
    const unlisten = await listen<WorldSnapshot>("world", (e) => cb(e.payload));
    return unlisten;
  }
}

/** The 360-day calendar the engine uses (12 months of 30 days). */
function clockFor(day: number) {
  return {
    day,
    year: 1 + Math.floor(day / 360),
    month: 1 + Math.floor((day % 360) / 30),
    dayOfMonth: 1 + (day % 30),
  };
}

class MockAdapter implements SimAdapter {
  readonly live = false;
  private day = 0;
  private status: SimStatus = { paused: false, speed: 1 };
  private subscribers = new Set<(w: WorldSnapshot) => void>();
  private timer: ReturnType<typeof setInterval> | null = null;

  private snapshot(): WorldSnapshot {
    return { ...sampleWorld, clock: clockFor(this.day) };
  }

  private push() {
    const snap = this.snapshot();
    this.subscribers.forEach((cb) => cb(snap));
  }

  /** (Re)arm the tick timer to match the current speed, unless paused. */
  private schedule() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    if (this.status.paused) return;
    this.timer = setInterval(() => {
      this.day += 1;
      this.push();
    }, 1000 / Math.max(1, this.status.speed));
  }

  async getWorld() {
    return this.snapshot();
  }
  async getStatus() {
    return this.status;
  }
  async setPaused(paused: boolean) {
    this.status = { ...this.status, paused };
    this.schedule();
  }
  async setSpeed(speed: number) {
    this.status = { ...this.status, speed: Math.max(1, Math.min(60, speed)) };
    this.schedule();
  }
  async onWorld(cb: (world: WorldSnapshot) => void) {
    this.subscribers.add(cb);
    if (!this.timer && !this.status.paused) this.schedule();
    return () => {
      this.subscribers.delete(cb);
    };
  }
}

export function createAdapter(): SimAdapter {
  return inTauri() ? new TauriAdapter() : new MockAdapter();
}
