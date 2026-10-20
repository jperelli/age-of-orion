#!/usr/bin/env python3
"""Generates the game data tables of Age of Orion from the 1oom sources (GPLv2).

Usage: tools/gen_1oom_tables.py <path to a 1oom checkout>

Writes:
  crates/core/src/data.rs     ship parts (game_shiptech.c), techs (game_techtypes.h),
                              tech groups (derived from the names), research flags (game_num.c)
  web/public/text/en.json     English names of techs and ship parts (merged into the file)

The data was ported from 1oom commit 42e9c0b. Rerun after changing the generator.
"""
import json
import os
import re
import sys

ROMAN = {"I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XIII", "XV", "XX"}
UPPER = {"ECM", "BC"}


def strip_comments(s):
    return re.sub(r"/\*.*?\*/", "", s, flags=re.S)


def title(words):
    out = []
    for w in words:
        if w in ROMAN or w in UPPER:
            out.append(w)
        elif "-" in w:
            out.append("-".join(p.capitalize() if p not in ROMAN else p for p in w.split("-")))
        else:
            out.append(w.capitalize())
    return " ".join(out)


def tech_name(ident):
    if ident.startswith("IMPROVED_TERRAFORMING__"):
        return "Improved Terraforming +" + ident.rsplit("_", 1)[1]
    m = re.match(r"(REDUCED_INDUSTRIAL_WASTE)_(\d+)$", ident)
    if m:
        return "Reduced Industrial Waste " + m.group(2) + "%"
    return title(ident.split("_"))


GROUP_RULES = [
    (r"^ecm_jammer_", 1), (r"^battle_computer_", 2), (r"^improved_robotic_controls_", 3),
    (r"_space_scanner$", 4), (r"^reduced_industrial_waste_|^industrial_waste_elimination$", 5),
    (r"^improved_industrial_tech_", 6),
    (r"^(duralloy|zortium|andrium|tritanium|adamantium|neutronium)_armor$", 7),
    (r"^automated_repair_system$|^advanced_damage_control$", 8), (r"_deflector_shields$", 9),
    (r"_planetary_shield$", 10), (r"^zyro_shield$|^lightning_shield$|^anti_missile_rockets$", 11),
    (r"^improved_terraforming_|^complete_terraforming$", 12), (r"^controlled_.*_environment$", 13),
    (r"eco_restoration$|^ecological_restoration$", 14),
    (r"^battle_suits$|^armored_exoskeleton$|^powered_armor$", 15), (r"^personal_.*_shield$", 16),
    (r"_engines$|_drives$", 17), (r"_fuel_cells$|_crystals$|^thorium_cells$", 18),
    (r"^(hand_lasers|ion_rifle|fusion_rifle|hand_phasor|plasma_rifle)$", 21),
    (r"_bomb$", 22), (r"^(death_spores|doom_virus|bio_terminator)$", 23), (r"_pulsar$", 24),
    (r"_stream_projector$", 25), (r"_rockets$|_missiles$|_torpedoes$", 20),
    (r"^(lasers|gatling_laser|neutron_pellet_gun|ion_cannon|mass_driver|neutron_blaster|"
     r"graviton_beam|hard_beam|fusion_beam|megabolt_cannon|phasor|auto_blaster|tachyon_beam|"
     r"gauss_autocannon|particle_beam|plasma_cannon|disruptor|pulse_phasor|"
     r"tri_focus_plasma_cannon|mauler_device|stellar_converter)$", 19),
]


def tech_group(key):
    for pat, g in GROUP_RULES:
        if re.search(pat, key):
            return g
    return 0


def parse_enums(src):
    """enum name -> {identifier: value} for the tech enums of game_techtypes.h."""
    enums = {}
    for body, name in re.findall(r"typedef enum \{(.*?)\}\s*(\w+);", src, flags=re.S):
        vals, v = {}, -1
        for item in strip_comments(body).split(","):
            item = item.strip()
            if not item:
                continue
            m = re.match(r"(\w+)\s*=\s*(\d+)", item)
            if m:
                ident, v = m.group(1), int(m.group(2))
            else:
                ident, v = item, v + 1
            vals[ident] = v
        enums[name] = vals
    return enums


def table(src, name):
    m = re.search(r"%s(?:\[[^\]]*\])+\s*=\s*\{(.*?)\n\};" % re.escape(name), src, flags=re.S)
    return strip_comments(m.group(1))


