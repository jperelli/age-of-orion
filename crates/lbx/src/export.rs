//! LBX set -> open asset tree (`manifest.json`, `gfx/**.png`, `text/**.json`).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::gfx::{decode, encode_png, GfxHeader};
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
        for (index, d) in archive.entries().enumerate() {
            let mut info = EntryInfo {
                index,
                bytes: d.len(),
                gfx: None,
                png: None,
                error: None,
            };
            if archive.kind == Kind::Graphics && d.len() >= 0x16 {
                match decode(d) {
                    Ok(img) => {
                        let pal = img.palette(&base_palette);
                        let (w, h, px) = img.sheet_rgba(&pal);
                        let path = format!("gfx/{dir}/{index:03}.png");
                        out.push(AssetFile {
                            path: path.clone(),
                            bytes: encode_png(w, h, &px),
                        });
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
        archives.insert(
            name.clone(),
            ArchiveInfo {
                kind: archive.kind,
                entries,
            },
        );
    }

    let aliases = aliases(&archives);
    let manifest = Manifest {
        format: "age-of-orion-assets",
        version: 1,
        source: "lbx",
        aliases,
        archives,
        text,
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

fn text_table(name: &str, archive: &Archive) -> Option<Vec<String>> {
    if archive.kind != Kind::Data || !matches!(name, "NAMES" | "EVENTMSG") {
        return None;
    }
    let strings = Table::parse(archive.entry(0)?).ok()?.strings();
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

/// Semantic names for the images the game UI knows how to use (indices from 1oom
/// `ui/classic/uiclassic.c`): PLANETS.LBX 0..=34 planet globes, 35..=44 race portraits,
/// STARMAP.LBX nebulae and the stargate.
fn aliases(archives: &BTreeMap<String, ArchiveInfo>) -> BTreeMap<String, String> {
    let mut a = BTreeMap::new();
    let png = |file: &str, i: usize| {
        archives
            .get(file)
            .and_then(|f| f.entries.get(i))
            .and_then(|e| e.png.clone())
    };
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
}
