//! Computer play in tactical battles (1oom `game_ai_classic.c`: `battle_ai_turn`,
//! `battle_ai_retreat` and their helpers), with the Classic+ fixes.

use crate::battle::{absorbdiv, dist, line_plot, opposed, Battle, Side, AREA_H, AREA_W};
use crate::data::WEAPONS;

/// 1oom `game_ai_battle_missile_dmg`.
fn missile_dmg(bt: &Battle, mi: usize) -> i32 {
    let m = &bt.missiles[mi];
    if m.target < 0 {
        return 0;
    }
    let b = &bt.item[m.target as usize];
    let bs = &bt.item[m.source.min(bt.item.len() - 1)];
    let w = &WEAPONS[m.wpnt];
    let mut miss = 50 - (bs.complevel - b.misdefense) * 10;
    if b.cloak == 1 {
        miss += 50;
    }
    miss -= w.extra_acc as i32 * 10;
    miss = miss.clamp(0, 95);
    let damagediv = if m.target == 0 && !w.bomb && w.miss_type > 0 {
        2
    } else {
        1
    };
    let mut damage = w.damage_max as i32 / damagediv;
    damage -= b.absorb;
    damage *= w.damage_mul as i32;
    damage = damage.min(b.hp1);
    damage = damage
        .saturating_mul(m.damagemul2)
        .saturating_mul(m.nummissiles);
    if damage > 0 {
        ((100 - miss) as i64 * damage as i64 / 100) as i32
    } else {
        0
    }
}

/// 1oom `game_ai_battle_incoming_missiles_dmg`: in ships of the target.
fn incoming_missiles_dmg(bt: &Battle, itemi: usize) -> i32 {
    if itemi == 0 {
        return 0;
    }
    let hp = bt.item[itemi].hp1.max(1);
    (0..bt.missiles.len())
        .filter(|&i| bt.missiles[i].target == itemi as i32)
        .map(|i| missile_dmg(bt, i) / hp)
        .sum()
}

/// 1oom `game_ai_battle_dmgmax`: how dangerous a stack is.
fn dmgmax(bt: &Battle, itemi: usize) -> i32 {
    let b = &bt.item[itemi];
    let num_weap = if itemi == 0 { 1 } else { 4 };
    let planet_opp = opposed(b.side, bt.item[0].side);
    let mut v = 0;
    for i in 0..num_weap {
        let wp = b.wpn[i];
        let w = &WEAPONS[wp.t];
        if wp.numshots == 0 {
            continue;
        }
        if w.bomb {
            if planet_opp {
                let d = w.damage_max as i32 - bt.item[0].absorb;
                if d > 0 {
                    v += d * wp.n * w.fire as i32;
                }
                if w.bio {
                    let d = w.damage_max as i32 - bt.antidote;
                    if d > 0 {
                        v += d * wp.n * w.fire as i32 * 50;
                    }
                }
            }
        } else {
            v += w.damage_max as i32 * wp.n * w.fire as i32;
        }
    }
    if b.blackhole == 1 {
        v += 1000;
    }
    v
}

