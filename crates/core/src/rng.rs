use serde::{Deserialize, Serialize};

/// The pseudo random generator of MOO1 (port of 1oom `rnd_0_nm1`): a 32-bit linear
/// feedback shift register stepped 9 times per call. Keeping it identical makes galaxy
/// generation comparable with the original for the same seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng(pub u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 123 } else { seed })
    }

    /// Uniform-ish value in `0..n` (`n` > 0).
    pub fn below(&mut self, n: u16) -> u16 {
        let mut r: u16 = 0;
        let mut di_si = self.0;
        for _ in 0..9 {
            let mut ax = di_si as u16;
            let dx;
            {
                let mut dx_bx = di_si;
                dx_bx >>= 1;
                ax ^= dx_bx as u16;
                dx_bx >>= 1;
                ax ^= dx_bx as u16;
                dx_bx >>= 2;
                ax ^= dx_bx as u16;
                dx_bx >>= 2;
                ax ^= dx_bx as u16;
                let d = (dx_bx >> 25) as u16;
                let al = (ax & 0xff) as u8 ^ (d as u8);
                ax = (ax & 0xff00) | al as u16;
                dx = ax;
            }
            r <<= 1;
            if dx & 1 != 0 {
                r += 1;
            }
            di_si >>= 1;
            if ax & 1 != 0 {
                di_si = di_si.wrapping_add(0x8000_0000);
            }
        }
        if di_si == 0 {
            di_si = 0x30be;
        }
        self.0 = di_si;
        r % n.max(1)
    }

    /// Value in `1..=n`.
    pub fn one_to(&mut self, n: u16) -> u16 {
        1 + self.below(n)
    }

    pub fn chance(&mut self, percent: u16) -> bool {
        self.below(100) < percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_in_range() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            let v = a.below(20);
            assert_eq!(v, b.below(20));
            assert!(v < 20);
        }
        let mut c = Rng::new(7);
        assert!((0..200).all(|_| (1..=6).contains(&c.one_to(6))));
    }

    #[test]
    fn spreads_values() {
        let mut r = Rng::new(1234);
        let mut seen = [0u32; 10];
        for _ in 0..5000 {
            seen[r.below(10) as usize] += 1;
        }
        assert!(seen.iter().all(|&n| n > 300), "{seen:?}");
    }
}
