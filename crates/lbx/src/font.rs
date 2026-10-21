//! MOO1 bitmap fonts (FONTS.LBX entry 0, read as 1oom `lbxfont.c` does) converted to
//! TrueType fonts whose glyphs are made of square pixels, so the browser can use them for
//! any text.

use serde::Serialize;

/// One MOO1 font: 96 glyphs for the characters 0x20..=0x7f.
#[derive(Debug, Clone)]
pub struct BitmapFont {
    pub height: usize,
    /// Pixels between characters.
    pub gap_w: usize,
    /// Pixels between lines.
    pub gap_h: usize,
    /// Per glyph, its columns of `height` pixels.
    pub glyphs: Vec<Vec<Vec<bool>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontInfo {
    pub index: usize,
    pub path: String,
    /// Height of the glyph cell in pixels: use a CSS font size of a multiple of it.
    pub height: usize,
    pub gap_h: usize,
}

fn le16(d: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_le_bytes([*d.get(at)?, *d.get(at + 1)?]) as usize)
}

/// Fonts of FONTS.LBX entry 0 (1oom `lbxfont_select`: heights at 0x16a, gaps at 0x17a and
/// 0x18a, widths at 0x19a, glyph offsets at 0x49a). Glyphs are column-wise runs: a byte
/// below 0x80 is `(count << 4) | colour`, `0x80 | n` skips n pixels, 0x80 ends the column.
pub fn parse(d: &[u8]) -> Vec<BitmapFont> {
    let mut out = Vec::new();
    for f in 0..8 {
        let (Some(height), Some(gap_w), Some(gap_h)) = (
            le16(d, 0x16a + f * 2),
            le16(d, 0x17a + f * 2),
            le16(d, 0x18a + f * 2),
        ) else {
            break;
        };
        if height == 0 || height > 64 {
            continue;
        }
        let mut glyphs = Vec::with_capacity(96);
        for c in 0..96 {
            let w = d.get(0x19a + f * 0x60 + c).copied().unwrap_or(0) as usize;
            let mut p = le16(d, 0x49a + f * 0xc0 + c * 2).unwrap_or(0);
            let mut cols = Vec::with_capacity(w);
            for _ in 0..w.min(64) {
                let mut col = vec![false; height];
                let mut y = 0;
                while let Some(&b) = d.get(p) {
                    p += 1;
                    if b & 0x80 == 0 {
                        for _ in 0..(b >> 4) {
                            if y < height {
                                col[y] = true;
                            }
                            y += 1;
                        }
                    } else if b & 0x7f != 0 {
                        y += (b & 0x7f) as usize;
                    } else {
                        break;
                    }
                }
                cols.push(col);
            }
            glyphs.push(cols);
        }
        out.push(BitmapFont {
            height,
            gap_w,
            gap_h,
            glyphs,
        });
    }
    out
}

/// Font units per pixel.
const PX: i32 = 100;

struct Glyph {
    advance: u16,
    contours: Vec<[(i16, i16); 4]>,
}

impl Glyph {
    fn bbox(&self) -> (i16, i16, i16, i16) {
        let pts = self.contours.iter().flatten();
        let xmin = pts.clone().map(|p| p.0).min().unwrap_or(0);
        let ymin = pts.clone().map(|p| p.1).min().unwrap_or(0);
        let xmax = pts.clone().map(|p| p.0).max().unwrap_or(0);
        let ymax = pts.map(|p| p.1).max().unwrap_or(0);
        (xmin, ymin, xmax, ymax)
    }
}

fn be16(v: &mut Vec<u8>, x: u16) {
    v.extend(x.to_be_bytes());
}

fn be32(v: &mut Vec<u8>, x: u32) {
    v.extend(x.to_be_bytes());
}

fn checksum(d: &[u8]) -> u32 {
    d.chunks(4).fold(0u32, |s, c| {
        let mut b = [0u8; 4];
        b[..c.len()].copy_from_slice(c);
        s.wrapping_add(u32::from_be_bytes(b))
    })
}