/// 1oom `game_ai_battle_dmggive`: damage `itemi1` can do to `itemi2` (`a2` > 0: ignoring
/// range and shots left).
pub(crate) fn dmggive(bt: &Battle, itemi1: usize, itemi2: usize, a2: i32) -> i32 {
    let b = &bt.item[itemi1];
    let bd = &bt.item[itemi2];
    if bd.stasisby > 0 && (a2 == 0 || a2 == 2) {
        return 0;
    }
    let num_weap = if itemi1 == 0 { 1 } else { 4 };
    let mut miss_beam = 50 - (b.complevel - bd.defense) * 10;
    let mut miss_missile = 50 - (b.complevel - bd.misdefense) * 10;
    if bd.cloak == 1 {
        miss_beam += 50;
        miss_missile += 50;
    }
    miss_beam = miss_beam.clamp(0, 95);
    miss_missile = miss_missile.min(95);
    let d = dist(b.sx, b.sy, bd.sx, bd.sy);
    let mut v = 0;
    let mut damagediv = 1;
    for i in 0..num_weap {
        let wp = b.wpn[i];
        let w = &WEAPONS[wp.t];
        let adiv = absorbdiv(b, wp.t, true);
        if itemi2 == 0 {
            damagediv = if !w.bomb && (w.miss_type > 0 || w.damage_min != w.damage_max) {
                2
            } else {
                1
            };
        }
        let mut range = 0;
        if w.damage_min != w.damage_max && !w.bomb {
            range = b.extrarange;
        } else if b.maxrange < 0 {
            range = b.maxrange;
        }
        if itemi1 == 0 {
            range += 40;
        }
        if (!w.bomb || itemi2 == 0)
            && wp.numshots != 0
            && (w.range as i32 + range >= d || a2 > 0)
            && (wp.numfire > 0 || a2 > 0)
        {
            let mut dmg;
            if w.damage_min == w.damage_max {
                let miss = (miss_missile - w.extra_acc as i32 * 10).clamp(0, 95);
                dmg = ((100 - miss) / 5) * (w.damage_max as i32 - bd.absorb / adiv);
                dmg /= damagediv;
                dmg *= w.damage_mul as i32;
                dmg *= w.streams as i32;
            } else {
                let mut miss = miss_beam;
                if bd.absorb > w.damage_min as i32 {
                    miss += ((100 - miss) * (bd.absorb + 1 - w.damage_min as i32))
                        / (w.damage_max as i32 + 1 - w.damage_min as i32);
                    miss = miss.min(95);
                }
                if (w.damage_max as i32 / damagediv) > (bd.absorb / adiv) {
                    let dmgmax = w.damage_max as i32 / damagediv - bd.absorb / adiv;
                    let dmgmin = (w.damage_min as i32 / damagediv - bd.absorb / adiv).max(1);
                    dmg = (dmgmax + dmgmin) / 2;
                    dmg = ((100 - miss) * dmg) / 5;
                    dmg *= w.damage_mul as i32;
                } else {
                    dmg = 0;
                }
            }
            if w.bio && wp.numshots != 0 {
                dmg = 500 * w.damage_max as i32;
            }
            if dmg <= 0 {
                dmg = i32::from((w.damage_max as i32 / damagediv) > (bd.absorb / adiv));
            }
            v += wp.n * dmg * if a2 > 0 { w.fire as i32 } else { wp.numfire };
        }
    }
    if b.warpdis == 1 && (d <= 3 || a2 > 0) && itemi2 != 0 && bd.unman < bd.man {
        v += 100;
    }
    if b.blackhole == 1 && (d <= 1 || a2 > 0) {
        v += 1000;
    }
    if b.technull == 1 && (d <= 4 || a2 > 0) {
        v += 500;
    }
    if b.pulsar == 1 && bt.special_button == 1 && (d <= 1 || a2 > 0) {
        v += ((5 - bd.absorb) * bd.num).max(0);
    }
    if b.pulsar == 2 && bt.special_button == 1 && (d <= 1 || a2 > 0) {
        v += ((14 - bd.absorb) * bd.num).max(0);
    }
    if b.stream == 1 && (d <= 2 || a2 > 0) {
        v += ((bd.hp1 * (b.num / 2 + 20)) / 100 * bd.num).max(0);
    }
    if b.stream == 2 && (d <= 2 || a2 > 0) {
        v += ((bd.hp1 * (b.num + 40)) / 100 * bd.num).max(0);
    }
    v
}

