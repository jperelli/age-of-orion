//! LBX set -> open asset tree (`manifest.json`, `gfx/**.png`, `text/**.json`).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::font::{self, FontInfo};
use crate::gfx::{decode, encode_png, GfxHeader};
use crate::music;
use crate::sfx;
use crate::{Archive, Kind, Palette, Table};

/// One file of the generated asset tree.
#[derive(Debug, Clone)]
pub struct AssetFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryInfo {
    pub index: usize,
    pub bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gfx: Option<GfxHeader>,
    /// Path of the PNG; multi-frame images are horizontal sheets of `gfx.width` frames.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub png: Option<String>,
    /// First frame alone, for multi-frame images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub png0: Option<String>,
    /// MIDI file converted from an XMIDI tune.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub loops: bool,
    /// WAV file converted from a sound effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wav: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchiveInfo {
    pub kind: Kind,
    pub entries: Vec<EntryInfo>,
}

/// `manifest.json` of an imported asset tree.
#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub format: &'static str,
    pub version: u32,
    pub source: &'static str,
    /// Semantic asset names used by the game UI -> file path.
    pub aliases: BTreeMap<String, String>,
    pub archives: BTreeMap<String, ArchiveInfo>,
    pub text: Vec<String>,
    /// Tunes that loop (aliases `music/<n>`).
    pub looping: Vec<String>,
    /// MOO1 fonts converted to TrueType (aliases `font/<n>`).
    pub fonts: Vec<FontInfo>,
    /// The animations (aliases `anim/...`, horizontal sheets of equal frames): frames,
    /// frame width and height, and the frame the loop restarts from.
    pub frames: BTreeMap<String, [u16; 4]>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub files: Vec<AssetFile>,
    pub manifest: Manifest,
}

/// Placeholders used by EVENTMSG.LBX (high CP437 characters, 1oom `game_news_get_msg`)
/// -> named tokens.
const EVENT_TOKENS: [(char, &str); 13] = [
    ('Ç', "{planet}"),
    ('ü', "{n1}"),
    ('é', "{n2}"),
    ('â', "{s}"),
    ('ä', "{race}"),
    ('à', "{other_race}"),
    ('å', "{field}"),
    ('ç', "{tech}"),
    ('ê', "{personality}"),
    ('ë', "{objective}"),
    ('î', "{leader}"),
    ('ì', "{s1}"),
    ('Ä', "{an}"),
];

fn stem(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    base.split('.').next().unwrap_or(base).to_ascii_uppercase()
}

