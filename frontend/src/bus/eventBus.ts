// A tiny typed event bus (UI roadmap Phase 1). Decouples the simulation feed from
// whatever wants to react to it — the store updates on `world`, and any view can
// subscribe without knowing where the data comes from.

import type { ClockView, WorldSnapshot } from "../types";

export interface AppEvents {
  /** A fresh world snapshot arrived from the engine. */
  world: WorldSnapshot;
  /** The clock advanced (carried separately for cheap date-only subscribers). */
  tick: ClockView;
}

type Handler<T> = (payload: T) => void;

class EventBus<Events> {
  private handlers = new Map<keyof Events, Set<Handler<unknown>>>();

  /** Subscribe to an event; returns an unsubscribe function. */
  on<K extends keyof Events>(event: K, handler: Handler<Events[K]>): () => void {
    let set = this.handlers.get(event);
    if (!set) {
      set = new Set();
      this.handlers.set(event, set);
    }
    set.add(handler as Handler<unknown>);
    return () => set!.delete(handler as Handler<unknown>);
  }

  emit<K extends keyof Events>(event: K, payload: Events[K]): void {
    this.handlers.get(event)?.forEach((h) => h(payload));
  }
}

export const bus = new EventBus<AppEvents>();
