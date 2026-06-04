import type { WorldView } from "../types";

// Mirrors backend/src/world_gen.rs so the map shows a coherent world until the
// backend exposes a live API. Populations are the Phase 1 starting sizes.

export const sampleWorld: WorldView = {
  nations: [
    { id: "aurelia", name: "Aurelia", color: 0x4f9d69 },
    { id: "khoresan", name: "Khoresan", color: 0xd9a441 },
    { id: "nordheim", name: "Nordheim", color: 0x5b7fb5 },
  ],
  // terrain/climate/infra/deposits mirror backend/src/world_gen.rs.
  regions: [
    { id: 1, name: "Goldfields", nationId: "aurelia", population: 220_000, terrain: "Plains", climate: "Temperate", infrastructure: 0.7, resources: [{ good: "IronOre", abundance: 0.8 }, { good: "Coal", abundance: 0.7 }], gdp: 9_500, x: 0.22, y: 0.35 },
    { id: 2, name: "Port Vesper", nationId: "aurelia", population: 150_000, terrain: "Coast", climate: "Temperate", infrastructure: 0.85, resources: [{ good: "Oil", abundance: 0.9 }], gdp: 14_000, x: 0.32, y: 0.6 },
    { id: 3, name: "Sandreach", nationId: "khoresan", population: 138_000, terrain: "Desert", climate: "Arid", infrastructure: 0.4, resources: [{ good: "Oil", abundance: 1.2 }], gdp: 4_200, x: 0.62, y: 0.4 },
    { id: 4, name: "Oasis Hold", nationId: "khoresan", population: 89_000, terrain: "Hills", climate: "Arid", infrastructure: 0.5, resources: [{ good: "IronOre", abundance: 0.7 }, { good: "Coal", abundance: 0.5 }], gdp: 3_100, x: 0.7, y: 0.62 },
    { id: 5, name: "Frostmark", nationId: "nordheim", population: 205_000, terrain: "Plains", climate: "Continental", infrastructure: 0.75, resources: [{ good: "IronOre", abundance: 0.9 }, { good: "Coal", abundance: 0.8 }, { good: "Oil", abundance: 0.4 }], gdp: 16_500, x: 0.5, y: 0.18 },
  ],
  // Snapshot near where the Phase 3 sim settles after a few months: car firms
  // pull the chain so cars stay scarce and dear, steel sits a touch below base.
  // supply/demand are the last trading day's flows (Phase 7). Raw and intermediate
  // goods sit in surplus (supply > demand) so their prices ride below base; the car
  // chain runs scarce (demand > supply), holding cars dear above base.
  prices: [
    { good: "Grain", tier: "raw", price: 0.8, base: 1.0, supply: 3_200, demand: 2_600 },
    { good: "IronOre", tier: "raw", price: 1.0, base: 2.0, supply: 1_400, demand: 900 },
    { good: "Coal", tier: "raw", price: 0.4, base: 2.0, supply: 1_800, demand: 700 },
    { good: "Oil", tier: "raw", price: 0.4, base: 4.0, supply: 900, demand: 320 },
    { good: "Iron", tier: "intermediate", price: 4.0, base: 6.0, supply: 520, demand: 470 },
    { good: "Steel", tier: "intermediate", price: 7.0, base: 12.0, supply: 300, demand: 240 },
    { good: "Plastic", tier: "intermediate", price: 1.0, base: 10.0, supply: 410, demand: 180 },
    { good: "Car", tier: "finished", price: 426.0, base: 120.0, supply: 60, demand: 95 },
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
  // Per-nation economy (design §17). GDP is the sum of the regions' value added;
  // the two industrial powers (Aurelia, Nordheim) out-produce thin-industry
  // Khoresan, which also runs a steep import deficit feeding others' factories.
  economy: [
    { nationId: "aurelia", treasury: 22_000, gdp: 23_500, exports: 48_000, imports: 22_000 },
    { nationId: "khoresan", treasury: 0, gdp: 7_300, exports: 9_000, imports: 31_000 },
    { nationId: "nordheim", treasury: 9_000, gdp: 16_500, exports: 41_000, imports: 19_000 },
  ],
  // Phase 4, near the year-1 sim state. Aurelia and Nordheim run fat corporate-tax
  // surpluses — stable money, zero inflation, rates cut to the floor. Khoresan's
  // thin industry can't cover welfare, so it monetises the deficit: debt builds,
  // inflation runs ~5%/mo, and its central bank hikes to fight it.
  finance: [
    { nationId: "aurelia", policyRate: 0.0, inflation: 0.0, debt: 0, moneySupply: 50_000 },
    { nationId: "khoresan", policyRate: 0.107, inflation: 0.048, debt: 19_297, moneySupply: 57_297 },
    { nationId: "nordheim", policyRate: 0.0, inflation: 0.0, debt: 0, moneySupply: 44_000 },
  ],
  crisis: false,
  // Phase 5, near the year-1 sim state. Aurelia's liberal democracy represents
  // its people and runs calm; Nordheim's monarchy is steady. Khoresan's autocracy
  // sits on a poorer, inflation-bitten populace, so stability sags and unrest runs
  // hot — the kind of pressure that, sustained, tips into a coup.
  politics: [
    { nationId: "aurelia", government: "Democracy", stability: 0.82, unrest: 0.18, happiness: 0.7 },
    { nationId: "khoresan", government: "Autocracy", stability: 0.46, unrest: 0.54, happiness: 0.42 },
    { nationId: "nordheim", government: "Monarchy", stability: 0.71, unrest: 0.29, happiness: 0.63 },
  ],
  // Phase 7 research: scientific Nordheim has climbed the ladder past its
  // neighbours (design §12); aggressive Khoresan lags, diplomatic Aurelia holds.
  technology: [
    { nationId: "aurelia", level: 1.0 },
    { nationId: "khoresan", level: 0.8 },
    { nationId: "nordheim", level: 1.3 },
  ],
  // Labour market (실업률/고용률). The healthy industrial powers run near full
  // employment; broke, inflation-bitten Khoresan — whose thin industry can't
  // absorb its workers — carries the slack. employmentRate (고용률) is the share
  // of the working-age population in work, so it reads ~60% even at low 실업률.
  labor: [
    { nationId: "aurelia", laborForce: 370_000, employed: 354_460, unemployed: 15_540, unemploymentRate: 0.042, employmentRate: 0.604 },
    { nationId: "khoresan", laborForce: 227_000, employed: 205_435, unemployed: 21_565, unemploymentRate: 0.095, employmentRate: 0.570 },
    { nationId: "nordheim", laborForce: 205_000, employed: 197_825, unemployed: 7_175, unemploymentRate: 0.035, employmentRate: 0.608 },
  ],
  // Commerce binds the two industrial powers into an alliance despite the
  // democracy/monarchy divide; the authoritarian pair keeps a cool neutrality;
  // and the ideological gulf leaves Aurelia and Khoresan rivals.
  relations: [
    { a: "aurelia", b: "nordheim", status: "ally", score: 0.52 },
    { a: "khoresan", b: "nordheim", status: "neutral", score: 0.34 },
    { a: "aurelia", b: "khoresan", status: "rival", score: -0.18 },
  ],
  // Phase 6, near the year-1 sim state. Each nation arms from its economy, so the
  // surplus-rich industrial powers field the strongest militaries while broke
  // Khoresan can barely fund one. No one is at war: the rivalry is cold and no
  // aggressor holds a decisive enough edge — commerce keeps the peace (design §15).
  military: [
    { nationId: "aurelia", power: 405, strength: 150, exhaustion: 0, atWar: false },
    { nationId: "khoresan", power: 137, strength: 5, exhaustion: 0, atWar: false },
    { nationId: "nordheim", power: 432, strength: 120, exhaustion: 0, atWar: false },
  ],
  wars: [],
};