/// Converts a set of LBX files (`(file name, bytes)`) into an asset tree.
///
/// FONTS.LBX provides the palettes; without it images are exported in grayscale.
/// Files that fail to parse are reported in `manifest.warnings` and skipped.
pub fn import(files: &[(String, Vec<u8>)]) -> ImportResult {
    let mut warnings = Vec::new();
    let mut archives = BTreeMap::new();
    let mut out = Vec::new();
    let mut text = Vec::new();

    let parsed: Vec<(String, Archive)> = files
        .iter()
        .filter_map(|(name, data)| match Archive::parse(data) {
            Ok(a) => Some((stem(name), a)),
            Err(e) => {
                warnings.push(format!("{name}: {e}"));
                None
            }
        })
        .collect();

    let base_palette = parsed
        .iter()
        .find(|(n, _)| n == "FONTS")
        .and_then(|(_, a)| Palette::from_fonts(a, 0).ok())
        .unwrap_or_else(|| {
            warnings.push("FONTS.LBX missing: images exported in grayscale".into());
            Palette::grayscale()
        });

    for (name, archive) in &parsed {
        let dir = name.to_ascii_lowercase();
        let mut entries = Vec::new();
        // The landing and ground battle screens use FONTS.LBX palette 6 (1oom `uilanding.c`).
        let archive_palette = (name == "LANDING")
            .then(|| parsed.iter().find(|(n, _)| n == "FONTS"))
            .flatten()
            .and_then(|(_, a)| Palette::from_fonts(a, 6).ok());
        let base_palette = archive_palette.as_ref().unwrap_or(&base_palette);
        for (index, d) in archive.entries().enumerate() {
            let mut info = EntryInfo {
                index,
                bytes: d.len(),
                gfx: None,
                png: None,
                png0: None,
                midi: None,
                loops: false,
                wav: None,
                error: None,
            };
            if sfx::is_voc(d) {
                match sfx::voc_to_wav(d) {
                    Some(wav) => {
                        let path = format!("audio/{dir}/{index:03}.wav");
                        out.push(AssetFile {
                            path: path.clone(),
                            bytes: wav,
                        });
                        info.wav = Some(path);
                    }
                    None => info.error = Some("invalid VOC".into()),
                }
            }
            if music::is_xmid(d) {
                match music::convert_xmid(d) {
                    Some((midi, loops)) => {
                        let path = format!("audio/{dir}/{index:03}.mid");
                        out.push(AssetFile {
                            path: path.clone(),
                            bytes: midi,
                        });
                        info.midi = Some(path);
                        info.loops = loops;
                    }
                    None => info.error = Some("invalid XMIDI".into()),
                }
            }
            if archive.kind == Kind::Graphics && d.len() >= 0x16 {
                match decode(d) {
                    Ok(img) => {
                        let pal = img.palette(base_palette);
                        let (w, h, px) = img.sheet_rgba(&pal);
                        let path = format!("gfx/{dir}/{index:03}.png");
                        out.push(AssetFile {
                            path: path.clone(),
                            bytes: encode_png(w, h, &px),
                        });
                        // The GNN logo is shown at its last frame (1oom `uinews.c`).
                        if name == "NEWSCAST" && index == 1 && img.frames.len() > 1 {
                            let last = img.frames.len() - 1;
                            let (w, h) = (img.header.width as u32, img.header.height as u32);
                            out.push(AssetFile {
                                path: format!("gfx/{dir}/{index:03}-last.png"),
                                bytes: encode_png(w, h, &img.frame_rgba(last, &pal)),
                            });
                        }
                        if img.frames.len() > 1 {
                            let path0 = format!("gfx/{dir}/{index:03}-0.png");
                            let (w, h) = (img.header.width as u32, img.header.height as u32);
                            out.push(AssetFile {
                                path: path0.clone(),
                                bytes: encode_png(w, h, &img.frame_rgba(0, &pal)),
                            });
                            info.png0 = Some(path0);
                        }
                        info.gfx = Some(img.header);
                        info.png = Some(path);
                    }
                    Err(e) => info.error = Some(e.to_string()),
                }
            }
            entries.push(info);
        }
        if let Some(strings) = text_table(name, archive) {
            let path = format!("text/lbx/{dir}.json");
            out.push(AssetFile {
                path: path.clone(),
                bytes: serde_json::to_vec_pretty(&strings).expect("json"),
            });
            text.push(path);
        }
        if let Some(msgs) = (name == "DIPLOMAT").then(|| diplomat(archive)).flatten() {
            let path = "text/lbx/diplomat.json".to_string();
            out.push(AssetFile {
                path: path.clone(),
                bytes: serde_json::to_vec_pretty(&msgs).expect("json"),
            });
            text.push(path);
        }
        archives.insert(
            name.clone(),
            ArchiveInfo {
                kind: archive.kind,
                entries,
            },
        );
    }

    let mut fonts = Vec::new();
    if let Some(d) = parsed
        .iter()
        .find(|(n, _)| n == "FONTS")
        .and_then(|(_, a)| a.entry(0))
    {
        for (i, f) in font::parse(d).iter().enumerate() {
            let path = format!("fonts/moo1-{i}.ttf");
            out.push(AssetFile {
                path: path.clone(),
                bytes: font::to_ttf(f, &format!("MOO1 Font {i}")),
            });
            fonts.push(FontInfo {
                index: i,
                path,
                height: f.height,
                gap_h: f.gap_h,
            });
        }
    }
    let mut aliases = aliases(&archives);
    for f in &fonts {
        aliases.insert(format!("font/{}", f.index), f.path.clone());
    }
    // RESEARCH.LBX entry 0 gives each tech its picture in TECHNO.LBX (1oom
    // `game_tech_get_gfx_i`); advanced techs have one per field (1oom `ui_newtech`).
    let techno = |i: usize| {
        archives
            .get("TECHNO")
            .and_then(|t| t.entries.get(i))
            .and_then(|e| e.png.clone())
    };
    if let Some((_, research)) = parsed.iter().find(|(n, _)| n == "RESEARCH") {
        for (field, tech, gfx) in tech_pictures(research) {
            if let Some(p) = techno(gfx) {
                aliases.insert(format!("tech/{field}/{tech}"), p);
            }
        }
    }
    for (field, gfx) in [9, 22, 20, 25, 15, 29].into_iter().enumerate() {
        if let Some(p) = techno(gfx) {
            aliases.insert(format!("tech/{field}/advanced"), p);
        }
    }
    let mut frames = BTreeMap::new();
    for (alias, (path, n)) in animations(&archives) {
        aliases.insert(alias.clone(), path);
        frames.insert(alias, n);
    }
    let looping = archives
        .get("MUSIC")
        .map(|m| {
            m.entries
                .iter()
                .filter(|e| e.loops)
                .map(|e| format!("music/{}", e.index))
                .collect()
        })
        .unwrap_or_default();
    let manifest = Manifest {
        format: "age-of-orion-assets",
        version: 2,
        source: "lbx",
        aliases,
        archives,
        text,
        looping,
        fonts,
        frames,
        warnings,
    };
    out.push(AssetFile {
        path: "manifest.json".into(),
        bytes: serde_json::to_vec_pretty(&manifest).expect("json"),
    });
    ImportResult {
        files: out,
        manifest,
    }
}

