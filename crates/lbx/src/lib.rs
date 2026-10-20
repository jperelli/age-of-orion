//! Reader for the LBX archives of Master of Orion 1 (SimTex, 1993) and a converter
//! from LBX to an open asset tree (PNG images and JSON text).
//!
//! The format is documented by the 1oom project (`doc/format_lbx.txt`, `src/lbxgfx.c`,
//! `src/lbxpal.c`). The engine of Age of Orion never reads LBX at run time: the player's
//! own files are converted once, in the browser or with the CLI, by [`import`].

mod cp437;
mod export;
mod gfx;
mod palette;
mod table;

pub use export::{import, AssetFile, ImportResult, Manifest};
pub use gfx::{decode, encode_png, Frame, GfxHeader, Image};
pub use palette::{Palette, PALETTE_COUNT};
pub use table::{decode_cp437, Table};

use std::fmt;

/// Signature at offset 2 of every LBX file.
pub const LBX_MAGIC: u16 = 0xFEAD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotLbx,
    Truncated(&'static str),
    BadEntry(usize),
    Missing(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotLbx => write!(f, "not an LBX file (missing 0xFEAD signature)"),
            Error::Truncated(what) => write!(f, "truncated data while reading {what}"),
            Error::BadEntry(i) => write!(f, "entry {i} has an invalid offset"),
            Error::Missing(name) => write!(f, "required file {name} is missing"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Kind of content stored in an archive (header field at offset 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Graphics,
    Sound,
    Fonts,
    Help,
    Data,
    Other(u16),
}

impl From<u16> for Kind {
    fn from(v: u16) -> Self {
        match v {
            0 => Kind::Graphics,
            1 => Kind::Sound,
            2 => Kind::Fonts,
            3 => Kind::Help,
            5 => Kind::Data,
            n => Kind::Other(n),
        }
    }
}

/// A parsed LBX archive: a list of anonymous binary entries.
#[derive(Debug, Clone)]
pub struct Archive<'a> {
    pub kind: Kind,
    entries: Vec<&'a [u8]>,
}

pub(crate) fn u16_at(d: &[u8], off: usize) -> Option<u16> {
    d.get(off..off + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
}

pub(crate) fn u32_at(d: &[u8], off: usize) -> Option<u32> {
    d.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

impl<'a> Archive<'a> {
    /// Layout: `u16 count, u16 0xFEAD, u16 0, u16 kind, u32 offsets[count + 1]`.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let count = u16_at(data, 0).ok_or(Error::Truncated("header"))? as usize;
        if u16_at(data, 2) != Some(LBX_MAGIC) {
            return Err(Error::NotLbx);
        }
        let kind = Kind::from(u16_at(data, 6).ok_or(Error::Truncated("header"))?);
        let offsets = (0..=count)
            .map(|i| u32_at(data, 8 + 4 * i).map(|o| o as usize))
            .collect::<Option<Vec<_>>>()
            .ok_or(Error::Truncated("offset table"))?;
        let entries = offsets
            .windows(2)
            .enumerate()
            .map(|(i, w)| data.get(w[0]..w[1]).ok_or(Error::BadEntry(i)))
            .collect::<Result<Vec<_>>>()?;
        Ok(Archive { kind, entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entry(&self, i: usize) -> Option<&'a [u8]> {
        self.entries.get(i).copied()
    }

    pub fn entries(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        self.entries.iter().copied()
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    /// Builds an LBX archive in memory from raw entries.
    pub fn build_lbx(kind: u16, entries: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend((entries.len() as u16).to_le_bytes());
        out.extend(super::LBX_MAGIC.to_le_bytes());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_container() {
        let data = testutil::build_lbx(5, &[vec![1, 2, 3], vec![], vec![9]]);
        let a = Archive::parse(&data).unwrap();
        assert_eq!(a.kind, Kind::Data);
        assert_eq!(a.len(), 3);
        assert_eq!(a.entry(0), Some(&[1u8, 2, 3][..]));
        assert_eq!(a.entry(1), Some(&[][..]));
        assert_eq!(a.entry(2), Some(&[9u8][..]));
    }

    #[test]
    fn rejects_non_lbx() {
        assert_eq!(Archive::parse(&[0; 16]).unwrap_err(), Error::NotLbx);
        assert!(matches!(Archive::parse(&[1]), Err(Error::Truncated(_))));
    }
}