/// Rows of the cell above the baseline: the bottom of "x".
fn baseline(f: &BitmapFont) -> usize {
    let x = &f.glyphs[(b'x' - 0x20) as usize];
    x.iter()
        .filter_map(|col| col.iter().rposition(|&p| p))
        .max()
        .map_or(f.height, |r| r + 1)
}

/// Top row with ink in a glyph.
fn top(cols: &[Vec<bool>]) -> Option<usize> {
    cols.iter().filter_map(|c| c.iter().position(|&p| p)).min()
}

/// The Spanish letters MOO1's fonts lack (á é í ó ú ü ñ, the capitals, ¡ ¿), composed
/// from the base glyphs with accents drawn above them.
fn spanish(f: &BitmapFont) -> Vec<(u32, Vec<Vec<bool>>)> {
    let g = |c: char| f.glyphs[(c as usize).saturating_sub(0x20).min(95)].clone();
    let mut out = Vec::new();
    let mark = |mut cols: Vec<Vec<bool>>, kind: char| -> Vec<Vec<bool>> {
        if cols.is_empty() {
            return cols;
        }
        let w = cols.len();
        let t = top(&cols).unwrap_or(0);
        // Row of the accent: two rows above the letter when there is room.
        let r = t.saturating_sub(2);
        let r2 = t.saturating_sub(3);
        let mut set = |x: usize, y: usize| {
            if x < w && y < f.height {
                cols[x][y] = true;
            }
        };
        let cx = w / 2;
        match kind {
            '\'' => {
                if t >= 3 {
                    set(cx, r);
                    set(cx + 1, r2);
                } else {
                    set(cx, r);
                }
            }
            '"' => {
                set(cx.saturating_sub(1), r);
                set((cx + 1).min(w - 1), r);
            }
            _ => {
                for x in 0..w {
                    set(
                        x,
                        if x % 2 == 1 {
                            r2.max(r.saturating_sub(1))
                        } else {
                            r
                        },
                    );
                }
            }
        }
        cols
    };
    // A dotless i: the stem without the dot above the first gap.
    let mut dotless = g('i');
    for col in dotless.iter_mut() {
        if let Some(first) = col.iter().position(|&p| p) {
            if let Some(gap) = col[first..].iter().position(|&p| !p) {
                if col[first + gap..].iter().any(|&p| p) {
                    for p in col.iter_mut().take(first + gap) {
                        *p = false;
                    }
                }
            }
        }
    }
    let flip = |cols: Vec<Vec<bool>>| -> Vec<Vec<bool>> {
        cols.into_iter()
            .map(|c| c.into_iter().rev().collect())
            .collect()
    };
    for (code, base, kind) in [
        (0xe1, g('a'), '\''),
        (0xe9, g('e'), '\''),
        (0xed, dotless, '\''),
        (0xf3, g('o'), '\''),
        (0xfa, g('u'), '\''),
        (0xfc, g('u'), '"'),
        (0xf1, g('n'), '~'),
        (0xc1, g('A'), '\''),
        (0xc9, g('E'), '\''),
        (0xcd, g('I'), '\''),
        (0xd3, g('O'), '\''),
        (0xda, g('U'), '\''),
        (0xdc, g('U'), '"'),
        (0xd1, g('N'), '~'),
    ] {
        out.push((code, mark(base, kind)));
    }
    out.push((0xa1, flip(g('!'))));
    out.push((0xbf, flip(g('?'))));
    out
}

