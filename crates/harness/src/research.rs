//! A RESEARCH.LBX for 1oom built from the tables of Age of Orion (tech names, groups and
//! tiers derived in `tools/gen_1oom_tables.py`), for players who did not import theirs.
//! Layout: 1oom `doc/format_lbx.txt`.

use aoo_core::data::{TECHS, TECH_GROUPS};

fn table(num: u16, size: u16, data: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(data.len() + 4);
    v.extend(num.to_le_bytes());
    v.extend(size.to_le_bytes());
    v.extend(data);
    v
}

fn lbx(kind: u16, entries: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend((entries.len() as u16).to_le_bytes());
    out.extend(0xfeadu16.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out.extend(kind.to_le_bytes());
    let mut off = 0x200u32;
    out.extend(off.to_le_bytes());
    for e in entries {
        off += e.len() as u32;
        out.extend(off.to_le_bytes());
    }
    out.resize(0x200, 0);
    for e in entries {
        out.extend(e);
    }
    out
}

pub fn research_lbx() -> Vec<u8> {
    let mut d0 = Vec::new();
    let mut names = Vec::new();
    for f in 0..6 {
        for t in 1..=50 {
            let [g, tier] = TECH_GROUPS[f][t];
            let name = TECHS[f][t];
            let exists = !name.is_empty() && tier > 0;
            let offs = names.len() as u16;
            d0.push(if exists { g } else { 0xff });
            d0.push(tier);
            d0.extend(0u16.to_le_bytes());
            d0.extend(offs.to_le_bytes());
            names.extend(name.to_ascii_uppercase().replace('_', " ").as_bytes());
            names.push(0);
        }
    }
    names.resize(0xf3c, 0);
    let descr = vec![0u8; 300 * 0xc3];
    lbx(
        5,
        &[
            table(300, 6, &d0),
            table(1, 0xf3c, &names),
            table(300, 0xc3, &descr),
        ],
    )
}
