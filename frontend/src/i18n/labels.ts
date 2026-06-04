// Korean display labels for the engine's enum values.
//
// The simulation (and the browser mock that mirrors it) emit English enum names
// straight from Rust's `{:?}` — goods, governments, relation statuses and good
// tiers. These maps render them in Korean. Anything unmapped falls back to the
// raw value, so a new enum variant shows up readable rather than crashing.

/** Goods — mirrors `Good` in backend/src/resources.rs. */
const GOOD_KO: Record<string, string> = {
  Grain: "곡물",
  Wood: "목재",
  IronOre: "철광석",
  Coal: "석탄",
  Oil: "석유",
  Uranium: "우라늄",
  RareEarth: "희토류",
  Iron: "철",
  Steel: "강철",
  Plastic: "플라스틱",
  Semiconductor: "반도체",
  Battery: "배터리",
  Car: "자동차",
  Electronics: "전자제품",
  Weapon: "무기",
  Computer: "컴퓨터",
  Robot: "로봇",
};

/** Government types — mirrors `Government` in backend/src/components.rs. */
const GOVERNMENT_KO: Record<string, string> = {
  Democracy: "민주제",
  Autocracy: "독재제",
  Monarchy: "군주제",
  Junta: "군사정권",
};

/** Diplomatic relation statuses — mirrors `Relation` in backend/src/diplomacy.rs. */
const RELATION_KO: Record<string, string> = {
  ally: "동맹",
  neutral: "중립",
  rival: "경쟁",
  hostile: "적대",
};

/** Professions / population classes — mirrors `Profession` in backend/src/components.rs. */
const PROFESSION_KO: Record<string, string> = {
  Farmer: "농민",
  Laborer: "노동자",
  Merchant: "상인",
  Engineer: "기술자",
  Researcher: "과학자",
  Soldier: "군인",
  Bureaucrat: "관료",
  Unemployed: "실업자",
};

/** Production tiers used by the market panel. */
const TIER_KO: Record<string, string> = {
  raw: "원자재",
  intermediate: "중간재",
  finished: "완성품",
};

/** Terrain types — mirrors `Terrain` in backend/src/components.rs. */
const TERRAIN_KO: Record<string, string> = {
  Plains: "평원",
  Hills: "구릉",
  Mountains: "산악",
  Coast: "해안",
  Desert: "사막",
};

/** Climate types — mirrors `Climate` in backend/src/components.rs. */
const CLIMATE_KO: Record<string, string> = {
  Temperate: "온대",
  Tropical: "열대",
  Arid: "건조",
  Continental: "대륙성",
  Polar: "한대",
};

export const goodLabel = (g: string): string => GOOD_KO[g] ?? g;
export const professionLabel = (p: string): string => PROFESSION_KO[p] ?? p;
export const governmentLabel = (g: string): string => GOVERNMENT_KO[g] ?? g;
export const relationLabel = (s: string): string => RELATION_KO[s] ?? s;
export const tierLabel = (t: string): string => TIER_KO[t] ?? t;
export const terrainLabel = (t: string): string => TERRAIN_KO[t] ?? t;
export const climateLabel = (c: string): string => CLIMATE_KO[c] ?? c;
