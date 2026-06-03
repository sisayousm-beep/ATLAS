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
  // Snapshot near where the Phase 3 sim settles after a few months: car firms
  // pull the chain so cars stay scarce and dear, steel sits a touch below base.
  prices: [
    { good: "Grain", tier: "raw", price: 0.8, base: 1.0 },
    { good: "IronOre", tier: "raw", price: 1.0, base: 2.0 },
    { good: "Coal", tier: "raw", price: 0.4, base: 2.0 },
    { good: "Oil", tier: "raw", price: 0.4, base: 4.0 },
    { good: "Iron", tier: "intermediate", price: 4.0, base: 6.0 },
    { good: "Steel", tier: "intermediate", price: 7.0, base: 12.0 },
    { good: "Plastic", tier: "intermediate", price: 1.0, base: 10.0 },
    { good: "Car", tier: "finished", price: 426.0, base: 120.0 },
  ],
  // Firms from world_gen.rs; capital/staff near the day-120 sim state. Car plants
  // dominate; the steelworks run at a loss and are shedding workers.
  corporations: [
    { name: "Goldfield Foundry", nationId: "aurelia", industries: ["Iron"], capital: 21_567, employees: 17_000 },
    { name: "Aurelia Steel", nationId: "aurelia", industries: ["Steel"], capital: -21_846, employees: 5_905 },
    { name: "Vesper Plastics", nationId: "aurelia", industries: ["Plastic"], capital: 9_367, employees: 9_720 },
    { name: "Aurelia Motors", nationId: "aurelia", industries: ["Car"], capital: 501_376, employees: 17_000 },
    { name: "Sandreach Petrochem", nationId: "khoresan", industries: ["Plastic"], capital: 7_814, employees: 6_480 },
    { name: "Oasis Forge", nationId: "khoresan", industries: ["Iron"], capital: 11_531, employees: 5_000 },
    { name: "Nordheim Foundry", nationId: "nordheim", industries: ["Iron"], capital: 24_272, employees: 18_000 },
    { name: "Nordheim Steel", nationId: "nordheim", industries: ["Steel"], capital: -22_605, employees: 6_561 },
    { name: "Nordheim Motors", nationId: "nordheim", industries: ["Car"], capital: 479_942, employees: 17_000 },
  ],
};
