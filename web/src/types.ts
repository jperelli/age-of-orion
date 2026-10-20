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

export interface Colony {
  sliders: number[];
  spending: number[];
  build: number;
  bc_to_ship: number;
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

export interface DesignView {
  key: string;
  cost: number;
  attack: number;
  hp: number;
  bombs: number;
  colony: boolean;
  range: number;
}

export interface Abilities {
  colonist_oper_factories: number;
  factory_cost: number;
  fuel_range: number;
  speed: number;
  scanner_range: number;
  colony_for: PlanetType;
  planet_shield: number;
  attack_pct: number;
  hp_pct: number;
  ground_bonus: number;
  [k: string]: number | string;
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
  tech: { level: number[]; invested: number[]; next_cost: number[]; alloc: number[] };
  abilities: Abilities;
  base_cost: number;
  planets: PlanetView[];
  empires: EmpireView[];
  fleets: FleetView[];
  transports: TransportView[];
  designs: DesignView[];
  events: GameEvent[];
}

export type Command =
  | { type: "set_sliders"; planet: number; sliders: number[] }
  | { type: "set_build"; planet: number; design: number }
  | { type: "send_fleet"; from: number; to: number; ships: number[] }
  | { type: "redirect_fleet"; fleet: number; to: number }
  | { type: "send_transport"; from: number; to: number; pop: number }
  | { type: "set_research"; alloc: number[] };

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
  rules?: unknown;
}