/// RESEARCH.LBX entry 0: 6 bytes per tech (field * 50 + tech - 1) after a 4-byte header, the
/// third the tech's TECHNO.LBX picture (pictures start at 8; 0 for no tech).
fn tech_pictures(archive: &Archive) -> Vec<(usize, u8, usize)> {
    let Some(d) = archive.entry(0) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for field in 0..6 {
        for tech in 1..=50u8 {
            let off = 4 + (field * 50 + tech as usize - 1) * 6 + 2;
            if let Some(&gfx) = d.get(off) {
                if gfx >= 8 {
                    out.push((field, tech, gfx as usize));
                }
            }
        }
    }
    out
}

fn text_table(name: &str, archive: &Archive) -> Option<Vec<String>> {
    // RESEARCH.LBX entry 2: tech descriptions, field * 50 + tech - 1 (1oom `game_tech_get_descr`).
    let entry = match name {
        "NAMES" | "EVENTMSG" => 0,
        "RESEARCH" => 2,
        _ => return None,
    };
    if archive.kind != Kind::Data && name != "RESEARCH" {
        return None;
    }
    let strings = Table::parse(archive.entry(entry)?).ok()?.strings();
    Some(
        strings
            .into_iter()
            .map(|s| {
                EVENT_TOKENS
                    .iter()
                    .fold(s, |s, (c, tok)| s.replace(*c, tok))
            })
            .collect(),
    )
}

/// DIPLOMAT.LBX's placeholders (byte 0x80 + code, 1oom `audience_build_diplo_msg`) as the
/// game's own: the races, rulers, planet, treaty, tech and amounts of the message.
fn diplomat_token(code: u8) -> &'static str {
    match code {
        0 => "{me}",
        1 => "{race}",
        2 => "{sabotage}",
        3 => "{player}",
        4 => "{field}",
        5 => "{planet}",
        6 | 8 | 0xc | 0x11 | 0x12 => "{empire}",
        7 | 0x13 | 0x15 => "{bc}",
        9 => "{leader}",
        0xa => " {recalled}",
        0xb => "{treaty}",
        0xd => "{gift}.",
        0xe | 0x14 => "{tech} tech.",
        0xf => "{year}",
        0x10 => "{tech}",
        0x16 => "{an}",
        0x17 => "{broken}",
        _ => "",
    }
}

