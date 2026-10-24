// What a planet's values and an empire's race mean in the rules, for the popups of the side
// panel and the planet window. The numbers follow the core (1oom): planet sizes from
// `game_generate_planets`, specials from `game_adjust_prod_by_special` and
// `game_get_tech_prod`, environments from `game_get_pop_growth_max`, race research costs from
// `Rules::race_tech_cost`.
import { defaultRules } from "./engine";
import type { Theme } from "./theme";
import { FIELDS, PLANET_TYPES, RACES, type Growth, type PlanetType, type Race, type Special, type StarType } from "./types";

/** Usual population of each kind of planet (before the occasional ±20). */
const KIND_POP: Partial<Record<PlanetType, [number, number]>> = {
  radiated: [10, 40],
  toxic: [10, 40],
  inferno: [10, 40],
  dead: [20, 50],
  tundra: [20, 50],
  barren: [30, 50],
  minimal: [30, 50],
  desert: [35, 50],
  steppe: [45, 60],
  arid: [55, 70],
  ocean: [65, 80],
  jungle: [75, 90],
  terran: [85, 100],
};

/** The tech that lets colony ships settle each hostile kind. */
const KIND_TECH: Partial<Record<PlanetType, string>> = {
  barren: "controlled_barren_environment",
  tundra: "controlled_tundra_environment",
  dead: "controlled_dead_environment",
  inferno: "controlled_inferno_environment",
  toxic: "controlled_toxic_environment",
  radiated: "controlled_radiated_environment",
};

const tech = (theme: Theme, key: string) => theme.t(`tech.planetology.${key}`);

export function kindLines(theme: Theme, kind: PlanetType): string[] {
  if (kind === "not_habitable") return [theme.t("explain.kind.none")];
  const [min, max] = KIND_POP[kind] ?? [0, 0];
  const lines = [theme.t("explain.kind.pop", { min, max })];
  const needs = KIND_TECH[kind];
  lines.push(needs ? theme.t("explain.kind.needs", { tech: tech(theme, needs) }) : theme.t("explain.kind.basic"));
  if (PLANET_TYPES.indexOf(kind) < PLANET_TYPES.indexOf("minimal")) lines.push(theme.t("explain.kind.hostile"));
  return lines;
}

export function specialLines(theme: Theme, special: Special): string[] {
  return [theme.t(`explain.special.${special}`)];
}

export function growthLines(theme: Theme, growth: Growth, race: Race): string[] {
  const next: Record<Growth, string> = { hostile: "atmospheric_terraforming", normal: "soil_enrichment", fertile: "advanced_soil_enrichment", gaia: "" };
  const lines = [theme.t(`explain.growth.${growth}`, { tech: next[growth] ? tech(theme, next[growth]) : "" })];
  if (race === "silicoid") lines.push(theme.t("explain.growth.silicoid", { race: theme.t("race.silicoid.plural") }));
  return lines;
}

export function starLines(theme: Theme, star: StarType): string[] {
  const k = (x: PlanetType) => theme.t(`planet.${x}`);
  return [theme.t(`explain.star.${star}`, { ocean: k("ocean"), jungle: k("jungle"), terran: k("terran"), arid: k("arid") }), theme.t("explain.star.after")];
}

/** What a race does differently: research costs by field, then its abilities. */
export function raceLines(theme: Theme, race: Race, modRules?: unknown): string[] {
  const rules = { ...defaultRules(), ...((modRules as Record<string, unknown> | undefined) ?? {}) };
  const costs = (rules.race_tech_cost as number[][] | undefined)?.[RACES.indexOf(race)] ?? [];
  const lines: string[] = [];
  costs.forEach((c, f) => {
    if (c < 100) lines.push(theme.t("explain.race.cheaper", { field: theme.t(`field.${FIELDS[f]}`), pct: 100 - c }));
    else if (c > 100) lines.push(theme.t("explain.race.dearer", { field: theme.t(`field.${FIELDS[f]}`), pct: c - 100 }));
  });
  for (let i = 1; theme.has(`explain.race.${race}.${i}`); i++) lines.push(theme.t(`explain.race.${race}.${i}`));
  return lines;
}
