//! Types of the generated tables in `data.rs` (1oom `game_shiptech.h`). Per-hull arrays
//! are indexed by hull: small, medium, large, huge.

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Weapon {
    pub key: &'static str,
    /// `damage_min != damage_max` for beams and bombs, equal for missiles.
    pub damage_min: u16,
    pub damage_max: u16,
    pub range: u8,
    pub extra_acc: u8,
    pub halve_shield: bool,
    pub bomb: bool,
    pub damage_fade: bool,
    pub miss_type: u8,
    pub damage_mul: u8,
    /// Shots per combat turn.
    pub fire: u8,
    /// Shots per battle, -1 = unlimited.
    pub shots: i8,
    /// BC x 10.
    pub cost: u16,
    pub space: u16,
    pub power: u16,
    pub bio: bool,
    /// Weapon tech number that unlocks it.
    pub tech: u8,
    /// Missiles: warheads per launch; beams: streaming.
    pub streams: u8,
}

impl Weapon {
    pub fn is_missile(&self) -> bool {
        self.damage_min == self.damage_max && !self.bio && self.key != "none"
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Comp {
    pub key: &'static str,
    pub power: [u16; 4],
    pub space: [u16; 4],
    pub cost: [u16; 4],
    pub tech: u8,
    pub level: u8,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Engine {
    pub key: &'static str,
    pub power: u16,
    pub space: u16,
    pub cost: u16,
    pub warp: u8,
    pub tech: u8,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Armor {
    pub key: &'static str,
    pub cost: [u16; 4],
    pub space: [u16; 4],
    /// Hit points in percent of the hull's.
    pub armor: u16,
    pub tech: u8,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Shield {
    pub key: &'static str,
    pub cost: [u16; 4],
    pub space: [u16; 4],
    pub power: [u16; 4],
    pub absorb: u8,
    pub tech: u8,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Special {
    pub key: &'static str,
    pub cost: [u16; 4],
    pub space: [u16; 4],
    pub power: [u16; 4],
    pub tech: u8,
    /// Research field of `tech`.
    pub field: u8,
    /// Group: 1 fuel tanks, 2 colony bases, 3 scanner, ... (only one per group).
    pub kind: u8,
    pub repair: u8,
    pub extra_man: u8,
    pub missile_shield: u8,
    pub extra_range: u8,
    pub pulsar: u8,
    pub stream: u8,
    /// Bit flags of 1oom `ship_special_bool_i_t`.
    pub flags: u16,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Hull {
    pub key: &'static str,
    /// BC x 10.
    pub cost: u16,
    pub space: u16,
    pub hits: u16,
    pub power: u16,
    pub defense: i16,
}

/// 1oom `tech_reduce_50percent_per_10pts`: cost in percent, by tech levels above the part.
pub const REDUCE_50: [u8; 51] = [
    100, 93, 87, 81, 76, 71, 66, 62, 58, 54, 50, 47, 44, 41, 38, 35, 33, 31, 29, 27, 25, 23, 22,
    20, 19, 18, 16, 15, 14, 13, 13, 12, 11, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 5, 4, 4, 4, 4, 3, 3,
];

/// 1oom `tech_reduce_25percent_per_10pts`: space in percent, by tech levels above the part.
pub const REDUCE_25: [u8; 51] = [
    100, 97, 94, 92, 89, 87, 84, 82, 79, 77, 75, 73, 71, 69, 67, 65, 63, 61, 60, 58, 56, 55, 53,
    52, 50, 49, 47, 46, 45, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 27,
    26, 25, 24, 24,
];
