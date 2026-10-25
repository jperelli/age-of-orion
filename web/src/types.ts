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
  /** Sliders locked by the player (clicking their name). */
  locks: boolean[];
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
  /** Colonists leaving at the end of the turn: [pop, destination]. */
  transport: [number, number] | null;
}

export interface PlanetView {
  id: number;
  /** Empty until explored (the game screen shows "Unexplored"). */
  name: string;
  x: number;
  y: number;
  star: StarType;
  /** Star picture variant on the classic map (0 or 6). */
  look: number;
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
  rebels: number | null;
  rebellion: boolean;
  reserve: number | null;
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
  /** Picture (look) and hull size of each design of the owner. */
  looks: [number, number][];
  speed: number;
  range: number;
  /** The player can change its destination (still at its planet, or Hyperspace Communications). */
  redirect: boolean;
}

export interface TransportView {
  id: number;
  owner: number;
  x: number;
  y: number;
  dest: number;
  eta: number;
  pop: number | null;
  /** The player can change its destination (Hyperspace Communications). */
  redirect: boolean;
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

export type TechEffect =
  | { type: "level"; from: number; to: number }
  | { type: "value"; what: string; from: number; to: number; superseded: boolean }
  | { type: "gain"; what: string; cost: number }
  | { type: "colonize"; kind: PlanetType; superseded: boolean }
  | { type: "part"; kind: "weapon" | "comp" | "jammer" | "shield" | "armor" | "engine" | "special"; index: number };

export interface TechChoice {
  tech: number;
  key: string;
  cost: number;
  /** What the tech gives. */
  effects: TechEffect[];
}

export interface FieldView {
  field: TechField;
  level: number;
  project: number;
  project_key: string;
  cost: number;
  invested: number;
  alloc: number;
  /** Research slider locked by the player. */
  locked: boolean;
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
  weapons: {
    key: string;
    damage_min: number;
    damage_max: number;
    range: number;
    extra_acc: number;
    halve_shield: boolean;
    bomb: boolean;
    damage_fade: boolean;
    miss_type: number;
    fire: number;
    shots: number;
    bio: boolean;
    streams: number;
    tech: number;
    fuel: number;
    dtbl: number[];
    sound: number;
  }[];
  comps: { key: string; level: number }[];
  jammers: { key: string; level: number }[];
  engines: { key: string; warp: number }[];
  armor: { key: string; armor: number }[];
  shields: { key: string; absorb: number }[];
  specials: { key: string; kind: number }[];
  hulls: { key: string; hits: number; space: number; defense: number }[];
  /** Tech keys per field and tech number ("" = none). */
  techs: string[][];
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

export type Treaty = "none" | "non_aggression" | "alliance" | "war" | "final_war";
export type SpyMode = "hide" | "espionage" | "sabotage";
export type TechRef = [number, number];

export interface AudienceOptions {
  greeting: number;
  talks: boolean;
  later: boolean;
  nap: boolean;
  alliance: boolean;
  peace: boolean;
  war_on: number[];
  break_alliance_with: number[];
  trade: number[];
  tribute: number[];
  break_treaty: boolean;
  break_trade: boolean;
  threaten: boolean;
  exchange: boolean;
}

export interface RelationView {
  empire: number;
  contact: boolean;
  met: boolean;
  treaty: Treaty;
  relation: number;
  trade_bc: number;
  trade_percent: number;
  ceasefire: number;
  spies: number;
  spying: number;
  spy_mode: SpyMode;
  personality: number | null;
  objective: number | null;
  spy_levels: number[] | null;
  spy_report: TechRef[] | null;
  spy_report_year: number | null;
  audience: AudienceOptions | null;
}

export interface MsgParams {
  planet: number | null;
  field: number | null;
  empire: number | null;
  bc: number;
  tech: TechRef | null;
  offer_bc: number;
  offer_tech: TechRef | null;
  techs: TechRef[];
}

export interface Incoming {
  to: number;
  from: number;
  kind: number;
  params: MsgParams;
  then: number | null;
  needs_answer: boolean;
}

export interface SabotageTarget {
  planet: number;
  name: string;
  kind: PlanetType;
  pop: number;
  factories: number;
  bases: number;
  /** Numbers from the last report (out of scanner range). */
  last_report: boolean;
}

export type SpyChance =
  | { type: "steal"; target: number; techs: TechRef[]; framed: boolean; penalty: number }
  | { type: "sabotage"; target: number; strength: number; framed: boolean; penalty: number };

export type SabotageKind = "factories" | "bases" | "revolt";

export interface Election {
  candidates: [number, number];
  votes: number[];
  total: number;
  has_voted: boolean[];
  ballots: (number | null)[];
  got: [number, number];
  elected: number | null;
  awaiting_acceptance: boolean;
}

export interface MonsterView {
  kind: "crystal" | "amoeba";
  x: number;
  y: number;
  dest: number;
}

export type Talk =
  | { type: "propose_nap" }
  | { type: "propose_alliance" }
  | { type: "propose_peace" }
  | { type: "declare_war_on"; target: number }
  | { type: "break_alliance_with"; target: number }
  | { type: "propose_trade"; bc: number }
  | { type: "break_treaty" }
  | { type: "break_trade" }
  | { type: "threaten" }
  | { type: "tribute_bc"; bc: number }
  | { type: "tribute_techs" }
  | { type: "tribute_tech"; field: number; tech: number }
  | { type: "tech_offers" }
  | { type: "exchange_tech"; want: TechRef; give: TechRef }
  | { type: "accept_sweetener" }
  | { type: "decline_sweetener" };

export interface Reply {
  kind: number;
  params: MsgParams;
  accepted: boolean;
  sweetener: { bc: number; tech: TechRef | null } | null;
  tech_deals: { want: TechRef; give: TechRef[] }[];
  techs: TechRef[];
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
  security: number;
  tax: number;
  total_trade: number;
  spying_maint: number;
  relations: RelationView[];
  messages: [number, Incoming][];
  spy_chances: [number, SpyChance][];
  /** For each spy chance, the colonies a sabotage can strike. */
  spy_targets: SabotageTarget[][];
  election: Election | null;
  final_war: boolean;
  high_master: number | null;
  victory: "conquest" | "council" | null;
  monsters: MonsterView[];
  /** The battle the turn waits for. */
  battle: Battle | null;
  in_battle: boolean;
  /** The question the turn waits for (crates/core/src/state.rs `Question`). */
  question: Question | null;
}

export type Question =
  | { type: "colonize"; empire: number; planet: number; ship: number; first: boolean }
  | { type: "bomb"; empire: number; planet: number; pop: number; factories: number; bio: number; inbound: number };

// ------------------------------------------------------------- battles (crates/core/src/battle.rs)

export type BattleSide = "l" | "r" | "none";

export interface BattleWpn {
  t: number;
  n: number;
  numfire: number;
  numshots: number;
}

export interface BattleItem {
  uid: number;
  name: string;
  design: number | null;
  look: number;
  complevel: number;
  specials: number[];
  hp1: number;
  hp2: number;
  hull: number;
  man: number;
  defense: number;
  misdefense: number;
  absorb: number;
  repair: number;
  misshield: number;
  pshield: number;
  extrarange: number;
  num: number;
  pulsar: number;
  stream: number;
  stasis: number;
  cloak: number;
  subspace: number;
  warpdis: number;
  blackhole: number;
  technull: number;
  repulsor: number;
  sbmask: number;
  retreat: number;
  sx: number;
  sy: number;
  stasisby: number;
  unman: number;
  side: BattleSide;
  actman: number;
  hploss: number;
  maxrange: number;
  missile: number;
  wpn: BattleWpn[];
}

export interface BattleMissile {
  id: number;
  wpnt: number;
  nummissiles: number;
  x: number;
  y: number;
  source: number;
  target: number;
  fuel: number;
  speed: number;
}

export interface BattleSideInfo {
  /** Empire, or -1 guardian, -2 crystal, -3 amoeba. */
  party: number;
  race: Race | null;
  ships: number[];
  items: number;
  human: boolean;
  auto: boolean;
  base_missile: boolean;
}

export type BattleAnim =
  | { type: "turn"; uid: number }
  | { type: "move"; uid: number; x: number; y: number }
  | { type: "teleport"; uid: number; x: number; y: number }
  | { type: "beam"; from: number; to: number; weapon: number }
  | { type: "bomb"; from: number; to: number; weapon: number }
  | { type: "damage"; uid: number; amount: number; killed: number }
  | { type: "missile_launch"; id: number; from: number; to: number; weapon: number; x: number; y: number }
  | { type: "missile_step"; id: number; x: number; y: number }
  | { type: "missile_gone"; id: number }
  | { type: "missile_shield"; uid: number }
  | { type: "explosion"; x: number; y: number }
  | { type: "destroyed"; uid: number }
  | { type: "retreat"; uid: number }
  | { type: "special"; kind: string; from: number; to: number | null }
  | { type: "repulse"; uid: number; x: number; y: number }
  | { type: "cloak"; uid: number; on: boolean };

export interface Battle {
  planet: number;
  planet_side: BattleSide;
  pop: number;
  fact: number;
  bases: number;
  biodamage: number;
  s: [BattleSideInfo, BattleSideInfo];
  autoresolve: boolean;
  special_button: number;
  num_turn: number;
  cur_item: number;
  item: BattleItem[];
  rocks: [number, number, number][];
  missiles: BattleMissile[];
  /** -100 rock, 1 reachable, 10+i friend, 30+i enemy in range, -i enemy out of range. */
  area: number[][];
  winner: BattleSide | null;
  waiting: boolean;
  started: boolean;
  applied: boolean;
  log: BattleAnim[];
}

export type BattleAction =
  | { type: "cell"; x: number; y: number }
  | { type: "done" }
  | { type: "wait" }
  | { type: "auto" }
  | { type: "missile" }
  | { type: "special" }
  | { type: "retreat" };

export type Command =
  | { type: "answer"; yes: boolean }
  | { type: "rename_planet"; planet: number; name: string }
  | { type: "set_sliders"; planet: number; sliders: number[] }
  | { type: "set_build"; planet: number; design: number }
  | { type: "send_fleet"; from: number; to: number; ships: number[] }
  | { type: "redirect_fleet"; fleet: number; to: number }
  | { type: "redirect_transport"; transport: number; to: number }
  | { type: "send_transport"; from: number; to: number; pop: number }
  | { type: "set_research"; alloc: number[] }
  | { type: "set_research_project"; field: number; tech: number }
  | { type: "lock_slider"; planet: number; slider: number; locked: boolean }
  | { type: "lock_research"; field: number; locked: boolean }
  | { type: "set_rally"; planet: number; to: number | null }
  | { type: "scrap_ships"; planet: number; ships: number[] }
  | { type: "save_design"; slot: number | null; design: ShipDesign }
  | { type: "scrap_design"; design: number }
  | { type: "set_bombard"; enabled: boolean }
  | { type: "set_spying"; target: number; tenths: number }
  | { type: "set_spy_mode"; target: number; mode: SpyMode }
  | { type: "set_security"; tenths: number }
  | { type: "set_tax"; tenths: number }
  | { type: "transfer_reserve"; planet: number; bc: number }
  | { type: "answer_message"; index: number; accept: boolean; pick?: TechRef | null }
  | { type: "steal_tech"; index: number; field: number }
  | { type: "sabotage"; index: number; planet: number; kind: SabotageKind }
  | { type: "vote"; candidate: number | null }
  | { type: "accept_council"; accept: boolean }
  | { type: "start_battle"; mode: "fight" | "auto" | "retreat" }
  | { type: "battle"; action: BattleAction };

export interface NewGameOptions {
  seed: number;
  galaxy_size: GalaxySize;
  difficulty: Difficulty;
  players: number;
  /** 0: Classic (MOO1's computer players, bugs included), 1: Classic+ (1oom's fixes). */
  ai_id?: number;
  race: Race | null;
  banner: number | null;
  star_names?: string[];
  home_names?: string[];
  leader_names?: string[];
  leader_name_lists?: string[][];
  leader?: string | null;
  orion_name?: string | null;
  start_design_names?: string[];
  ship_names?: string[][];
  rules?: unknown;
}
