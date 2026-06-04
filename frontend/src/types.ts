// View-model types for the map. These will eventually be fed by the backend
// API; for now they are populated from a static sample world.

export interface NationView {
  id: string;
  name: string;
  /** Display colour as a 0xRRGGBB integer (Pixi-friendly). */
  color: number;
}

export interface RegionResource {
  good: string;
  /** Extraction multiplier from the region's deposits. */
  abundance: number;
}

export interface RegionView {
  id: number;
  name: string;
  nationId: string;
  population: number;
  /** Terrain type, e.g. "Plains". */
  terrain: string;
  /** Climate type, e.g. "Temperate". */
  climate: string;
  /** Infrastructure quality, 0..1. Multiplies production & logistics. */
  infrastructure: number;
  /** Natural endowment: raw goods the region is rich in, with abundance. */
  resources: RegionResource[];
  /** Value added in the region over the last month (GDP, design §17). */
  gdp: number;
  /** Normalised layout position, each in 0..1. */
  x: number;
  y: number;
}

export type GoodTier = "raw" | "intermediate" | "finished";

export interface GoodPrice {
  good: string;
  tier: GoodTier;
  /** Live market price. */
  price: number;
  /** Reference price; the live price drifts around this. */
  base: number;
  /** Last day's market supply flow (Phase 7). */
  supply: number;
  /** Last day's market demand flow (Phase 7). */
  demand: number;
}

export interface CorporationView {
  name: string;
  /** Nation the firm is domiciled in (matches `NationView.id`). */
  nationId: string;
  /** Goods the firm produces. */
  industries: string[];
  /** Cash on hand; negative means the firm is bleeding. */
  capital: number;
  employees: number;
  /** Gross sales accumulated this game month (resets monthly). */
  revenue: number;
  /** Margin accumulated this game month (resets monthly). */
  profit: number;
}

export interface EconomyView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Cash in the national treasury. */
  treasury: number;
  /** Gross domestic product: value added across the nation's regions last month. */
  gdp: number;
  /** Cumulative value of goods sold abroad. */
  exports: number;
  /** Cumulative value of goods bought from abroad. */
  imports: number;
}

export interface FinanceView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Central-bank policy interest rate, as a fraction (e.g. 0.05 = 5%). */
  policyRate: number;
  /** Current inflation, as a fraction. */
  inflation: number;
  /** Outstanding government debt. */
  debt: number;
  /** Broad money the central bank has issued. */
  moneySupply: number;
}

export interface PoliticsView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Government type, e.g. "Democracy". */
  government: string;
  /** Regime stability, 0..1. */
  stability: number;
  /** Unrest, 0..1 — the mirror of stability. */
  unrest: number;
  /** Population-weighted average happiness, 0..1 (drives stability, design §13). */
  happiness: number;
}

export interface TechnologyView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Aggregate technology level index (design §12, §17). */
  level: number;
}

export interface LaborView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Working population in the labour force (the modelled active population). */
  laborForce: number;
  /** Heads holding a job. */
  employed: number;
  /** Heads without work: idle industrial labour + explicitly unemployed pops (실업자). */
  unemployed: number;
  /** Unemployed share of the labour force, 0..1 (실업률). */
  unemploymentRate: number;
  /** Employed share of the working-age population, 0..1 (고용률). */
  employmentRate: number;
}

export type RelationStatus = "ally" | "neutral" | "rival" | "hostile";

export interface RelationView {
  /** The two nations, matching `NationView.id`. */
  a: string;
  b: string;
  status: RelationStatus;
  /** Relation score, -1 (hostile) .. 1 (ally). */
  score: number;
}

export interface MilitaryView {
  /** Nation the figures belong to (matches `NationView.id`). */
  nationId: string;
  /** Effective combat power: strength + troops, scaled by tech, sapped by exhaustion. */
  power: number;
  /** Standing military strength, bought from the economy. */
  strength: number;
  /** War exhaustion, 0 (fresh) .. 1 (spent). */
  exhaustion: number;
  /** Whether the nation is currently fighting a war. */
  atWar: boolean;
}

export interface WarView {
  /** The nation that declared the war (matches `NationView.id`). */
  aggressor: string;
  /** The nation defending (matches `NationView.id`). */
  defender: string;
}

export interface WorldView {
  nations: NationView[];
  regions: RegionView[];
  /** Global commodity prices (Phase 2). */
  prices: GoodPrice[];
  /** Profit-seeking firms (Phase 3). */
  corporations: CorporationView[];
  /** Per-nation economy: treasury, GDP, trade balance (design §17). */
  economy: EconomyView[];
  /** Per-nation money: central-bank rate, inflation, debt (Phase 4). */
  finance: FinanceView[];
  /** Whether the world is in a financial crisis (Phase 4, design §11). */
  crisis: boolean;
  /** Per-nation political state: government, stability, unrest (Phase 5, §13). */
  politics: PoliticsView[];
  /** Per-nation technology level (design §12, §17). */
  technology: TechnologyView[];
  /** Per-nation labour market: force, employment, unemployment (실업률/고용률). */
  labor: LaborView[];
  /** Pairwise relations between nations (Phase 5, §14). */
  relations: RelationView[];
  /** Per-nation armed forces: power, strength, exhaustion (Phase 6, §15). */
  military: MilitaryView[];
  /** Active wars between nations (Phase 6, §15). Empty when the world is at peace. */
  wars: WarView[];
}

/** The game clock, for the Phase 1 loop viewer. */
export interface ClockView {
  /** Days since epoch. Day 0 = Year 1, Month 1, Day 1. */
  day: number;
  year: number;
  month: number;
  dayOfMonth: number;
}

/** A full world plus the clock — exactly what the simulation adapter delivers. */
export type WorldSnapshot = WorldView & { clock: ClockView };

/** Loop state mirrored from the engine, for the speed / pause controls. */
export interface SimStatus {
  paused: boolean;
  /** In-game days per real second. */
  speed: number;
}

/** A feed entry raised when something notable changes between two snapshots
 * (Phase 4 HUD notification feed). */
export interface GameNotification {
  id: number;
  /** Game date the event was detected on. */
  year: number;
  month: number;
  /** Visual tone: good news, bad news, or neutral. */
  tone: "good" | "bad" | "info";
  text: string;
}