/// 1oom `game_ai_battle_rival`: the best target for `itemi`, -1 if none.
pub(crate) fn rival(bt: &mut Battle, itemi: usize, a2: i32) -> i32 {
    let mut best = -1;
    let mut maxw = 0;
    for i in 0..bt.item.len() {
        let (bside, bnum) = (bt.item[itemi].side, bt.item[itemi].num);
        let b2 = &bt.item[i];
        if !(opposed(bside, b2.side) && bnum > 0) {
            continue;
        }
        let dmgmissile = incoming_missiles_dmg(bt, i);
        let dmax = dmgmax(bt, i);
        let mut give = dmggive(bt, itemi, i, a2);
        let (b2repair, b2hp1, b2hp2, b2num, b2x, b2y) = {
            let b2 = &bt.item[i];
            (b2.repair, b2.hp1.max(1), b2.hp2, b2.num, b2.sx, b2.sy)
        };
        let repair = b2repair * b2hp2 / 100;
        if itemi == 0 {
            bt.item[0].wpn.swap(0, 1);
            let other = dmggive(bt, itemi, i, a2);
            if other > give {
                give = other;
            } else {
                bt.item[0].wpn.swap(0, 1);
            }
        }
        let mut dmgmany =
            ((bnum as i64 * give as i64 - repair as i64) / (b2hp1 as i64 * 20)) as i32;
        let mut w;
        if dmgmany > 0 {
            let vt = b2num - dmgmissile;
            if vt < dmgmany {
                dmgmany = vt.max(0);
            }
            w = dmgmany.saturating_mul(dmax).saturating_mul(20);
        } else {
            w = if dmgmissile > 0 && give > 0 {
                3
            } else {
                ((give - repair) as i64 * dmax as i64 / b2hp1 as i64).clamp(0, i32::MAX as i64)
                    as i32
            };
            w = w.max(0);
            if give > 0 {
                w = w.max(3);
            }
            if w == 1 && i != 0 {
                w = 2;
            }
        }
        if dmax <= 0 && give > 0 {
            w = if i == 0 { 1 } else { 2 };
        }
        if itemi == 0 && w > 0 {
            let (bx, by) = (bt.item[0].sx, bt.item[0].sy);
            w += (10 - dist(bx, by, b2x, b2y)) * 750;
        }
        if i == 0 && w == 1 {
            for wp in bt.item[itemi].wpn {
                let ww = &WEAPONS[wp.t];
                if ww.bio && wp.numshots != 0 {
                    let g = ww.damage_max as i32 - bt.antidote;
                    if g > 0 {
                        w += g * wp.n * 100;
                    }
                }
            }
        }
        if w > maxw {
            best = i as i32;
            maxw = w;
            bt.bases_using_mirv = WEAPONS[bt.item[0].wpn[0].t].streams > 1;
        }
    }
    if itemi == 0 {
        let t = bt.item[0].wpn[0].t;
        let mirv = WEAPONS[t].streams > 1;
        if bt.bases_using_mirv != mirv {
            bt.item[0].wpn.swap(0, 1);
        }
    }
    best
}

fn missile_none_fired_by(bt: &Battle, itemi: usize) -> bool {
    !bt.missiles.iter().any(|m| m.source == itemi)
}

/// 1oom `game_battle_stasis_target`.
fn stasis_target(bt: &mut Battle) -> i32 {
    let itemi = bt.cur_item;
    let bside = bt.item[itemi].side;
    let (base, num) = if bside == Side::R {
        (1, bt.s[0].items)
    } else {
        (bt.s[0].items + 1, bt.s[1].items)
    };
    let n = (0..num)
        .filter(|&i| base + i < bt.item.len() && bt.item[base + i].stasisby > 0)
        .count();
    if num as i32 - 2 >= n as i32 {
        bt.item[itemi].stasis = 1;
        let mut target = -1;
        let mut vmax = 0;
        for i in 1..bt.item.len() {
            let bd = &bt.item[i];
            if bside != bd.side && bd.stasisby == 0 && bd.cloak != 1 {
                let v = dmgmax(bt, i).saturating_mul(bd.num);
                if v > vmax {
                    vmax = v;
                    target = i as i32;
                }
            }
        }
        target
    } else {
        bt.item[itemi].stasis = 2;
        -1
    }
}

/// 1oom `game_battle_ai_missile_evade`.
fn missile_evade(bt: &mut Battle) -> i32 {
    let itemi = bt.cur_item;
    let b = bt.item[itemi].clone();
    if b.unman == b.man {
        return 0;
    }
    let mut evade = 0;
    for i in 0..bt.missiles.len() {
        if evade > 1 {
            break;
        }
        let m = bt.missiles[i].clone();
        if m.target != itemi as i32 {
            continue;
        }
        let mut sx = m.x / 32;
        let roomx = if b.subspace == 1 {
            (9 - sx).max(sx)
        } else {
            sx -= b.sx;
            if sx < 0 {
                9 - b.sx
            } else {
                b.sx
            }
        };
        let mut movex = (b.man - b.unman) * m.fuel;
        if movex > roomx || b.subspace == 1 {
            movex = roomx;
        }
        let mut d = dist_fast(b.sx * 32 + 16, b.sy * 24 + 12, m.x, m.y);
        let danger = WEAPONS[m.wpnt].dtbl[0] as i32 * m.fuel + 13;
        if d > danger {
            evade = 1;
        }
        d += movex * 32 - 18;
        if (bt.rng_one_to(3) as i32) < bt.difficulty as i32 && d > danger {
            evade = 2;
        }
    }
    evade
}