/// A TrueType font with the glyphs of `f` mapped to U+0020..U+007E, plus the Spanish
/// letters.
pub fn to_ttf(f: &BitmapFont, family: &str) -> Vec<u8> {
    let base = baseline(f) as i32;
    let descent = (f.height as i32 - base) * PX;
    let ascent = base * PX;
    let mut glyphs = vec![Glyph {
        advance: (f.gap_w as i32 * PX + 3 * PX) as u16,
        contours: Vec::new(),
    }];
    let mut all: Vec<(u32, Vec<Vec<bool>>)> = (0..95)
        .map(|c| (0x20 + c as u32, f.glyphs[c].clone()))
        .collect();
    all.extend(spanish(f));
    for (_, cols) in &all {
        let mut contours = Vec::new();
        for (x, col) in cols.iter().enumerate() {
            let mut y = 0;
            while y < col.len() {
                if !col[y] {
                    y += 1;
                    continue;
                }
                let y0 = y;
                while y < col.len() && col[y] {
                    y += 1;
                }
                let (x0, x1) = ((x as i32 * PX) as i16, ((x as i32 + 1) * PX) as i16);
                let top = ((base - y0 as i32) * PX) as i16;
                let bottom = ((base - y as i32) * PX) as i16;
                contours.push([(x0, bottom), (x0, top), (x1, top), (x1, bottom)]);
            }
        }
        glyphs.push(Glyph {
            advance: ((cols.len() + f.gap_w) as i32 * PX) as u16,
            contours,
        });
    }
    let n = glyphs.len() as u16;
    // glyf + loca
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    let (mut fx0, mut fy0, mut fx1, mut fy1) = (i16::MAX, i16::MAX, i16::MIN, i16::MIN);
    let (mut max_pts, mut max_cont) = (0u16, 0u16);
    let mut hmtx = Vec::new();
    let mut min_rsb = i16::MAX;
    let mut max_extent = 0i16;
    for g in &glyphs {
        be32(&mut loca, glyf.len() as u32);
        let (x0, y0, x1, y1) = g.bbox();
        be16(&mut hmtx, g.advance);
        be16(&mut hmtx, x0 as u16);
        if g.contours.is_empty() {
            continue;
        }
        fx0 = fx0.min(x0);
        fy0 = fy0.min(y0);
        fx1 = fx1.max(x1);
        fy1 = fy1.max(y1);
        min_rsb = min_rsb.min(g.advance as i16 - x1);
        max_extent = max_extent.max(x1);
        max_pts = max_pts.max(g.contours.len() as u16 * 4);
        max_cont = max_cont.max(g.contours.len() as u16);
        be16(&mut glyf, g.contours.len() as u16);
        for v in [x0, y0, x1, y1] {
            be16(&mut glyf, v as u16);
        }
        for i in 0..g.contours.len() {
            be16(&mut glyf, (i * 4 + 3) as u16);
        }
        be16(&mut glyf, 0);
        glyf.extend(std::iter::repeat_n(1u8, g.contours.len() * 4));
        let (mut px, mut py) = (0i16, 0i16);
        let pts: Vec<(i16, i16)> = g.contours.iter().flatten().copied().collect();
        for &(x, _) in &pts {
            be16(&mut glyf, (x - px) as u16);
            px = x;
        }
        for &(_, y) in &pts {
            be16(&mut glyf, (y - py) as u16);
            py = y;
        }
        while glyf.len() % 4 != 0 {
            glyf.push(0);
        }
    }
    be32(&mut loca, glyf.len() as u32);
    if fx0 > fx1 {
        (fx0, fy0, fx1, fy1) = (0, 0, 0, 0);
    }
    let upm = (ascent + descent) as u16;
    let adv_max = glyphs.iter().map(|g| g.advance).max().unwrap_or(0);
    let avg = (glyphs.iter().map(|g| g.advance as u32).sum::<u32>() / n as u32) as u16;
    // head
    let mut head = Vec::new();
    be32(&mut head, 0x0001_0000);
    be32(&mut head, 0x0001_0000);
    be32(&mut head, 0);
    be32(&mut head, 0x5F0F_3CF5);
    be16(&mut head, 0x000B);
    be16(&mut head, upm);
    head.extend([0u8; 16]);
    for v in [fx0, fy0, fx1, fy1] {
        be16(&mut head, v as u16);
    }
    be16(&mut head, 0);
    be16(&mut head, 6);
    be16(&mut head, 2);
    be16(&mut head, 1);
    be16(&mut head, 0);
    // hhea
    let mut hhea = Vec::new();
    be32(&mut hhea, 0x0001_0000);
    be16(&mut hhea, ascent as u16);
    be16(&mut hhea, (-descent) as u16);
    be16(&mut hhea, (f.gap_h as i32 * PX) as u16);
    be16(&mut hhea, adv_max);
    be16(&mut hhea, 0);
    be16(&mut hhea, min_rsb.min(0) as u16);
    be16(&mut hhea, max_extent as u16);
    be16(&mut hhea, 1);
    be16(&mut hhea, 0);
    be16(&mut hhea, 0);
    hhea.extend([0u8; 8]);
    be16(&mut hhea, 0);
    be16(&mut hhea, n);
    // maxp
    let mut maxp = Vec::new();
    be32(&mut maxp, 0x0001_0000);
    be16(&mut maxp, n);
    be16(&mut maxp, max_pts);
    be16(&mut maxp, max_cont);
    be16(&mut maxp, 0);
    be16(&mut maxp, 0);
    be16(&mut maxp, 2);
    for _ in 0..8 {
        be16(&mut maxp, 0);
    }
    // OS/2 version 4
    let mut os2 = Vec::new();
    be16(&mut os2, 4);
    be16(&mut os2, avg);
    be16(&mut os2, 400);
    be16(&mut os2, 5);
    be16(&mut os2, 0);
    let em = upm;
    for v in [
        em / 2,
        em / 2,
        0,
        em / 4,
        em / 2,
        em / 2,
        0,
        em / 4,
        PX as u16,
        (ascent / 2) as u16,
    ] {
        be16(&mut os2, v);
    }
    be16(&mut os2, 0);
    os2.extend([0u8; 10]);
    be32(&mut os2, 1);
    be32(&mut os2, 0);
    be32(&mut os2, 0);
    be32(&mut os2, 0);
    os2.extend(b"AOO ");
    be16(&mut os2, 0x40);
    be16(&mut os2, 0x20);
    be16(&mut os2, 0xfc);
    be16(&mut os2, ascent as u16);
    be16(&mut os2, (-descent) as u16);
    be16(&mut os2, (f.gap_h as i32 * PX) as u16);
    be16(&mut os2, fy1.max(ascent as i16) as u16);
    be16(&mut os2, (-fy0.min(-descent as i16)) as u16);
    be32(&mut os2, 1);
    be32(&mut os2, 0);
    be16(&mut os2, (ascent / 2) as u16);
    be16(&mut os2, ascent as u16);
    be16(&mut os2, 0);
    be16(&mut os2, 0x20);
    be16(&mut os2, 0);
    // cmap: format 4, one segment per run of consecutive characters and glyphs.
    let mut map: Vec<(u32, u16)> = all
        .iter()
        .enumerate()
        .map(|(i, (c, _))| (*c, i as u16 + 1))
        .collect();
    map.sort();
    let mut segs: Vec<(u16, u16, u16)> = Vec::new();
    for (c, g) in map {
        match segs.last_mut() {
            Some((start, end, g0)) if c as u16 == *end + 1 && g == *g0 + (*end - *start) + 1 => {
                *end = c as u16
            }
            _ => segs.push((c as u16, c as u16, g)),
        }
    }
    segs.push((0xffff, 0xffff, 1));
    let n_seg = segs.len() as u16;
    let es_c = 15 - n_seg.leading_zeros() as u16;
    let sr_c = 2 * (1u16 << es_c);
    let mut sub = Vec::new();
    be16(&mut sub, 4);
    be16(&mut sub, 16 + 8 * n_seg);
    be16(&mut sub, 0);
    be16(&mut sub, 2 * n_seg);
    be16(&mut sub, sr_c);
    be16(&mut sub, es_c);
    be16(&mut sub, 2 * n_seg - sr_c);
    for (_, end, _) in &segs {
        be16(&mut sub, *end);
    }
    be16(&mut sub, 0);
    for (start, _, _) in &segs {
        be16(&mut sub, *start);
    }
    for (start, _, g) in &segs {
        be16(&mut sub, g.wrapping_sub(*start));
    }
    for _ in &segs {
        be16(&mut sub, 0);
    }
    let mut cmap = Vec::new();
    be16(&mut cmap, 0);
    be16(&mut cmap, 1);
    be16(&mut cmap, 3);
    be16(&mut cmap, 1);
    be32(&mut cmap, 12);
    cmap.extend(sub);
    // name
    let ps: String = family
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let names = [
        (1u16, family.to_string()),
        (2, "Regular".into()),
        (3, format!("{ps}-Regular")),
        (4, family.to_string()),
        (5, "Version 1.0".into()),
        (6, ps),
    ];
    let mut strings = Vec::new();
    let mut name = Vec::new();
    be16(&mut name, 0);
    be16(&mut name, names.len() as u16);
    be16(&mut name, (6 + names.len() * 12) as u16);
    for (id, s) in &names {
        let bytes: Vec<u8> = s.encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
        for v in [
            3u16,
            1,
            0x409,
            *id,
            bytes.len() as u16,
            strings.len() as u16,
        ] {
            be16(&mut name, v);
        }
        strings.extend(bytes);
    }
    name.extend(strings);
    // post version 3
    let mut post = Vec::new();
    be32(&mut post, 0x0003_0000);
    be32(&mut post, 0);
    be16(&mut post, (-PX) as u16);
    be16(&mut post, PX as u16);
    be32(&mut post, 0);
    post.extend([0u8; 16]);

    let tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"OS/2", os2),
        (b"cmap", cmap),
        (b"glyf", glyf),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"loca", loca),
        (b"maxp", maxp),
        (b"name", name),
        (b"post", post),
    ];
    let nt = tables.len() as u16;
    let es = 15 - nt.leading_zeros() as u16;
    let sr = (1u16 << es) * 16;
    let mut out = Vec::new();
    be32(&mut out, 0x0001_0000);
    be16(&mut out, nt);
    be16(&mut out, sr);
    be16(&mut out, es);
    be16(&mut out, nt * 16 - sr);
    let mut offset = 12 + tables.len() * 16;
    let mut body = Vec::new();
    let mut head_at = 0;
    for (tag, data) in &tables {
        out.extend(*tag);
        be32(&mut out, checksum(data));
        be32(&mut out, offset as u32);
        be32(&mut out, data.len() as u32);
        if *tag == b"head" {
            head_at = offset;
        }
        body.extend(data);
        while body.len() % 4 != 0 {
            body.push(0);
        }
        offset = 12 + tables.len() * 16 + body.len();
    }
    out.extend(body);
    let adj = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
    out[head_at + 8..head_at + 12].copy_from_slice(&adj.to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        let mut glyphs = vec![vec![vec![false; 5]; 3]; 96];
        // "x": a 3x3 cross in rows 2..=4.
        glyphs[(b'x' - 0x20) as usize] = vec![
            vec![false, false, true, false, true],
            vec![false, false, false, true, false],
            vec![false, false, true, false, true],
        ];
        BitmapFont {
            height: 6,
            gap_w: 1,
            gap_h: 2,
            glyphs: glyphs
                .into_iter()
                .map(|g| {
                    g.into_iter()
                        .map(|mut c| {
                            c.resize(6, false);
                            c
                        })
                        .collect()
                })
                .collect(),
        }
    }

    #[test]
    fn writes_a_consistent_truetype_file() {
        let f = font();
        assert_eq!(baseline(&f), 5);
        let ttf = to_ttf(&f, "MOO1 Test");
        assert_eq!(&ttf[0..4], &[0, 1, 0, 0]);
        let nt = u16::from_be_bytes([ttf[4], ttf[5]]) as usize;
        assert_eq!(nt, 10);
        // Every table lies inside the file and the whole-file checksum is the magic one.
        for i in 0..nt {
            let r = &ttf[12 + i * 16..28 + i * 16];
            let off = u32::from_be_bytes([r[8], r[9], r[10], r[11]]) as usize;
            let len = u32::from_be_bytes([r[12], r[13], r[14], r[15]]) as usize;
            assert!(off % 4 == 0 && off + len <= ttf.len());
        }
        assert_eq!(checksum(&ttf), 0xB1B0_AFBA);
    }
}
