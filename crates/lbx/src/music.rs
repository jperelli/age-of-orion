//! MUSIC.LBX tunes: XMIDI to standard MIDI (1oom `fmt_mus.c`, `fmt_mus_convert_xmid`).
//!
//! XMIDI stores each note-on with its duration; MIDI needs explicit note-offs, so they are
//! queued and emitted in time order. AIL loop markers end the tune (and mark it as looping)
//! and tempo events are dropped, as MOO1 ignores them (60 ticks per quarter at 120 bpm).

/// Entry signature (`0xdeaf 1`, 1oom `HDRID_LBXXMID`).
pub const XMID_ID: [u8; 4] = [0xaf, 0xde, 0x01, 0x00];
const TICKS_PER_Q: u16 = 60;

pub fn is_xmid(d: &[u8]) -> bool {
    d.len() >= 32 && d[..4] == XMID_ID
}

fn be32(d: &[u8], at: usize) -> Option<usize> {
    d.get(at..at + 4)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
}

/// 1oom `xmid_encode_delta_time`.
fn delta(out: &mut Vec<u8>, mut t: u32) {
    let mut v = t & 0x7f;
    loop {
        t >>= 7;
        if t == 0 {
            break;
        }
        v <<= 8;
        v |= (t & 0x7f) | 0x80;
    }
    loop {
        out.push((v & 0xff) as u8);
        if v & 0x80 != 0 {
            v >>= 8;
        } else {
            break;
        }
    }
}

struct NoteOff {
    t: u32,
    ch: u8,
    note: u8,
}

/// Converts an XMIDI entry; returns the MIDI file and whether the tune loops.
pub fn convert_xmid(d: &[u8]) -> Option<(Vec<u8>, bool)> {
    if !is_xmid(d) || d.len() < 0x4e || &d[0x10..0x14] != b"FORM" || &d[0x3e..0x42] != b"TIMB" {
        return None;
    }
    let len_timbre = be32(d, 0x42)?;
    if d.len() < 0x4e + len_timbre || d.get(0x46 + len_timbre..0x4a + len_timbre)? != b"EVNT" {
        return None;
    }
    let len_evnt = be32(d, 0x4a + len_timbre)?;
    let start = 0x4e + len_timbre;
    let evnt = d.get(start..(start + len_evnt).min(d.len()))?;
    let (track, loops) = convert_events(evnt)?;
    let mut out = Vec::with_capacity(track.len() + 22);
    out.extend(b"MThd");
    out.extend([0, 0, 0, 6, 0, 0, 0, 1]);
    out.extend(TICKS_PER_Q.to_be_bytes());
    out.extend(b"MTrk");
    out.extend((track.len() as u32).to_be_bytes());
    out.extend(track);
    Some((out, loops))
}