fn dist_fast(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    if dx > dy {
        dx + dy / 2
    } else {
        dy + dx / 2
    }
}

/// How far the enemy can get away in two turns (stored as a negative `maxrange`).
fn possible_distance_increase(bt: &mut Battle, target_i: usize) {
    let ci = bt.cur_item;
    let bd = &bt.item[target_i];
    let si = if bt.item[ci].sx < bd.sx {
        AREA_W - bd.sx - 1
    } else {
        bd.sx
    };
    let t = (bd.man - bd.unman) * 2;
    bt.item[ci].maxrange = -(si.min(t));
}

/// 1oom `game_battle_ai_best_range`.
fn best_range(bt: &mut Battle, target_i: usize) -> i32 {
    if target_i == 0 {
        return 1;
    }
    let itemi = bt.cur_item;
    let mut bestrange = 1;
    let mut weight1 = 0;
    for i in 1..10 {
        let b = bt.item[itemi].clone();
        let bd = bt.item[target_i].clone();
        let mut weight: i64 = 0;
        if b.blackhole > 0 && i <= 1 {
            weight += 2000;
        }
        if b.stasis == 1 && i <= 1 {
            weight += 2000;
        }
        if b.pulsar == 1 && i <= 1 {
            weight += ((6 - bd.absorb) * bd.num).max(0) as i64;
        }
        if b.pulsar == 2 && i <= 1 {
            weight += ((16 - bd.absorb) * bd.num).max(0) as i64;
        }
        if b.stream == 1 && i <= 2 {
            weight += (((bd.hp1 * (b.num + 20) + 99) / 100) * bd.num).max(0) as i64;
        }
        if b.stream == 2 && i <= 2 {
            weight += (((bd.hp1 * (b.num + 40) + 99) / 100) * bd.num).max(0) as i64;
        }
        for j in 0..4 {
            let wp = b.wpn[j];
            let w = &WEAPONS[wp.t];
            let adiv = if w.damage_min != w.damage_max {
                absorbdiv(&b, wp.t, true)
            } else {
                1
            };
            let mut range = if itemi == 0 { 12 } else { 0 };
            if w.damage_max != w.damage_min && !w.bomb {
                range = b.extrarange;
            } else if !w.bomb {
                possible_distance_increase(bt, target_i);
                range = bt.item[itemi].maxrange;
            }
            if w.range as i32 + range >= i && wp.numshots != 0 && (!w.damage_fade || i == 1) {
                let mut dmg = (w.damage_max as i32 - bd.absorb / adiv)
                    * w.damage_mul as i32
                    * wp.n
                    * w.fire as i32;
                if w.bio {
                    dmg = w.damage_max as i32 * 100;
                }
                weight += dmg.max(0) as i64;
            }
        }
        if i == 1 {
            weight1 = weight;
        }
        if weight == 0 {
            break;
        }
        if b.cloak == 1 {
            if weight1 > weight {
                break;
            }
            bestrange = i;
        } else if (weight1 * 2) / (weight * 3) <= 1 {
            bestrange = i;
        } else {
            break;
        }
    }
    bestrange
}

/// 1oom `eval_pos_for_pulsar_use`.
fn eval_pulsar(bt: &Battle, _sx: i32, _sy: i32) -> i32 {
    let b = &bt.item[bt.cur_item];
    let (ndiv, rbase) = if b.pulsar == 1 { (2, 5) } else { (1, 10) };
    let dmg = rbase + b.num / ndiv;
    let mut w = 0;
    for bd in &bt.item {
        if dist(b.sx, b.sy, bd.sx, bd.sy) == 1 && bd.absorb < dmg {
            w += if b.side == bd.side { -4 } else { 3 };
        }
    }
    w
}

/// 1oom `game_battle_ai_target1_sub3`: how good a cell is to fire at the target from.
fn score_cell(bt: &Battle, sx: i32, sy: i32, target_i: usize, bestrange: i32) -> i32 {
    let itemi = bt.cur_item;
    let b = &bt.item[itemi];
    let bd = &bt.item[target_i];
    let d = dist(sx, sy, bd.sx, bd.sy);
    let line = line_plot(sx, sy, bd.sx, bd.sy);
    let mut si = if d == bestrange { 10 } else { -d * 4 };
    if b.pulsar == 1 || b.pulsar == 2 {
        si += eval_pulsar(bt, sx, sy);
    }
    if bt.line_ok(&line) < 1 {
        si -= 2;
    }
    if bt.area[sy as usize][sx as usize] == itemi as i8 + 10 {
        si += if d == bestrange { 1 } else { -1 };
    }
    si
}