/// DIPLOMAT.LBX: the messages of the computer rulers, up to 15 wordings for each of the 81
/// kinds of message (1oom `DIPLOMAT_MSG_PTR`, `diplomat.d0`), as `[kind][wording]` =
/// `[text, music]` (music 2: the tense tune of the race's audience).
fn diplomat(archive: &Archive) -> Option<Vec<Vec<(String, u8)>>> {
    let counts = Table::parse(archive.entry(0)?).ok()?;
    let msgs = Table::parse(archive.entry(1)?).ok()?;
    let recs = msgs.records();
    let out = counts
        .records()
        .iter()
        .enumerate()
        .map(|(t, c)| {
            let n = (u16::from_le_bytes([c[0], c[1]]) as usize).min(15);
            (0..n)
                .filter_map(|r| recs.get(t * 15 + r))
                .map(|m| {
                    let text: String = m
                        .iter()
                        .take(0xc4)
                        .take_while(|&&b| b != 0)
                        .map(|&b| match b {
                            0x80.. => diplomat_token(b & 0x7f).to_string(),
                            0x20..=0x7f => (b as char).to_string(),
                            _ => String::new(),
                        })
                        .collect();
                    (
                        text.split_whitespace().collect::<Vec<_>>().join(" "),
                        m.get(0xc6).copied().unwrap_or(0),
                    )
                })
                .collect()
        })
        .collect();
    Some(out)
}

/// The animations the game plays (1oom draws them frame by frame): alias -> (sheet,
/// [frames, width, height, loop frame]).
/// Their first frames also have static aliases (`ship/<look>`, `star/<k>`, ...).
fn animations(archives: &BTreeMap<String, ArchiveInfo>) -> BTreeMap<String, (String, [u16; 4])> {
    let mut a = BTreeMap::new();
    let sheet = |file: &str, i: usize| {
        let e = archives.get(file)?.entries.get(i)?;
        let g = e.gfx?;
        let loop_frame = if g.loop_frame < g.frames {
            g.loop_frame
        } else {
            0
        };
        if g.frames > 1 {
            Some((e.png.clone()?, [g.frames, g.width, g.height, loop_frame]))
        } else {
            None
        }
    };
    let mut put = |alias: String, s: Option<(String, [u16; 4])>| {
        if let Some(s) = s {
            a.insert(alias, s);
        }
    };
    // Twinkling stars (1oom `ui_starmap_draw_starmap`), as `star/<k>`.
    for (i, k) in [0, 1, 2, 4, 3, 5, 6, 7, 8, 10, 9, 11]
        .into_iter()
        .enumerate()
    {
        put(format!("anim/star/{k}"), sheet("STARMAP", 3 + i));
    }
    // Ships' engines (1oom `uibattle.c` `frame_ship`), as `ship/<look>`.
    for look in 0..0x93 {
        let s = match look {
            0..=0x45 => sheet("SHIPS2", look),
            0x48..=0x8f => sheet("SHIPS", look - 0x48),
            0x90.. => sheet("SHIPS2", look - 0x48),
            _ => None,
        };
        put(format!("anim/ship/{look}"), s);
    }
    // EMBASSY.LBX (1oom `uiaudience.c`): each race's ambassador 2 + race.
    for r in 0..10 {
        put(format!("anim/embassy/{r}"), sheet("EMBASSY", 2 + r));
    }
    // NEWSCAST.LBX (1oom `uinews.c`): the newscaster 2 and the globe 3.
    put("anim/news/anchor".into(), sheet("NEWSCAST", 2));
    put("anim/news/world".into(), sheet("NEWSCAST", 3));
    // LANDING.LBX (1oom `uilanding.c`): the transport coming down 0, the colonists walking
    // out 0x15 + banner. BACKGRND.LBX: the bombs falling 0x10 (1oom `uibomb.c`), missile
    // bases 7 and factories 8 blowing up (`uispy_sab.c`).
    put("anim/landing/transport".into(), sheet("LANDING", 0));
    for b in 0..6 {
        put(format!("anim/landing/walk/{b}"), sheet("LANDING", 0x15 + b));
    }
    put("anim/bombard".into(), sheet("BACKGRND", 0x10));
    put("anim/sabotage/bases".into(), sheet("BACKGRND", 7));
    put("anim/sabotage/factories".into(), sheet("BACKGRND", 8));
    // SPIES.LBX (1oom `uinewtech.c`): 10 * source + race; source 0 is research, 1 spies,
    // 2 artifacts.
    for i in 0..30 {
        put(format!("anim/newtech/{i}"), sheet("SPIES", i));
    }
    a
}

