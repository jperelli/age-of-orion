use crate::palette::{vga6_to_8, Palette};
use crate::{u16_at, u32_at, Error, Result};

/// Header of a graphics entry (LBX kind 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct GfxHeader {
    pub width: u16,
    pub height: u16,
    pub frames: u16,
    pub loop_frame: u16,
    /// Offset of the embedded palette metadata, 0 if none.
    pub palette_offset: u16,
    pub independent_frames: bool,
    /// 0 = normal, 1 = pixels >= 0xE8 shade what is already on screen.
    pub format: u8,
}

impl GfxHeader {
    pub fn parse(d: &[u8]) -> Result<Self> {
        let f = |off| u16_at(d, off).ok_or(Error::Truncated("gfx header"));
        Ok(GfxHeader {
            width: f(0)?,
            height: f(2)?,
            frames: f(6)?,
            loop_frame: f(8)?,
            palette_offset: f(0x0e)?,
            independent_frames: *d.get(0x10).ok_or(Error::Truncated("gfx header"))? != 0,
            format: *d.get(0x11).ok_or(Error::Truncated("gfx header"))?,
        })
    }
}

pub const TRANSPARENT: u16 = 0xffff;
/// Pixels `SHADE + k` darken the background through shading table `k` (format 1).
pub const SHADE: u16 = 0x100;

/// One decoded frame: `width * height` pixels, row-major. Values are palette indices,
/// [`TRANSPARENT`] or `SHADE + k`.
pub type Frame = Vec<u16>;

#[derive(Debug, Clone)]
pub struct Image {
    pub header: GfxHeader,
    pub frames: Vec<Frame>,
    /// Colours from the embedded palette: `(first_index, rgb...)`.
    pub embedded_palette: Option<(usize, Vec<[u8; 3]>)>,
}

struct Reader<'a> {
    d: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Result<u8> {
        let b = *self.d.get(self.pos).ok_or(Error::Truncated("gfx frame"))?;
        self.pos += 1;
        Ok(b)
    }
}

/// Decodes all frames of a graphics entry.
///
/// Frame data mirrors 1oom `lbxgfx_draw_pixels_fmt0/1`: byte 0 is 1 to clear the buffer
/// (0 draws over the previous frame), then column by column a mode byte (0xFF = empty
/// column, bit 0x80 = compressed), a total length, then segments of
/// `run, skip_rows, run bytes`. In compressed columns a byte above 0xDF repeats the next
/// byte `b - 0xDF` times, and both bytes count against `run`.
pub fn decode(d: &[u8]) -> Result<Image> {
    let header = GfxHeader::parse(d)?;
    let (w, h, n) = (
        header.width as usize,
        header.height as usize,
        header.frames as usize,
    );
    let offsets = (0..=n)
        .map(|i| u32_at(d, 0x12 + 4 * i).map(|o| o as usize))
        .collect::<Option<Vec<_>>>()
        .ok_or(Error::Truncated("frame offsets"))?;
    let mut buf = vec![TRANSPARENT; w * h];
    let mut frames = Vec::with_capacity(n);
    for fi in 0..n {
        let fd = d
            .get(offsets[fi]..offsets[fi + 1].max(offsets[fi]))
            .ok_or(Error::Truncated("frame data"))?;
        if !fd.is_empty() {
            draw_frame(fd, &mut buf, w, h, header.format)?;
        }
        frames.push(buf.clone());
    }
    let embedded_palette = embedded_palette(d, header.palette_offset as usize);
    Ok(Image {
        header,
        frames,
        embedded_palette,
    })
}

fn draw_frame(fd: &[u8], buf: &mut [u16], w: usize, h: usize, format: u8) -> Result<()> {
    if fd[0] == 1 {
        buf.fill(TRANSPARENT);
    }
    let mut r = Reader { d: fd, pos: 1 };
    let pixel = |b: u8| -> u16 {
        if format == 1 && b >= 0xe8 {
            SHADE + (b - 0xe8) as u16
        } else {
            b as u16
        }
    };
    for x in 0..w {
        if r.pos >= fd.len() {
            break;
        }
        let mode = r.byte()?;
        if mode == 0xff {
            continue;
        }
        let compressed = mode & 0x80 != 0;
        let mut total = r.byte()? as i32;
        let mut y = 0usize;
        while total >= 1 {
            let mut run = r.byte()? as i32;
            y += r.byte()? as usize;
            total -= run + 2;
            while run > 0 {
                let mut v = r.byte()?;
                let mut len = 1;
                if compressed && v > 0xdf {
                    len = (v - 0xdf) as usize;
                    v = r.byte()?;
                    run -= 1;
                }
                run -= 1;
                for _ in 0..len {
                    if y < h {
                        buf[y * w + x] = pixel(v);
                    }
                    y += 1;
                }
            }
        }
    }
    Ok(())
}