def entries(body):
    """Splits a C initializer list of structs into the token lists of each struct."""
    out, depth, cur = [], 0, ""
    for ch in body:
        if ch == "{":
            depth += 1
            if depth == 1:
                cur = ""
                continue
        if ch == "}":
            depth -= 1
            if depth == 0:
                out.append(cur)
                continue
        if depth >= 1:
            cur += ch
    return out


def tokens(entry):
    """Flat list of values; nested {a, b, c} become Python lists."""
    vals, i = [], 0
    s = entry.replace("\n", " ")
    while i < len(s):
        c = s[i]
        if c == "{":
            j = s.index("}", i)
            vals.append([t.strip() for t in s[i + 1:j].split(",") if t.strip()])
            i = j + 1
        elif c in ", \t":
            i += 1
        else:
            j = i
            depth = 0
            while j < len(s) and (depth > 0 or s[j] != ","):
                depth += {"(": 1, ")": -1}.get(s[j], 0)
                j += 1
            vals.append(s[i:j].strip())
            i = j
    return vals


def strings(src, name):
    m = re.search(r"%s\[[^\]]*\]\s*=\s*\{(.*?)\};" % re.escape(name), src, flags=re.S)
    return re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1))


def main(root):
    game = os.path.join(root, "src", "game")
    read = lambda f: open(os.path.join(game, f), encoding="latin-1").read()
    techtypes, shiptech, shiptech_h, strsrc, num = (
        read("game_techtypes.h"), read("game_shiptech.c"), read("game_shiptech.h"),
        read("game_str.c"), read("game_num.c"))
    enums = parse_enums(techtypes)
    symbols = {}
    for vals in enums.values():
        symbols.update(vals)
    for vals in parse_enums(shiptech_h).values():
        symbols.update(vals)
    symbols.update({"TECH_FIELD_COMPUTER": 0, "TECH_FIELD_CONSTRUCTION": 1, "TECH_FIELD_FORCE_FIELD": 2,
                    "TECH_FIELD_PLANETOLOGY": 3, "TECH_FIELD_PROPULSION": 4, "TECH_FIELD_WEAPON": 5,
                    "false": 0, "true": 1})
    shipenums = parse_enums(shiptech_h)

    def val(t):
        t = t.strip()
        if re.match(r"^-?(0x[0-9a-fA-F]+|\d+)$", t):
            return int(t, 0)
        m = re.match(r"\(1 << (\w+)\)", t)
        if m:
            return 1 << symbols[m.group(1)]
        return symbols[t]

    def keys(enum, prefix):
        return [k[len(prefix):].lower() for k, _ in sorted(shipenums[enum].items(), key=lambda kv: kv[1])
                if not k.endswith("_NUM")]

    text = {}
    rs = ["// Generated by tools/gen_1oom_tables.py from the 1oom sources (GPLv2). Do not edit.",
          "// Ship parts: 1oom src/game/game_shiptech.c; techs: game_techtypes.h; research flags: game_num.c.",
          "#![allow(clippy::all)]", "", "use super::data_types::*;", ""]

    # --- techs
    fields = [("tech_comp_t", "TECH_COMP_", "computer"), ("tech_cons_t", "TECH_CONS_", "construction"),
              ("tech_ffld_t", "TECH_FFLD_", "force_field"), ("tech_plan_t", "TECH_PLAN_", "planetology"),
              ("tech_prop_t", "TECH_PROP_", "propulsion"), ("tech_weap_t", "TECH_WEAP_", "weapon")]
    rs.append("/// Tech keys per field, indexed by tech number (1..=50); `\"\"` = no tech with that number.")
    rs.append("pub const TECHS: [[&str; 51]; 6] = [")
    for enum, prefix, fname in fields:
        names = [""] * 51
        for ident, v in enums[enum].items():
            short = ident[len(prefix):]
            if v == 0 or short.startswith("UNUSED"):
                continue
            key = short.lower()
            names[v] = key
            text["tech.%s.%s" % (fname, key)] = tech_name(short)
        rs.append("    [%s]," % ", ".join('"%s"' % n for n in names))
    rs.append("];")
    rs.append("")

    # Tech groups and tiers (1oom tech_group_t). MOO1 keeps them in RESEARCH.LBX; here they
    # are derived from the tech names: techs of one series form a group, ranked by number.
    rs.append("/// Group (1oom `tech_group_t`) and tier of each tech, per field and tech number.")
    rs.append("pub const TECH_GROUPS: [[[u8; 2]; 51]; 6] = [")
    for enum, prefix, fname in fields:
        groups = [[0, 0] for _ in range(51)]
        counter = {}
        for ident, v in sorted(enums[enum].items(), key=lambda kv: kv[1]):
            short = ident[len(prefix):]
            if v == 0 or short.startswith("UNUSED"):
                continue
            g = tech_group(short.lower())
            counter[g] = counter.get(g, 0) + 1
            groups[v] = [g, counter[g] if g else 1]
        rs.append("    [%s]," % ", ".join("[%d, %d]" % tuple(x) for x in groups))
    rs.append("];")
    rs.append("")

    # research flags (1oom game_num_ng_tech): bit 0 never, bit 1 always, bit 2 essential
    macros = dict(re.findall(r"#define (GAME_NUMT_\w+) (\{[^}]*\})", num))
    body = strip_comments(table(num, "game_num_ng_tech"))
    for k, v in macros.items():
        body = body.replace(k, v)
    nums = [int(x) for x in re.findall(r"\d+", body)]
    assert len(nums) == 10 * 6 * 51, len(nums)
    rs.append("/// 1oom `game_num_ng_tech`: per race and field, flags per tech number (1 never, 2 always, 4 essential).")
    rs.append("pub const RESEARCH_FLAGS: [[[u8; 51]; 6]; 10] = [")
    for r in range(10):
        rs.append("    [")
        for f in range(6):
            row = nums[(r * 6 + f) * 51:(r * 6 + f + 1) * 51]
            rs.append("        [%s]," % ", ".join(map(str, row)))
        rs.append("    ],")
    rs.append("];")
    rs.append("")

    # --- weapons
    wkeys = keys("weapon_t", "WEAPON_")
    wnames = ["None"] + [title(n.split()) for n in strings(strsrc, "game_str_tbl_st_weap")]
    wextra = [""] + [x.strip().capitalize() for x in strings(strsrc, "game_str_tbl_st_weapx")]
    rs.append("pub const WEAPONS: [Weapon; %d] = [" % len(wkeys))
    for i, e in enumerate(entries(table(shiptech, "tbl_shiptech_weap"))):
        t = tokens(e)
        (_, _, dmin, dmax, rng, acc, halve, bomb, fade, miss, mul, nfire, nshots, cost, space, power,
         bio, tech, _v24, _dtbl, _snd, nmiss) = t
        rs.append("    Weapon { key: \"%s\", damage_min: %d, damage_max: %d, range: %d, extra_acc: %d, "
                  "halve_shield: %s, bomb: %s, damage_fade: %s, miss_type: %d, damage_mul: %d, "
                  "fire: %d, shots: %d, cost: %d, space: %d, power: %d, bio: %s, tech: %d, streams: %d },"
                  % (wkeys[i], val(dmin), val(dmax), val(rng), val(acc), str(bool(val(halve))).lower(),
                     str(bool(val(bomb))).lower(), str(bool(val(fade))).lower(), val(miss), val(mul),
                     val(nfire), val(nshots), val(cost), val(space), val(power), str(bool(val(bio))).lower(),
                     val(tech), val(nmiss)))
        if i:
            text["part.weapon.%s" % wkeys[i]] = wnames[i]
            if wextra[i]:
                text["part.weapon.%s.info" % wkeys[i]] = wextra[i]
    rs.append("];")
    rs.append("")

    def per_hull(name, struct, enum, prefix, fmt, strname, none_first=True):
        ks = keys(enum, prefix)
        names = strings(strsrc, strname)
        if none_first:
            names = ["None"] + names
        rs.append("pub const %s: [%s; %d] = [" % (name, struct, len(ks)))
        for i, e in enumerate(entries(table(shiptech, "tbl_shiptech_" + name.lower()[:-1] if name != "ARMOR" else "tbl_shiptech_armor"))):
            rs.append("    " + fmt(ks[i], tokens(e)) + ",")
            if i or not none_first:
                text["part.%s.%s" % (name.lower().rstrip("s"), ks[i])] = title(names[i].split())
        rs.append("];")
        rs.append("")

    a4 = lambda l: "[%s]" % ", ".join(str(val(x)) for x in l)
    per_hull("COMPS", "Comp", "ship_comp_t", "SHIP_COMP_",
             lambda k, t: "Comp { key: \"%s\", power: %s, space: %s, cost: %s, tech: %d, level: %d }"
             % (k, a4(t[1]), a4(t[2]), a4(t[3]), val(t[4]), val(t[5])), "game_str_tbl_st_comp")
    per_hull("JAMMERS", "Comp", "ship_jammer_t", "SHIP_JAMMER_",
             lambda k, t: "Comp { key: \"%s\", power: %s, space: %s, cost: %s, tech: %d, level: %d }"
             % (k, a4(t[1]), a4(t[2]), a4(t[3]), val(t[4]), val(t[5])), "game_str_tbl_st_jammer")
    per_hull("ENGINES", "Engine", "ship_engine_t", "SHIP_ENGINE_",
             lambda k, t: "Engine { key: \"%s\", power: %d, space: %d, cost: %d, warp: %d, tech: %d }"
             % (k, val(t[1]), val(t[2]), val(t[3]), val(t[4]), val(t[5])), "game_str_tbl_st_engine", False)
    per_hull("ARMOR", "Armor", "ship_armor_t", "SHIP_ARMOR_",
             lambda k, t: "Armor { key: \"%s\", cost: %s, space: %s, armor: %d, tech: %d }"
             % (k, a4(t[1]), a4(t[2]), val(t[3]), val(t[4])), "game_str_tbl_st_armor", False)
    per_hull("SHIELDS", "Shield", "ship_shield_t", "SHIP_SHIELD_",
             lambda k, t: "Shield { key: \"%s\", cost: %s, space: %s, power: %s, absorb: %d, tech: %d }"
             % (k, a4(t[1]), a4(t[2]), a4(t[3]), val(t[4]), val(t[5])), "game_str_tbl_st_shield")

    skeys = keys("ship_special_t", "SHIP_SPECIAL_")
    snames = ["None"] + [title(n.split()) for n in strings(strsrc, "game_str_tbl_st_special")]
    sextra = [""] + [x.strip().capitalize() for x in strings(strsrc, "game_str_tbl_st_specialx")]
    rs.append("pub const SPECIALS: [Special; %d] = [" % len(skeys))
    for i, e in enumerate(entries(table(shiptech, "tbl_shiptech_special"))):
        t = tokens(e)
        t += ["0"] * (15 - len(t))  # the NONE entry omits trailing fields
        (_, _, cost, space, power, tech, field, kind, repair, extraman, misshield, extrarange, pulsar,
         stream, mask) = t
        rs.append("    Special { key: \"%s\", cost: %s, space: %s, power: %s, tech: %d, field: %d, kind: %d, "
                  "repair: %d, extra_man: %d, missile_shield: %d, extra_range: %d, pulsar: %d, stream: %d, "
                  "flags: %d },"
                  % (skeys[i], a4(cost), a4(space), a4(power), val(tech), val(field), val(kind), val(repair),
                     val(extraman), val(misshield), val(extrarange), val(pulsar), val(stream), val(mask)))
        if i:
            text["part.special.%s" % skeys[i]] = snames[i]
            text["part.special.%s.info" % skeys[i]] = sextra[i]
    rs.append("];")
    rs.append("")

    hkeys = keys("ship_hull_t", "SHIP_HULL_")
    rs.append("pub const HULLS: [Hull; %d] = [" % len(hkeys))
    for i, e in enumerate(entries(table(shiptech, "tbl_shiptech_hull"))):
        t = tokens(e)
        rs.append("    Hull { key: \"%s\", cost: %d, space: %d, hits: %d, power: %d, defense: %d },"
                  % (hkeys[i], val(t[1]), val(t[2]), val(t[3]), val(t[4]), val(t[5])))
        text["part.hull.%s" % hkeys[i]] = hkeys[i].capitalize()
    rs.append("];")

    here = os.path.dirname(os.path.abspath(__file__))
    out = os.path.join(here, "..", "crates", "core", "src", "data.rs")
    with open(out, "w") as f:
        f.write("\n".join(rs) + "\n")
    en_path = os.path.join(here, "..", "web", "public", "text", "en.json")
    en = json.load(open(en_path))
    en.update(text)
    with open(en_path, "w") as f:
        json.dump(en, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print("wrote %s and %d texts" % (os.path.relpath(out), len(text)))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
