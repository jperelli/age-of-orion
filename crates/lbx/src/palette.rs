use crate::{Archive, Error, Result};

/// Number of palettes stored in FONTS.LBX (entries 2..=12).
pub const PALETTE_COUNT: usize = 11;

/// 256 RGB colours, 8 bits per channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette(pub [[u8; 3]; 256]);

/// VGA DACs use 6 bits per channel; widen to 8 bits the way 1oom does.
pub(crate) fn vga6_to_8(c: u8) -> u8 {
    let c = c & 0x3f;
    (c << 2) | (c >> 4)
}

impl Palette {
    /// Palette `n` is FONTS.LBX entry `n + 2`: 256 VGA 6-bit RGB triplets, followed by font
    /// colours (0x300), mouse cursors (0x500) and shading parameters (0x1500).
    /// Palette 0 is the main game palette (see 1oom `lbxpal_select`).
    pub fn from_fonts(fonts: &Archive, n: usize) -> Result<Self> {
        let e = fonts
            .entry(n + 2)
            .ok_or_else(|| Error::Missing(format!("FONTS.LBX palette {n}")))?;
        let raw = e.get(..768).ok_or(Error::Truncated("palette"))?;
        let mut p = [[0u8; 3]; 256];
        for (i, c) in p.iter_mut().enumerate() {
            *c = [
                vga6_to_8(raw[i * 3]),
                vga6_to_8(raw[i * 3 + 1]),
                vga6_to_8(raw[i * 3 + 2]),
            ];
        }
        Ok(Palette(p))
    }

    /// Used when FONTS.LBX is not available, so images still show their shapes.
    pub fn grayscale() -> Self {
        let mut p = [[0u8; 3]; 256];
        for (i, c) in p.iter_mut().enumerate() {
            *c = [i as u8; 3];
        }
        Palette(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widens_vga_colours() {
        assert_eq!(vga6_to_8(0), 0);
        assert_eq!(vga6_to_8(63), 255);
        assert_eq!(vga6_to_8(32), 130);
    }
}
