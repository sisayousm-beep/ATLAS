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

export interface WorldView {
  nations: NationView[];
  regions: RegionView[];
}
