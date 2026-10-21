//! Sound effects: MOO1's Creative Voice samples (SOUNDFX.LBX, INTROSND.LBX) to WAV, as
//! 1oom `fmt_sfx.c` reads them (8-bit unsigned mono PCM).

const LBXVOC: [u8; 4] = [0xaf, 0xde, 0x02, 0x00];
const VOC: &[u8; 0x1a] = b"Creative Voice File\x1a\x1a\x00\x0a\x01\x29\x11";

/// Whether an LBX entry is a sound effect.
pub fn is_voc(d: &[u8]) -> bool {
    (d.len() > 0x10 + VOC.len() && d[..4] == LBXVOC) || d.starts_with(VOC)
}

fn le(d: &[u8], at: usize, n: usize) -> usize {
    (0..n).fold(0, |v, i| {
        v | (d.get(at + i).copied().unwrap_or(0) as usize) << (8 * i)
    })
}

/// Converts an LBX sound entry (or a bare VOC file) to a WAV file.
pub fn voc_to_wav(d: &[u8]) -> Option<Vec<u8>> {
    let d = if d[..4] == LBXVOC { &d[0x10..] } else { d };
    if !d.starts_with(VOC) {
        return None;
    }
    let mut p = VOC.len();
    let mut rate = 0u32;
    let mut pcm: Vec<u8> = Vec::new();
    while p < d.len() {
        let kind = d[p];
        p += 1;
        if kind == 0 {
            break;
        }
        if kind > 8 || p + 3 > d.len() {
            return None;
        }
        let size = le(d, p, 3);
        p += 3;
        let block = d.get(p..p + size)?;
        p += size;
        match kind {
            1 => {
                let r = 1_000_000 / (256 - block[0] as u32);
                if rate == 0 {
                    rate = r;
                }
                if block[1] != 0 {
                    return None;
                }
                pcm.extend(&block[2..]);
            }
            2 => pcm.extend(block),
            3 => {
                let n = le(block, 0, 2) + 1;
                if rate == 0 {
                    rate = 1_000_000 / (256 - block[2] as u32);
                }
                pcm.extend(std::iter::repeat_n(0x80u8, n));
            }
            8 => {
                let tc = le(block, 0, 2) as u32;
                let stereo = block[3] != 0;
                if rate == 0 {
                    rate = if stereo { 128_000_000 } else { 256_000_000 } / (65536 - tc);
                }
            }
            _ => {}
        }
    }
    if rate == 0 || pcm.is_empty() {
        return None;
    }
    let mut w = Vec::with_capacity(44 + pcm.len());
    w.extend(b"RIFF");
    w.extend((36 + pcm.len() as u32).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(8u16.to_le_bytes());
    w.extend(b"data");
    w.extend((pcm.len() as u32).to_le_bytes());
    w.extend(pcm);
    Some(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_a_voc_sample() {
        let mut d = LBXVOC.to_vec();
        d.resize(0x10, 0);
        d.extend(VOC);
        // Sound data block: rate byte 156 (10 kHz), 8-bit, 4 samples.
        d.extend([1, 6, 0, 0, 156, 0, 0x80, 0xff, 0x00, 0x80, 0]);
        assert!(is_voc(&d));
        let w = voc_to_wav(&d).unwrap();
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes([w[24], w[25], w[26], w[27]]), 10_000);
        assert_eq!(&w[44..], &[0x80, 0xff, 0x00, 0x80]);
    }
}