/// 1oom `game_battle_ai_target1_sub4`: get away from the enemy.
fn flee(bt: &mut Battle) {
    let itemi = bt.cur_item;
    let b = bt.item[itemi].clone();
    if b.unman == b.man {
        return;
    }
    if b.subspace == 1 {
        // 1oom keeps the first cell of the farthest group (its distance is to the last enemy).
        let (mut oppdist, mut n, mut dest) = (0, 0, (b.sx, b.sy));
        for sy in 0..AREA_H {
            for sx in 0..AREA_W {
                if bt.area[sy as usize][sx as usize] != 1 {
                    continue;
                }
                let mut d = 0;
                let mut mindist = 10;
                for bd in &bt.item {
                    mindist = 10;
                    if opposed(b.side, bd.side) {
                        d = dist(sx, sy, bd.sx, bd.sy);
                        mindist = mindist.min(d);
                    }
                }
                if oppdist <= mindist {
                    if oppdist < d {
                        n = 0;
                    }
                    oppdist = d;
                    if n == 0 {
                        dest = (sx, sy);
                    }
                    n += 1;
                }
            }
        }
        bt.item_move(itemi, dest.0, dest.1);
        return;
    }
    let mut oppdist = 10;
    for bd in &bt.item {
        if opposed(b.side, bd.side) {
            oppdist = oppdist.min(dist(b.sx, b.sy, bd.sx, bd.sy));
        }
    }
    let mut guard = 0;
    while itemi < bt.item.len() && bt.item[itemi].actman > 0 && guard < 20 {
        guard += 1;
        let (bx, by) = (bt.item[itemi].sx, bt.item[itemi].sy);
        let mut dest = None;
        for sx in bx - 1..=bx + 1 {
            if !(0..AREA_W).contains(&sx) {
                continue;
            }
            for sy in by - 1..by + 1 {
                if !(0..AREA_H).contains(&sy) {
                    continue;
                }
                if bt.area[sy as usize][sx as usize] == 1 {
                    let mut mindist = 10;
                    for bd in &bt.item {
                        if opposed(b.side, bd.side) {
                            mindist = mindist.min(dist(sx, sy, bd.sx, bd.sy));
                        }
                    }
                    if mindist > oppdist {
                        oppdist = mindist;
                        dest = Some((sx, sy));
                    }
                }
            }
        }
        match dest {
            Some((x, y)) => {
                bt.item_move(itemi, x, y);
                if bt.flag_cur_item_destroyed {
                    return;
                }
                bt.area_setup();
            }
            None => bt.item[itemi].actman = 0,
        }
    }
}

/// 1oom `game_battle_ai_target1_sub5`: dodge incoming missiles.
fn dodge(bt: &mut Battle) {
    let itemi = bt.cur_item;
    let b = bt.item[itemi].clone();
    if b.unman == b.man {
        return;
    }
    let near = |bt: &Battle, x: i32, y: i32| -> i32 {
        let mut m = 10;
        for mi in &bt.missiles {
            if mi.target == itemi as i32 && mi.fuel < 8 {
                m = m.min(dist(x, y, mi.x / 32, mi.y / 24));
            }
        }
        m
    };
    if b.subspace == 1 {
        let mut best = None;
        let mut missdist = 0;
        for sy in 0..AREA_H {
            for sx in 0..AREA_W {
                if bt.area[sy as usize][sx as usize] == 1 {
                    let d = near(bt, sx, sy);
                    if missdist < d {
                        best = Some((sx, sy));
                        missdist = d;
                    }
                }
            }
        }
        let (x, y) = best.unwrap_or((b.sx, b.sy));
        bt.item_move(itemi, x, y);
        return;
    }
    let mut guard = 0;
    while itemi < bt.item.len() && bt.item[itemi].actman > 0 && guard < 20 {
        guard += 1;
        let (bx, by) = (bt.item[itemi].sx, bt.item[itemi].sy);
        let mut missdist = near(bt, bx, by);
        let mut dest = None;
        for sx in bx - 1..=bx + 1 {
            if !(0..AREA_W).contains(&sx) {
                continue;
            }
            for sy in by - 1..by + 1 {
                if !(0..AREA_H).contains(&sy) {
                    continue;
                }
                if bt.area[sy as usize][sx as usize] == 1 && (bx != sx || by != sy) {
                    let d = near(bt, sx, sy);
                    if d > missdist {
                        missdist = d;
                        dest = Some((sx, sy));
                    }
                }
            }
        }
        match dest {
            Some((x, y)) => {
                bt.item_move(itemi, x, y);
                if bt.flag_cur_item_destroyed {
                    return;
                }
                bt.area_setup();
            }
            None => bt.item[itemi].actman = 0,
        }
    }
}