/// Semantic names for the images and tunes the game UI knows how to use (indices from 1oom
/// `ui/classic/uiclassic.c`): PLANETS.LBX 0..=34 planet globes, 35..=44 race portraits,
/// STARMAP.LBX nebulae and the stargate, battle pictures from SHIPS/SHIPS2/SPACE/MISSILE.LBX,
/// MUSIC.LBX tunes.
fn aliases(archives: &BTreeMap<String, ArchiveInfo>) -> BTreeMap<String, String> {
    let mut a = BTreeMap::new();
    let entry = |file: &str, i: usize| archives.get(file).and_then(|f| f.entries.get(i));
    let png = |file: &str, i: usize| entry(file, i).and_then(|e| e.png.clone());
    let png0 = |file: &str, i: usize| entry(file, i).and_then(|e| e.png0.clone().or(e.png.clone()));
    // Ship looks (1oom `ui_data.gfx.ships`): 0..0x48 SHIPS2.LBX, 0x48..0x90 SHIPS.LBX,
    // 0x90..0x93 the crystal, the amoeba and the Guardian (SHIPS2.LBX 0x48..).
    for look in 0..0x93 {
        let p = match look {
            0..=0x45 => png0("SHIPS2", look),
            0x48..=0x8f => png0("SHIPS", look - 0x48),
            0x90.. => png0("SHIPS2", look - 0x48),
            _ => None,
        };
        if let Some(p) = p {
            a.insert(format!("ship/{look}"), p);
        }
    }
    for i in 0..5 {
        if let Some(p) = png0("SPACE", 0x1a + i) {
            a.insert(format!("battle/bg/{i}"), p);
        }
    }
    for (i, e) in [0x28, 0x29, 0x37, 0x38].into_iter().enumerate() {
        if let Some(p) = png0("SPACE", e) {
            a.insert(format!("battle/rock/{i}"), p);
        }
    }
    for i in 0..10 {
        if let Some(p) = png0("SPACE", 7 + i) {
            a.insert(format!("battle/explosion/{i}"), p);
        }
    }
    // Missiles (1oom `ui_data.gfx.missile`): 5 kinds x 8 directions.
    for i in 0..40 {
        if let Some(p) = png0("MISSILE", i) {
            a.insert(format!("missile/{}/{}", i / 8, i % 8), p);
        }
    }
    if let Some(m) = archives.get("MUSIC") {
        for e in &m.entries {
            if let Some(p) = &e.midi {
                a.insert(format!("music/{}", e.index), p.clone());
            }
        }
    }
    for i in 0..35 {
        if let Some(p) = png("PLANETS", i) {
            a.insert(format!("planet/{i}"), p);
        }
    }
    for r in 0..10 {
        if let Some(p) = png("PLANETS", 35 + r) {
            a.insert(format!("race/{r}"), p);
        }
    }
    // STARMAP.LBX (1oom `uigame.c`, `uiclassic.c`): nebulae 0x0f.., the stargate 0x2d.
    for t in 0..10 {
        if let Some(p) = png("STARMAP", 0x0f + t) {
            a.insert(format!("nebula/{t}"), p);
        }
    }
    if let Some(p) = png("STARMAP", 0x2d) {
        a.insert("stargate".into(), p);
    }
    // Stars 3..0x0e, star type + look (0 or 6), in 1oom's order `{0, 1, 2, 4, 3, 5, ...}`.
    for (i, k) in [0, 1, 2, 4, 3, 5, 6, 7, 8, 10, 9, 11]
        .into_iter()
        .enumerate()
    {
        if let Some(p) = png0("STARMAP", 3 + i) {
            a.insert(format!("star/{k}"), p);
        }
    }
    // LANDING.LBX (1oom `uilanding.c`, `uiground.c`): the transport 0, colonization scenes
    // 6 + planet type, invasion scenes 0x24 + planet type, the fallen soldier 0x14, the
    // walking colonists 0x15 + banner, soldiers 0x1b + race.
    if let Some(p) = png0("LANDING", 0) {
        a.insert("landing/transport".into(), p);
    }
    for t in 1..14 {
        if let Some(p) = png0("LANDING", 6 + t) {
            a.insert(format!("landing/colonize/{t}"), p);
        }
        if let Some(p) = png0("LANDING", 0x24 + t) {
            a.insert(format!("landing/invade/{t}"), p);
        }
    }
    if let Some(p) = png("LANDING", 0x14) {
        a.insert("ground/death".into(), p);
    }
    for r in 0..10 {
        if let Some(p) = png0("LANDING", 0x1b + r) {
            a.insert(format!("ground/soldier/{r}"), p);
        }
    }
    // EMBASSY.LBX (1oom `uiaudience.c`): the frame 0, the room 1, the player's emperor seen
    // from behind 0xc + banner.
    if let Some(p) = png("EMBASSY", 0) {
        a.insert("embassy/border".into(), p);
    }
    if let Some(p) = png0("EMBASSY", 1) {
        a.insert("embassy/room".into(), p);
    }
    for b in 0..6 {
        if let Some(p) = png0("EMBASSY", 0xc + b) {
            a.insert(format!("embassy/emperor/{b}"), p);
        }
    }
    // TECHNO.LBX (1oom `uinewtech.c`): the laboratory 0, the spies' den 1, the ruins 2.
    for i in 0..3 {
        if let Some(p) = png0("TECHNO", i) {
            a.insert(format!("newtech/lab/{i}"), p);
        }
    }
    // NEWSCAST.LBX (1oom `uinews.c`): the TV 0, the GNN logo 1, the newscaster 2, the
    // globe 3 and an icon per news type at 3 + type.
    for (i, k) in ["news/tv", "news/gnn", "news/anchor", "news/world"]
        .iter()
        .enumerate()
    {
        if let Some(p) = png0("NEWSCAST", i) {
            let p = if i == 1 {
                p.replace("-0.png", "-last.png")
            } else {
                p
            };
            a.insert(k.to_string(), p);
        }
    }
    for t in 1..23 {
        if let Some(p) = png0("NEWSCAST", 3 + t) {
            a.insert(format!("news/icon/{t}"), p);
        }
    }
    // LANDING.LBX: the colonists walking out of the transport, by banner.
    for b in 0..6 {
        if let Some(p) = png0("LANDING", 0x15 + b) {
            a.insert(format!("landing/walk/{b}"), p);
        }
    }
    // COLONIES.LBX (1oom `uiexplore.c`): each kind of planet, 2 * type without a colony and
    // 2 * type + 1 with one.
    for t in 0..14 {
        if let Some(p) = png("COLONIES", 2 * t) {
            a.insert(format!("explore/{t}"), p);
        }
        if let Some(p) = png("COLONIES", 2 * t + 1) {
            a.insert(format!("explore/{t}/colony"), p);
        }
    }
    // BACKGRND.LBX: the bombs falling (0x10), missile bases (7) and factories (8) blowing up,
    // the espionage panel with the two agents (0xd, 1oom `uispy_esp.c`).
    for (alias, i) in [
        ("bombard", 0x10),
        ("sabotage/bases", 7),
        ("sabotage/factories", 8),
    ] {
        if let Some(p) = png0("BACKGRND", i) {
            a.insert(alias.into(), p);
        }
    }
    if let Some(p) = png("BACKGRND", 0xd) {
        a.insert("spy/espionage".into(), p);
    }
    // SOUNDFX.LBX (1oom `ui_sound_play_sfx`).
    if let Some(f) = archives.get("SOUNDFX") {
        for e in &f.entries {
            if let Some(p) = &e.wav {
                a.insert(format!("sfx/{}", e.index), p.clone());
            }
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::tests::sample_gfx;
    use crate::testutil::build_lbx;

    #[test]
    fn exports_png_text_and_manifest() {
        let mut names = vec![2, 0, 4, 0];
        names.extend(b"Ann\0");
        names.extend([b'a', 0x80, 0, 0]);
        let files = vec![
            ("PLANETS.LBX".to_string(), build_lbx(0, &[sample_gfx()])),
            ("dir/names.lbx".to_string(), build_lbx(5, &[names])),
            ("BROKEN.LBX".to_string(), vec![1, 2, 3]),
        ];
        let r = import(&files);
        let paths: Vec<_> = r.files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"gfx/planets/000.png"));
        assert!(paths.contains(&"text/lbx/names.json"));
        assert!(paths.contains(&"manifest.json"));
        assert_eq!(r.manifest.aliases["planet/0"], "gfx/planets/000.png");
        assert_eq!(r.manifest.warnings.len(), 2); // broken file + no FONTS.LBX
        let names = r
            .files
            .iter()
            .find(|f| f.path == "text/lbx/names.json")
            .unwrap();
        let v: Vec<String> = serde_json::from_slice(&names.bytes).unwrap();
        assert_eq!(v, vec!["Ann", "a{planet}"]);
    }

    /// `sample_gfx` with its frame twice.
    fn two_frames() -> Vec<u8> {
        let one = sample_gfx();
        let frame = one[0x12 + 8..].to_vec();
        let mut d = one[..0x12].to_vec();
        d[6] = 2;
        let start = 0x12 + 12;
        for k in 0..3 {
            d.extend(((start + k * frame.len()) as u32).to_le_bytes());
        }
        d.extend(&frame);
        d.extend(&frame);
        d
    }

    #[test]
    fn lists_animations_with_their_frames() {
        let files = vec![(
            "NEWSCAST.LBX".to_string(),
            build_lbx(0, &[sample_gfx(), sample_gfx(), two_frames()]),
        )];
        let r = import(&files);
        assert_eq!(
            r.manifest.aliases["anim/news/anchor"],
            "gfx/newscast/002.png"
        );
        assert_eq!(r.manifest.frames["anim/news/anchor"], [2, 2, 3, 0]);
        // The still picture is the first frame alone.
        assert_eq!(r.manifest.aliases["news/anchor"], "gfx/newscast/002-0.png");
        assert!(!r.manifest.frames.contains_key("news/tv"));
    }

    #[test]
    fn gives_each_tech_its_blueprint() {
        // RESEARCH.LBX entry 0: weapon tech 4 (field 5) has picture 8 in TECHNO.LBX.
        let mut table = vec![0x2c, 0x01, 0x06, 0x00];
        table.extend(vec![0u8; 300 * 6]);
        table[4 + (5 * 50 + 3) * 6 + 2] = 8;
        let techno: Vec<Vec<u8>> = (0..10).map(|_| sample_gfx()).collect();
        let files = vec![
            ("RESEARCH.LBX".to_string(), build_lbx(5, &[table])),
            ("TECHNO.LBX".to_string(), build_lbx(0, &techno)),
        ];
        let r = import(&files);
        assert_eq!(r.manifest.aliases["tech/5/4"], "gfx/techno/008.png");
        assert_eq!(r.manifest.aliases["tech/0/advanced"], "gfx/techno/009.png");
        assert!(!r.manifest.aliases.contains_key("tech/5/5"));
    }
}