/// Palette metadata: `u16 data_offset, u16 first_colour, u16 count`, VGA 6-bit RGB.
fn embedded_palette(d: &[u8], off: usize) -> Option<(usize, Vec<[u8; 3]>)> {
    if off == 0 {
        return None;
    }
    let data = u16_at(d, off)? as usize;
    let first = u16_at(d, off + 2)? as usize;
    let count = (u16_at(d, off + 4)? as usize).min(256usize.saturating_sub(first));
    let raw = d.get(data..data + count * 3)?;
    Some((
        first,
        raw.chunks(3)
            .map(|c| [vga6_to_8(c[0]), vga6_to_8(c[1]), vga6_to_8(c[2])])
            .collect(),
    ))
}

impl Image {
    /// The palette to render this image with: `base` overridden by the embedded colours.
    pub fn palette(&self, base: &Palette) -> Palette {
        let mut p = base.clone();
        if let Some((first, cols)) = &self.embedded_palette {
            for (i, c) in cols.iter().enumerate() {
                p.0[first + i] = *c;
            }
        }
        p
    }

    /// RGBA pixels of one frame.
    pub fn frame_rgba(&self, frame: usize, pal: &Palette) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.frames[frame].len() * 4);
        for &v in &self.frames[frame] {
            match v {
                TRANSPARENT => out.extend([0, 0, 0, 0]),
                v if v >= SHADE => out.extend([0, 0, 0, 96 + 8 * (v - SHADE).min(16) as u8]),
                v => {
                    let c = pal.0[v as usize];
                    out.extend([c[0], c[1], c[2], 255]);
                }
            }
        }
        out
    }

    /// All frames side by side (frame width = header width), as RGBA.
    pub fn sheet_rgba(&self, pal: &Palette) -> (u32, u32, Vec<u8>) {
        let (w, h) = (self.header.width as usize, self.header.height as usize);
        let n = self.frames.len().max(1);
        let mut out = vec![0u8; w * n * h * 4];
        for f in 0..self.frames.len() {
            let px = self.frame_rgba(f, pal);
            for y in 0..h {
                let dst = (y * w * n + f * w) * 4;
                out[dst..dst + w * 4].copy_from_slice(&px[y * w * 4..(y + 1) * w * 4]);
            }
        }
        ((w * n) as u32, h as u32, out)
    }
}

/// Encodes RGBA pixels as PNG.
pub fn encode_png(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w.max(1), h.max(1));
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().expect("png header");
        if w == 0 || h == 0 {
            wr.write_image_data(&[0, 0, 0, 0]).expect("png data");
        } else {
            wr.write_image_data(rgba).expect("png data");
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A 2x3 single-frame image: column 0 uncompressed [5,6] at rows 1..3,
    /// column 1 compressed: 3 copies of colour 7.
    pub fn sample_gfx() -> Vec<u8> {
        let mut d = vec![0u8; 0x12];
        d[0] = 2; // width
        d[2] = 3; // height
        d[6] = 1; // frames
        let frame: Vec<u8> = vec![
            1, // clear
            0x00, 4, 2, 1, 5, 6, // column 0: total 4, run 2, skip 1, pixels
            0x80, 4, 2, 0, 0xe2, 7, // column 1: total 4, run 2 (both bytes), 3 x colour 7
        ];
        let start = 0x12 + 8;
        d.extend((start as u32).to_le_bytes());
        d.extend(((start + frame.len()) as u32).to_le_bytes());
        d.extend(frame);
        d
    }

    #[test]
    fn decodes_columns_and_rle() {
        let img = decode(&sample_gfx()).unwrap();
        assert_eq!(img.header.width, 2);
        assert_eq!(img.frames.len(), 1);
        let t = TRANSPARENT;
        assert_eq!(img.frames[0], vec![t, 7, 5, 7, 6, 7]);
    }

    #[test]
    fn renders_rgba_and_png() {
        let img = decode(&sample_gfx()).unwrap();
        let pal = Palette::grayscale();
        let (w, h, px) = img.sheet_rgba(&pal);
        assert_eq!((w, h), (2, 3));
        assert_eq!(&px[0..8], &[0, 0, 0, 0, 7, 7, 7, 255]);
        let png = encode_png(w, h, &px);
        assert_eq!(&png[1..4], b"PNG");
    }
}