/// 1oom `game_battle_ai_target1`: move towards the best firing position.
fn target1(bt: &mut Battle, target_i: i32) -> i32 {
    let itemi = bt.cur_item;
    let bestrange = if target_i >= 0 {
        best_range(bt, target_i as usize)
    } else {
        1
    };
    let b = bt.item[itemi].clone();
    if b.actman > 0 || b.subspace == 1 {
        if target_i != -1 && missile_evade(bt) == 0 {
            let mut vmax = -999;
            let mut cells: Vec<(i32, i32)> = Vec::new();
            for sy in 0..AREA_H {
                for sx in 0..AREA_W {
                    if bt.area[sy as usize][sx as usize] == 1 || (b.sx == sx && b.sy == sy) {
                        let v = score_cell(bt, sx, sy, target_i as usize, bestrange);
                        if v > vmax {
                            cells.clear();
                            cells.push((sx, sy));
                            vmax = v;
                        } else if v == vmax && cells.len() < 20 {
                            cells.push((sx, sy));
                        }
                    }
                }
            }
            let i = if cells.len() < 2 {
                0
            } else {
                bt.rng_below(cells.len() as u16) as usize
            };
            if itemi != 0 {
                if let Some(&(x, y)) = cells.get(i) {
                    if (x, y) != (b.sx, b.sy) {
                        bt.item_move(itemi, x, y);
                    }
                }
            }
        } else if target_i == -1 {
            flee(bt);
        } else {
            dodge(bt);
        }
    }
    bestrange
}

/// 1oom `game_ai_classic_battle_ai_turn`.
pub fn turn(bt: &mut Battle) {
    let itemi = bt.cur_item;
    let mut target_i: i32 = 0;
    bt.area_setup();
    if bt.item[itemi].side == Side::R
        && rival(bt, itemi, 1) == -1
        && missile_none_fired_by(bt, itemi)
    {
        bt.item[itemi].retreat += 1;
    }
    if bt.item[itemi].man - bt.item[itemi].unman == 0 {
        bt.item[itemi].retreat = 0;
    }
    if bt.item[itemi].stasis > 0 {
        target_i = stasis_target(bt);
        if target_i != -1 {
            target1(bt, target_i);
            if bt.flag_cur_item_destroyed {
                return;
            }
            bt.area_setup();
            let t = target_i as usize;
            if t < bt.item.len() {
                let (b, bd) = (&bt.item[itemi], &bt.item[t]);
                if dist(b.sx, b.sy, bd.sx, bd.sy) <= 1 {
                    possible_distance_increase(bt, t);
                    bt.attack(itemi, t, false);
                }
            }
        }
    }
    if bt.flag_cur_item_destroyed || itemi >= bt.item.len() {
        return;
    }
    bt.area_setup();
    if incoming_missiles_dmg(bt, itemi) > 0 && target_i != -1 {
        target_i = rival(bt, itemi, 0);
        if target_i > -1 {
            let b = &bt.item[itemi];
            if (b.pulsar == 1 || b.pulsar == 2) && eval_pulsar(bt, b.sx, b.sy) < 0 {
                bt.special_button = 0;
            }
            possible_distance_increase(bt, target_i as usize);
            bt.attack(itemi, target_i as usize, false);
        }
    }
    if bt.flag_cur_item_destroyed || itemi >= bt.item.len() {
        return;
    }
    if bt.special_button == -1 {
        bt.special_button = 1;
    }
    bt.area_setup();
    target_i = if bt.item[itemi].retreat > 0 {
        -1
    } else {
        rival(bt, itemi, 2)
    };
    let v4 = target1(bt, target_i);
    if bt.flag_cur_item_destroyed || itemi >= bt.item.len() {
        return;
    }
    bt.area_setup();
    {
        let b = &bt.item[itemi];
        if (b.pulsar == 1 || b.pulsar == 2) && eval_pulsar(bt, b.sx, b.sy) < 0 {
            bt.special_button = 0;
        }
    }
    target_i = rival(bt, itemi, 0);
    if target_i == -1 && itemi == 0 && bt.item[0].num > 0 {
        let ii = if bt.item[0].side == Side::R {
            1
        } else {
            bt.s[0].items + 1
        };
        if ii < bt.item.len() && bt.item[ii].side != bt.item[0].side {
            bt.attack(0, ii, false);
        }
    }
    let mut loops = 0;
    while target_i != -1 && itemi < bt.item.len() && !bt.flag_cur_item_destroyed {
        target_i = rival(bt, itemi, 0);
        loops += 1;
        if bt.item[itemi].cloak == 1 && target_i != -1 {
            let (b, bd) = (&bt.item[itemi], &bt.item[target_i as usize]);
            if dist(b.sx, b.sy, bd.sx, bd.sy) != v4 {
                target_i = -1;
            }
        }
        if bt.item[itemi].retreat > 0 {
            target_i = -1;
        }
        if target_i > -1 {
            possible_distance_increase(bt, target_i as usize);
            bt.attack(itemi, target_i as usize, false);
        }
        if loops > 10 {
            break;
        }
    }
    if itemi < bt.item.len() {
        bt.item[itemi].selected = 0;
    }
    bt.turn_done = true;
}

