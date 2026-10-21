//! Enumerations of the MOO1 rules (same order as 1oom so tables port unchanged).

use serde::{Deserialize, Serialize};

pub type EmpireId = u8;
pub type PlanetId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanetType {
    NotHabitable,
    Radiated,
    Toxic,
    Inferno,
    Dead,
    Tundra,
    Barren,
    Minimal,
    Desert,
    Steppe,
    Arid,
    Ocean,
    Jungle,
    Terran,
}

impl PlanetType {
    pub const ALL: [PlanetType; 14] = [
        PlanetType::NotHabitable,
        PlanetType::Radiated,
        PlanetType::Toxic,
        PlanetType::Inferno,
        PlanetType::Dead,
        PlanetType::Tundra,
        PlanetType::Barren,
        PlanetType::Minimal,
        PlanetType::Desert,
        PlanetType::Steppe,
        PlanetType::Arid,
        PlanetType::Ocean,
        PlanetType::Jungle,
        PlanetType::Terran,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i.min(13)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StarType {
    Yellow,
    Red,
    Green,
    White,
    Blue,
    Neutron,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Special {
    UltraPoor,
    Poor,
    Normal,
    Artifacts,
    Rich,
    UltraRich,
    /// Orion: research output x4.
    Orion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Growth {
    Hostile,
    Normal,
    Fertile,
    Gaia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    Human,
    Mrrshan,
    Silicoid,
    Sakkra,
    Psilon,
    Alkari,
    Klackon,
    Bulrathi,
    Meklar,
    Darlok,
}

impl Race {
    pub const ALL: [Race; 10] = [
        Race::Human,
        Race::Mrrshan,
        Race::Silicoid,
        Race::Sakkra,
        Race::Psilon,
        Race::Alkari,
        Race::Klackon,
        Race::Bulrathi,
        Race::Meklar,
        Race::Darlok,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GalaxySize {
    #[default]
    Small,
    Medium,
    Large,
    Huge,
}

impl GalaxySize {
    pub const ALL: [GalaxySize; 4] = [
        GalaxySize::Small,
        GalaxySize::Medium,
        GalaxySize::Large,
        GalaxySize::Huge,
    ];
}

impl GalaxySize {
    /// Star grid of 1oom `game_generate_galaxy`.
    pub fn grid(self) -> (u16, u16) {
        match self {
            GalaxySize::Small => (6, 4),
            GalaxySize::Medium => (8, 6),
            GalaxySize::Large => (10, 7),
            GalaxySize::Huge => (12, 9),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Simple,
    #[default]
    Easy,
    Average,
    Hard,
    Impossible,
}

impl Difficulty {
    pub const ALL: [Difficulty; 5] = [
        Difficulty::Simple,
        Difficulty::Easy,
        Difficulty::Average,
        Difficulty::Hard,
        Difficulty::Impossible,
    ];
}

/// Research fields, 1oom order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechField {
    Computer,
    Construction,
    ForceField,
    Planetology,
    Propulsion,
    Weapon,
}

impl TechField {
    pub const ALL: [TechField; 6] = [
        TechField::Computer,
        TechField::Construction,
        TechField::ForceField,
        TechField::Planetology,
        TechField::Propulsion,
        TechField::Weapon,
    ];
}

/// Colony spending sliders, 1oom order.
pub const SLIDER_SHIP: usize = 0;
pub const SLIDER_DEF: usize = 1;
pub const SLIDER_IND: usize = 2;
pub const SLIDER_ECO: usize = 3;
pub const SLIDER_TECH: usize = 4;
pub const NUM_SLIDERS: usize = 5;

pub const NUM_FIELDS: usize = 6;

/// Field indices (1oom `tech_field_t`).
pub const FIELD_COMPUTER: usize = 0;
pub const FIELD_CONSTRUCTION: usize = 1;
pub const FIELD_FORCE_FIELD: usize = 2;
pub const FIELD_PLANETOLOGY: usize = 3;
pub const FIELD_PROPULSION: usize = 4;
pub const FIELD_WEAPON: usize = 5;