/// 1oom `xmid_convert_evnt`.
fn convert_events(data: &[u8]) -> Option<(Vec<u8>, bool)> {
    let mut out = Vec::with_capacity(data.len() * 2);
    let mut pending: Vec<NoteOff> = Vec::new();
    let mut loops = false;
    let mut looppoint: i32 = -1;
    let mut noteons = 0;
    let (mut t_now, mut delta_time) = (0u32, 0u32);
    let mut i = 0;
    let mut end_found = false;
    while i < data.len() && !end_found {
        let st = data[i];
        let mut add_event = true;
        let mut len_event = 0;
        let mut skip = 0;
        let mut extra: Vec<u8> = Vec::new();
        match st & 0xf0 {
            0x90 => {
                let mut dt_off = 0u32;
                skip = 1;
                loop {
                    let b = *data.get(i + 2 + skip)?;
                    if b & 0x80 == 0 {
                        dt_off |= b as u32;
                        break;
                    }
                    dt_off |= (b & 0x7f) as u32;
                    dt_off <<= 7;
                    skip += 1;
                }
                // Sorted insertion; equal times keep their order.
                let t = t_now + delta_time + dt_off;
                let pos = pending
                    .iter()
                    .position(|n| t < n.t)
                    .unwrap_or(pending.len());
                pending.insert(
                    pos,
                    NoteOff {
                        t,
                        ch: st & 0x8f,
                        note: *data.get(i + 1)?,
                    },
                );
                noteons += 1;
                len_event = 3;
            }
            0xb0 => {
                len_event = 3;
                let c = *data.get(i + 1)?;
                if (0x20..=0x2e).contains(&c)
                    || (0x3a..=0x3f).contains(&c)
                    || (0x6e..=0x78).contains(&c)
                {
                    add_event = false;
                    len_event = 0;
                    skip = 3;
                }
                match c {
                    0x74 => {
                        if looppoint < 0 {
                            looppoint = noteons;
                        }
                    }
                    0x75 => {
                        loops = looppoint >= 0;
                        len_event = 0;
                        end_found = true;
                        add_event = true;
                        extra = vec![0xff, 0x2f, 0x00];
                    }
                    _ => {}
                }
            }
            0xa0 | 0xe0 => len_event = 3,
            0x80 => return None,
            0xc0 | 0xd0 => len_event = 2,
            0xf0 => match st & 0x0f {
                0x08 | 0x0a | 0x0b | 0x0c => len_event = 1,
                0x0f => {
                    len_event = 3 + *data.get(i + 2)? as usize;
                    match data.get(i + 1)? {
                        0x2f => end_found = true,
                        0x51 => {
                            add_event = false;
                            skip = len_event;
                            len_event = 0;
                        }
                        _ => {}
                    }
                }
                _ => return None,
            },
            _ => {
                add_event = false;
                while i < data.len() && data[i] & 0x80 == 0 {
                    delta_time += data[i] as u32;
                    i += 1;
                }
            }
        }
        if add_event {
            while let Some(n) = pending.first() {
                if t_now + delta_time < n.t {
                    break;
                }
                let n = pending.remove(0);
                let d_off = n.t - t_now;
                delta(&mut out, d_off);
                out.extend([n.ch, n.note, 0]);
                delta_time -= d_off;
                t_now += d_off;
            }
            t_now += delta_time;
            delta(&mut out, delta_time);
            delta_time = 0;
            if end_found {
                for n in pending.drain(..) {
                    out.extend([n.ch, n.note, 0, 0]);
                }
            }
            out.extend(&extra);
            out.extend(data.get(i..i + len_event)?);
            i += len_event;
        }
        i += skip;
    }
    Some((out, loops))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xmid(evnt: &[u8]) -> Vec<u8> {
        let mut d = vec![0u8; 0x10];
        d[..4].copy_from_slice(&XMID_ID);
        d.extend(b"FORM");
        d.resize(0x3e, 0);
        d.extend(b"TIMB");
        d.extend(2u32.to_be_bytes());
        d.extend([0, 0]);
        d.extend(b"EVNT");
        d.extend((evnt.len() as u32).to_be_bytes());
        d.extend(evnt);
        d
    }

    #[test]
    fn note_offs_are_inserted() {
        // Note on C4 lasting 10 ticks, wait 20, end of track.
        let d = xmid(&[0x90, 60, 100, 10, 20, 0xff, 0x2f, 0x00]);
        let (midi, loops) = convert_xmid(&d).unwrap();
        assert!(!loops);
        assert_eq!(&midi[..4], b"MThd");
        let track = &midi[22..];
        assert_eq!(
            track,
            &[0, 0x90, 60, 100, 10, 0x80, 60, 0, 10, 0xff, 0x2f, 0x00]
        );
    }

    #[test]
    fn loop_markers_end_the_tune() {
        let d = xmid(&[0xb0, 0x74, 0, 0x90, 60, 100, 5, 0xb0, 0x75, 0]);
        let (midi, loops) = convert_xmid(&d).unwrap();
        assert!(loops);
        assert_eq!(&midi[midi.len() - 3..], &[0xff, 0x2f, 0x00]);
    }
}
