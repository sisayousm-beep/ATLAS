import type { WorldView } from "../types";

// Mirrors backend/src/world_gen.rs so the map shows a coherent world until the
// backend exposes a live API. Populations are the Phase 1 starting sizes.

export const sampleWorld: WorldView = {
  nations: [
    { id: "aurelia", name: "Aurelia", color: 0x4f9d69 },
    { id: "khoresan", name: "Khoresan", color: 0xd9a441 },
    { id: "nordheim", name: "Nordheim", color: 0x5b7fb5 },
  ],
  regions: [
    { id: 1, name: "Goldfields", nationId: "aurelia", population: 220_000, x: 0.22, y: 0.35 },
    { id: 2, name: "Port Vesper", nationId: "aurelia", population: 150_000, x: 0.32, y: 0.6 },
    { id: 3, name: "Sandreach", nationId: "khoresan", population: 138_000, x: 0.62, y: 0.4 },
    { id: 4, name: "Oasis Hold", nationId: "khoresan", population: 89_000, x: 0.7, y: 0.62 },
    { id: 5, name: "Frostmark", nationId: "nordheim", population: 205_000, x: 0.5, y: 0.18 },
  ],
  // Snapshot near the prices the Phase 2 sim settles into after a few months
  // (steel oversupplied → floored; cars scarce → ceilinged; iron balanced).
  prices: [
    { good: "Grain", tier: "raw", price: 0.8, base: 1.0 },
    { good: "IronOre", tier: "raw", price: 2.0, base: 2.0 },
    { good: "Coal", tier: "raw", price: 1.0, base: 2.0 },
    { good: "Oil", tier: "raw", price: 0.4, base: 4.0 },
    { good: "Iron", tier: "intermediate", price: 5.6, base: 6.0 },
    { good: "Steel", tier: "intermediate", price: 1.2, base: 12.0 },
    { good: "Plastic", tier: "intermediate", price: 1.0, base: 10.0 },
    { good: "Car", tier: "finished", price: 1200.0, base: 120.0 },
  ],
};
