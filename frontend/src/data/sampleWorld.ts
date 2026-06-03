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
    { id: 1, name: "Goldfields", nationId: "aurelia", population: 195_000, x: 0.22, y: 0.35 },
    { id: 2, name: "Port Vesper", nationId: "aurelia", population: 150_000, x: 0.32, y: 0.6 },
    { id: 3, name: "Sandreach", nationId: "khoresan", population: 130_000, x: 0.62, y: 0.4 },
    { id: 4, name: "Oasis Hold", nationId: "khoresan", population: 63_000, x: 0.7, y: 0.62 },
    { id: 5, name: "Frostmark", nationId: "nordheim", population: 175_000, x: 0.5, y: 0.18 },
  ],
};
