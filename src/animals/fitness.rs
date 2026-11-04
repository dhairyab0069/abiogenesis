//! How well a body does in a zone: six factors, each from 0 to 1 or so, multiplied.
//!
//! - **food:** a gut, jaws, senses and speed all help find and eat food, and a zone
//!   has only so much of it;
//! - **oxygen:** skin, gills and lungs supply it, and muscle, nerves and bone use it;
//! - **support:** on land nothing floats you, so a skeleton and limbs carry the weight;
//! - **drying:** a sealed skin keeps water in;
//! - **predation:** speed, senses and a hard body help you get away;
//! - **upkeep:** every organ costs something to run.
//!
//! None of the numbers is a measurement. The *shape* of the trade-offs is the point:
//! fins and gills win in water, a lung-like organ wins where water is short of oxygen
//! and air is close, and limbs, a sealed skin and lungs win on land.

use super::traits::{BILATERAL_UPKEEP, Organ, Traits};
use super::zone::{Environment, Zone};

/// The factors behind a fitness, kept apart so they can be shown and tested.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fitness {
    pub food: f32,
    pub oxygen: f32,
    pub support: f32,
    pub drying: f32,
    pub predation: f32,
    pub upkeep: f32,
}

impl Fitness {
    pub fn total(&self) -> f32 {
        self.food * self.oxygen * self.support * self.drying * self.predation * self.upkeep
    }
}

/// How fast and how steadily the animal can move in `zone`, 0 to 1.
pub fn speed(t: &Traits, zone: Zone) -> f32 {
    // fins push on water, limbs push on the ground
    let (fins, limbs) = [(0.9, 0.15), (0.8, 0.5), (0.35, 0.85), (0.03, 1.0)][zone.index()];
    let muscle = t.get(Organ::Muscle);
    let wriggle = if t.bilateral { 0.1 * muscle } else { 0.0 };
    let push = (wriggle + t.get(Organ::Fins) * fins + t.get(Organ::Limbs) * limbs).min(1.0);
    let direction = if t.bilateral { 1.0 } else { 0.3 };
    push * (0.25 + 0.75 * muscle) * direction
}

/// How much the animal can sense of its surroundings: needs a nerve to carry it.
pub fn sense(t: &Traits) -> f32 {
    t.get(Organ::Nerve) * (0.4 + 0.6 * t.get(Organ::Eyes))
}

pub fn fitness(t: &Traits, zone: Zone, env: &Environment) -> Fitness {
    let g = |o| t.get(o);
    let speed = speed(t, zone);
    let sense = sense(t);

    // food: a floating radial animal is a decent filter feeder in water
    let sessile = if !t.bilateral && zone.is_water() {
        0.1
    } else {
        0.0
    };
    let head_first = if t.bilateral { 0.06 } else { 0.0 };
    let food = zone.food()
        * (0.2
            + 0.25 * g(Organ::Gut)
            + 0.2 * g(Organ::Jaws)
            + 0.25 * sense
            + 0.4 * speed
            + sessile
            + head_first);

    // oxygen
    let medium = env.medium_oxygen(zone);
    let skin_gas = 0.45 * medium * (1.0 - 0.7 * g(Organ::Skin));
    let gill_gas = 0.9 * g(Organ::Gills) * medium * zone.gill_working();
    let lung_gas = 0.8 * g(Organ::Lungs) * zone.air_access();
    let need = 0.25
        + 0.3 * g(Organ::Muscle)
        + 0.15 * g(Organ::Nerve)
        + 0.1 * g(Organ::Eyes)
        + 0.1 * g(Organ::Skeleton);
    let oxygen = ((skin_gas + gill_gas + lung_gas) / need).clamp(0.02, 1.0);

    // support: the weight the surroundings do not carry has to be carried by the body
    let load = 1.0 - zone.support();
    let carry = (0.15 + 0.4 * g(Organ::Skeleton) + 0.45 * g(Organ::Limbs)).min(1.0);
    let support = 1.0 - 0.95 * load * (1.0 - carry);

    let drying = 1.0 - 0.9 * zone.drying() * (1.0 - g(Organ::Skin));

    let defence = 0.3 * g(Organ::Skeleton) + 0.35 * speed + 0.35 * sense;
    let predation = 1.0 - 0.55 * env.predation(zone) * (1.0 - defence.min(1.0));

    let cost: f32 = Organ::ALL.iter().map(|o| o.upkeep() * g(*o)).sum::<f32>()
        + if t.bilateral { BILATERAL_UPKEEP } else { 0.0 };
    let upkeep = 1.0 - 0.6 * cost;

    Fitness {
        food,
        oxygen,
        support,
        drying,
        predation,
        upkeep,
    }
}
