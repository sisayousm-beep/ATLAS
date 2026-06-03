// View-model types for the map. These will eventually be fed by the backend
// API; for now they are populated from a static sample world.

export interface NationView {
  id: string;
  name: string;
  /** Display colour as a 0xRRGGBB integer (Pixi-friendly). */
  color: number;
}

export interface RegionView {
  id: number;
  name: string;
  nationId: string;
  population: number;
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
  /** Per-nation money: central-bank rate, inflation, debt (Phase 4). */
  finance: FinanceView[];
  /** Whether the world is in a financial crisis (Phase 4, design §11). */
  crisis: boolean;
  /** Per-nation political state: government, stability, unrest (Phase 5, §13). */
  politics: PoliticsView[];
  /** Pairwise relations between nations (Phase 5, §14). */
  relations: RelationView[];
  /** Per-nation armed forces: power, strength, exhaustion (Phase 6, §15). */
  military: MilitaryView[];
  /** Active wars between nations (Phase 6, §15). Empty when the world is at peace. */
  wars: WarView[];
}
