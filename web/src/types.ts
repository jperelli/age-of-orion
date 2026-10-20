// Mirrors of the JSON produced by the Rust core (crates/core/src/view.rs).

export type PlanetType =
  | "not_habitable" | "radiated" | "toxic" | "inferno" | "dead" | "tundra" | "barren"
  | "minimal" | "desert" | "steppe" | "arid" | "ocean" | "jungle" | "terran";
export type StarType = "yellow" | "red" | "green" | "white" | "blue" | "neutron";
export type Special = "ultra_poor" | "poor" | "normal" | "artifacts" | "rich" | "ultra_rich" | "orion";
export type Growth = "hostile" | "normal" | "fertile" | "gaia";
export type Race =
  | "human" | "mrrshan" | "silicoid" | "sakkra" | "psilon"
  | "alkari" | "klackon" | "bulrathi" | "meklar" | "darlok";
export type TechField = "computer" | "construction" | "force_field" | "planetology" | "propulsion" | "weapon";
export type GalaxySize = "small" | "medium" | "large" | "huge";
export type Difficulty = "simple" | "easy" | "average" | "hard" | "impossible";

export const RACES: Race[] = [
  "human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok",
];
export const FIELDS: TechField[] = ["computer", "construction", "force_field", "planetology", "propulsion", "weapon"];
export const SLIDERS = ["ship", "def", "ind", "eco", "tech"] as const;
export const PLANET_TYPES: PlanetType[] = [
  "not_habitable", "radiated", "toxic", "inferno", "dead", "tundra", "barren",
  "minimal", "desert", "steppe", "arid", "ocean", "jungle", "terran",
];

export const BUILD_STARGATE = 6;

export interface Colony {
  sliders: number[];
  spending: number[];
  build: number;
  build_cost: number;
  bc_to_ship: number;
  rally: number | null;
  factory_cost: number;
  max_factories: number;
  research: number;
  waste: number;
  clean_eco: number;
  pop_oper_fact: number;
}

export interface PlanetView {
  id: number;
  name: string;
  x: number;
  y: number;
  star: StarType;
  explored: boolean;
  nebula: boolean;
  range: number;
  orion: boolean;
  guardian: boolean;
  image: number | null;
  kind: PlanetType | null;
  special: Special | null;
  growth: Growth | null;
  max_pop: number | null;
  owner: number | null;
  pop: number | null;
  factories: number | null;
  bases: number | null;
  shield: number | null;
  stargate: boolean;
  prod: number | null;
  colonizable: boolean;
  colony: Colony | null;
}

export interface FleetView {
  id: number | null;
  owner: number;
  at: number | null;
  x: number;
  y: number;
  dest: number | null;
  eta: number | null;
  ships: number[];
  names: string[];
  speed: number;
  range: number;
}

export interface TransportView {
  id: number;
  owner: number;
  x: number;
  y: number;
  dest: number;
  eta: number;
  pop: number | null;
}

export interface EmpireView {
  id: number;
  leader: string;
  race: Race;
  banner: number;
  alive: boolean;
  ai: boolean;
  known_colonies: number;
}

export interface ShipDesign {
  name: string;
  hull: number;
  look: number;
  weapons: number[];
  weapon_counts: number[];
  engine: number;
  engines?: number;
  specials: number[];
  shield: number;
  jammer: number;
  comp: number;
  armor: number;
  man: number;
  hp?: number;
}

export interface DesignStats {
  cost: number;
  space: number;
  free_space: number;
  engines: number;
  hp: number;
  speed: number;
  attack_level: number;
  defense: number;
  missile_defense: number;
  absorb: number;
  max_weapons: number[];
  error: string | null;
}

export interface DesignView extends ShipDesign {
  stats: DesignStats;
  count: number;
  range: number;
  colony: PlanetType | null;
}

export interface KnownParts {
  weapons: number[];
  engines: number[];
  comps: number[];
  jammers: number[];
  shields: number[];
  armor: number[];
  specials: number[];
}

export interface TechChoice {
  tech: number;
  key: string;
  cost: number;
}

export interface FieldView {
  field: TechField;
  level: number;
  project: number;
  project_key: string;
  cost: number;
  invested: number;
  alloc: number;
  choices: TechChoice[];
  completed: TechChoice[];
}

export interface Nebula {
  kind: number;
  x: number;
  y: number;
  rects: number[][];
}

/** Static ship part tables (crates/core/src/data.rs). */
export interface Parts {
  weapons: { key: string; damage_min: number; damage_max: number; range: number; extra_acc: number; halve_shield: boolean; bomb: boolean; fire: number; shots: number; bio: boolean; streams: number; tech: number }[];
  comps: { key: string; level: number }[];
  jammers: { key: string; level: number }[];
  engines: { key: string; warp: number }[];
  armor: { key: string; armor: number }[];
  shields: { key: string; absorb: number }[];
  specials: { key: string; kind: number }[];
  hulls: { key: string; hits: number; space: number; defense: number }[];
}

export interface Abilities {
  colonist_oper_factories: number;
  factory_cost: number;
  fuel_range: number;
  speed: number;
  scanner_range: number;
  colony_for: PlanetType;
  planet_shield: number;
  ground_bonus: number;
  engine_warp: number;
  stargates: boolean;
  [k: string]: number | string | boolean;
}

export interface GameEvent {
  to: number[];
  type: string;
  [k: string]: unknown;
}

export interface View {
  turn: number;
  year: number;
  player: number;
  width: number;
  height: number;
  alive: boolean;
  winner: number | null;
  reserve: number;
  total_prod: number;
  maintenance: number;
  research: number;
  bombard: boolean;
  tech: FieldView[];
  abilities: Abilities;
  base_cost: number;
  stargate_cost: number;
  planets: PlanetView[];
  nebulae: Nebula[];
  empires: EmpireView[];
  fleets: FleetView[];
  transports: TransportView[];
  designs: DesignView[];
  parts: KnownParts;
  events: GameEvent[];
}

export type Command =
  | { type: "set_sliders"; planet: number; sliders: number[] }
  | { type: "set_build"; planet: number; design: number }
  | { type: "send_fleet"; from: number; to: number; ships: number[] }
  | { type: "redirect_fleet"; fleet: number; to: number }
  | { type: "send_transport"; from: number; to: number; pop: number }
  | { type: "set_research"; alloc: number[] }
  | { type: "set_research_project"; field: number; tech: number }
  | { type: "set_rally"; planet: number; to: number | null }
  | { type: "scrap_ships"; planet: number; ships: number[] }
  | { type: "save_design"; slot: number | null; design: ShipDesign }
  | { type: "scrap_design"; design: number }
  | { type: "set_bombard"; enabled: boolean };

export interface NewGameOptions {
  seed: number;
  galaxy_size: GalaxySize;
  difficulty: Difficulty;
  players: number;
  race: Race | null;
  banner: number | null;
  star_names?: string[];
  home_names?: string[];
  leader_names?: string[];
  leader?: string | null;
  orion_name?: string | null;
  start_design_names?: string[];
  ship_names?: string[][];
  rules?: unknown;
}