/// 1oom `game_ai_classic_battle_ai_retreat`: does the right side flee?
pub fn retreat(bt: &mut Battle) -> bool {
    let n = bt.item.len();
    let mut missile = vec![0i64; n];
    let mut repair: Vec<i64> = bt
        .item
        .iter()
        .map(|b| (b.hp1 * b.repair / 5) as i64)
        .collect();
    let mut dmg = [0i64; 2];
    let mut hp = [0i64; 2];
    for i in 0..bt.missiles.len() {
        let m = &bt.missiles[i];
        if m.target < 0 || m.target as usize >= n || m.source >= n {
            continue;
        }
        let s = match bt.item[m.target as usize].side {
            Side::R => 1,
            _ => 0,
        };
        missile[m.source] += 2;
        dmg[s] += missile_dmg(bt, i) as i64 / (missile[bt.missiles[i].source] + 1);
        let t = bt.missiles[i].target as usize;
        dmg[s] -= repair[t];
        repair[t] = 0;
    }
    for i in 1..n {
        if i >= bt.item.len() {
            break;
        }
        let s = if i <= bt.s[0].items { 1 } else { 0 };
        let j = rival(bt, i, 1);
        if j >= 0 {
            let b = &bt.item[i];
            hp[s ^ 1] += (b.hp1 * b.num) as i64;
            dmg[s] += dmggive(bt, i, j as usize, 1) as i64 * bt.item[i].num as i64;
            dmg[s] -= repair[j as usize];
            repair[j as usize] = 0;
        } else {
            hp[s ^ 1] += 1;
        }
    }
    if bt.item[0].num > 0 && bt.item[0].side != Side::None {
        let s = if bt.item[0].side == Side::L { 1 } else { 0 };
        let b = &bt.item[0];
        hp[s ^ 1] += (b.hp1 * b.num * 7 / 10) as i64;
        let j = rival(bt, 0, 1);
        if j > 0 {
            dmg[s] += dmggive(bt, 0, j as usize, 1) as i64 * bt.item[0].num as i64 * 7 / 10;
            dmg[s] -= repair[j as usize] * 7 / 10;
        }
    }
    if hp[0] == 0 || hp[1] == 0 {
        return false;
    }
    if dmg[0] == 0 {
        return true;
    }
    let mut v = (5 - bt.difficulty as i64) * 5 + 5;
    match bt.item[0].side {
        Side::L => v += 10,
        Side::R => v += 20,
        Side::None => {}
    }
    v += bt.rng_one_to(15) as i64;
    let wr = dmg[1] * 100 / hp[1];
    let wl = dmg[0] * 100 * v / hp[0];
    wr > wl
}

impl Battle {
    pub(crate) fn rng_one_to(&mut self, n: u16) -> u16 {
        self.rng_mut().one_to(n)
    }
    pub(crate) fn rng_below(&mut self, n: u16) -> u16 {
        self.rng_mut().below(n)
    }
}
