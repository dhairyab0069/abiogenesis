//! The shoreline: four zones from open sea to dry land, and the three dials the
//! user can turn (the "what if" hook).
//!
//! The numbers are made up to tell a qualitative story (water holds little oxygen,
//! air a lot; water holds a body up, air does not; the shallows are warm and rich
//! in food; the tidal flats dry out twice a day). They are *not* measurements.

/// A stretch of the shore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    OpenSea,
    Shallows,
    TidalFlats,
    Land,
}

pub const ZONES: usize = 4;

impl Zone {
    pub const ALL: [Zone; ZONES] = [Zone::OpenSea, Zone::Shallows, Zone::TidalFlats, Zone::Land];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Zone {
        Self::ALL[i.min(ZONES - 1)]
    }

    pub fn label(self) -> &'static str {
        match self {
            Zone::OpenSea => "Open sea",
            Zone::Shallows => "Shallows",
            Zone::TidalFlats => "Tidal flats",
            Zone::Land => "Land",
        }
    }

    /// The zone one step toward the sea or toward the land, if there is one.
    pub fn neighbour(self, landward: bool) -> Option<Zone> {
        let i = self.index();
        if landward {
            Self::ALL.get(i + 1).copied()
        } else {
            i.checked_sub(1).map(Self::from_index)
        }
    }

    pub fn is_water(self) -> bool {
        self != Zone::Land
    }

    /// How much water is around, from full water (1) to none (0): on the flats the tide goes out.
    pub fn wet(self) -> f32 {
        [1.0, 1.0, 0.45, 0.0][self.index()]
    }

    /// How much a gill still works here (gills need water over them).
    pub fn gill_working(self) -> f32 {
        [1.0, 1.0, 0.6, 0.05][self.index()]
    }

    /// How easily an animal can reach air: not at all in deep water, a gulp at the
    /// surface of the shallows, mostly on the flats, fully on land.
    pub fn air_access(self) -> f32 {
        [0.0, 0.5, 0.8, 1.0][self.index()]
    }

    /// How much the surroundings hold a body up (water floats it, air does not).
    pub fn support(self) -> f32 {
        [0.95, 0.9, 0.65, 0.0][self.index()]
    }

    /// The risk of drying out.
    pub fn drying(self) -> f32 {
        [0.0, 0.03, 0.45, 1.0][self.index()]
    }

    /// How much there is to eat. Early land is thinner than the shallows.
    pub fn food(self) -> f32 {
        [0.6, 1.0, 0.85, 0.7][self.index()]
    }

    /// The share of the predators' pressure that reaches here: big hunters live in
    /// open water, and can hardly follow into the shallows, let alone onto land.
    pub fn predation(self) -> f32 {
        [1.0, 0.55, 0.25, 0.08][self.index()]
    }

    /// How oxygen-rich the shallow water is compared with the open sea (warm, still water holds less).
    fn water_oxygen_share(self) -> f32 {
        [1.0, 0.55, 0.3, 0.0][self.index()]
    }
}

/// Oxygen in air, on the same 0 to 1 scale as water oxygen: air holds far more than water.
pub const AIR_OXYGEN: f32 = 1.0;

/// The three dials: the hypothetical world the lineage evolves in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    /// How much oxygen the open sea holds, 0.35 (a stale ocean) to 1.
    pub oxygen: f32,
    /// How steep the shore is, 0 (a very gentle slope, wide shallows and flats) to 1 (a cliff, almost none).
    pub slope: f32,
    /// How hard predators hunt, 0 to 1.
    pub predators: f32,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            oxygen: 0.7,
            slope: 0.5,
            predators: 0.5,
        }
    }
}

impl Environment {
    pub const MIN_OXYGEN: f32 = 0.35;

    /// The same environment with every dial inside its range.
    pub fn clamped(self) -> Self {
        Self {
            oxygen: self.oxygen.clamp(Self::MIN_OXYGEN, 1.0),
            slope: self.slope.clamp(0.0, 1.0),
            predators: self.predators.clamp(0.0, 1.0),
        }
    }

    /// Oxygen in the medium around an animal in `zone`: water oxygen at sea, air on land.
    pub fn medium_oxygen(&self, zone: Zone) -> f32 {
        if zone.is_water() {
            self.oxygen * zone.water_oxygen_share()
        } else {
            AIR_OXYGEN
        }
    }

    /// Predator pressure in `zone`.
    pub fn predation(&self, zone: Zone) -> f32 {
        self.predators * zone.predation()
    }

    /// The share of the shore each zone takes, adding up to 1. A steep shore leaves
    /// little room for shallows and flats.
    pub fn shares(&self) -> [f32; ZONES] {
        let gentle = 1.0 - self.slope.clamp(0.0, 1.0);
        let raw = [1.0, 0.25 + 0.9 * gentle, 0.12 + 0.9 * gentle, 0.9];
        let total: f32 = raw.iter().sum();
        raw.map(|r| r / total)
    }
}
