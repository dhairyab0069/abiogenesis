//! The environment of the soup: temperature, pH and wetness, optionally cycling
//! over time (day and night, seasons, tides, wet/dry pools).
//!
//! Design choice: the environment is a pure function of the step number. It holds
//! no mutable state, so the same `(Environment, step)` always gives the same
//! `Conditions`. That makes it trivial to test, to replay a run for the time-lapse
//! in stage 5, and to compare two scenarios step by step.

use std::f64::consts::TAU;

/// One repeating swing around a base value: `base + amplitude * sin(...)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cycle {
    /// Steps for one full swing. Must be positive.
    pub period: f64,
    /// How far above and below the base value it swings.
    pub amplitude: f64,
    /// Where in the swing step 0 starts, as a fraction of a period (0.25 = at the peak).
    pub phase: f64,
}

impl Cycle {
    pub fn new(period: f64, amplitude: f64) -> Self {
        Self {
            period,
            amplitude,
            phase: 0.0,
        }
    }

    /// The offset from the base value at `step`, between `-amplitude` and `+amplitude`.
    pub fn offset(&self, step: u64) -> f64 {
        if self.period <= 0.0 {
            return 0.0;
        }
        self.amplitude * (TAU * (step as f64 / self.period + self.phase)).sin()
    }
}

/// What the soup feels at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Conditions {
    /// Kelvin.
    pub temperature: f64,
    pub ph: f64,
    /// 0.0 = fully dried out, 1.0 = fully wet.
    pub wetness: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    pub base_temperature: f64,
    pub base_ph: f64,
    pub base_wetness: f64,
    pub temperature_cycle: Option<Cycle>,
    pub ph_cycle: Option<Cycle>,
    pub wetness_cycle: Option<Cycle>,
}

impl Environment {
    /// A constant environment with no cycles, matching the old fixed behaviour.
    pub fn constant(temperature: f64, ph: f64) -> Self {
        Self {
            base_temperature: temperature,
            base_ph: ph,
            base_wetness: 1.0,
            temperature_cycle: None,
            ph_cycle: None,
            wetness_cycle: None,
        }
    }

    /// The conditions at `step`. Each value is clamped to something physical, so
    /// a large amplitude can never produce a negative temperature or a pH of 20.
    pub fn at(&self, step: u64) -> Conditions {
        let swing = |base: f64, cycle: &Option<Cycle>| base + cycle.map_or(0.0, |c| c.offset(step));
        Conditions {
            temperature: swing(self.base_temperature, &self.temperature_cycle).max(1.0),
            ph: swing(self.base_ph, &self.ph_cycle).clamp(0.0, 14.0),
            wetness: swing(self.base_wetness, &self.wetness_cycle).clamp(0.0, 1.0),
        }
    }
}

/// Steps in one day. The app runs 60 steps a second, so a day lasts 20 seconds.
pub const DAY_STEPS: f64 = 1200.0;
/// Steps in one wet/dry cycle of the pool (40 seconds in the app).
pub const POOL_STEPS: f64 = 2400.0;

/// Ready-made environments for the controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preset {
    /// Nothing changes: the stage 1 soup.
    #[default]
    Constant,
    /// Warmer days, cooler nights, and a pH that swings the other way.
    DayNight,
    /// The pool dries out and refills.
    WetDry,
    /// Both at once.
    Both,
}

impl Preset {
    pub const ALL: [Preset; 4] = [
        Preset::Constant,
        Preset::DayNight,
        Preset::WetDry,
        Preset::Both,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Constant => "Constant",
            Preset::DayNight => "Day and night",
            Preset::WetDry => "Wet and dry pools",
            Preset::Both => "Day and night + pools",
        }
    }
}

impl Environment {
    /// A preset around the given base temperature (K) and pH. The pool starts
    /// full, so a wet/dry run begins wet and dries out.
    pub fn preset(preset: Preset, temperature: f64, ph: f64) -> Self {
        let mut env = Self::constant(temperature, ph);
        if matches!(preset, Preset::DayNight | Preset::Both) {
            env.temperature_cycle = Some(Cycle::new(DAY_STEPS, 25.0));
            env.ph_cycle = Some(Cycle {
                period: DAY_STEPS,
                amplitude: 0.6,
                phase: 0.5,
            });
        }
        if matches!(preset, Preset::WetDry | Preset::Both) {
            env.base_wetness = 0.55;
            env.wetness_cycle = Some(Cycle {
                period: POOL_STEPS,
                amplitude: 0.5,
                phase: 0.25,
            });
        }
        env
    }
}

impl Default for Environment {
    /// 300 K, pH 7, fully wet: the standard soup.
    fn default() -> Self {
        Self::constant(300.0, 7.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_environment_never_changes() {
        let env = Environment::constant(310.0, 6.5);
        let first = env.at(0);
        assert_eq!(
            first,
            Conditions {
                temperature: 310.0,
                ph: 6.5,
                wetness: 1.0
            }
        );
        assert!((0..1000).all(|t| env.at(t) == first));
    }

    #[test]
    fn cycle_repeats_every_period() {
        let c = Cycle::new(100.0, 20.0);
        for t in [0, 7, 33, 80] {
            assert!((c.offset(t) - c.offset(t + 100)).abs() < 1e-9);
        }
    }

    #[test]
    fn cycle_peaks_at_quarter_period_and_stays_within_amplitude() {
        let c = Cycle::new(100.0, 20.0);
        assert!((c.offset(25) - 20.0).abs() < 1e-9);
        assert!((c.offset(75) + 20.0).abs() < 1e-9);
        assert!((0..400).all(|t| c.offset(t).abs() <= 20.0 + 1e-9));
    }

    #[test]
    fn phase_shifts_the_start() {
        let c = Cycle {
            period: 100.0,
            amplitude: 5.0,
            phase: 0.25,
        };
        assert!((c.offset(0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn conditions_are_clamped_to_physical_ranges() {
        let env = Environment {
            temperature_cycle: Some(Cycle::new(10.0, 1000.0)),
            ph_cycle: Some(Cycle::new(10.0, 50.0)),
            wetness_cycle: Some(Cycle::new(10.0, 5.0)),
            ..Environment::default()
        };
        for t in 0..40 {
            let c = env.at(t);
            assert!(c.temperature >= 1.0);
            assert!((0.0..=14.0).contains(&c.ph));
            assert!((0.0..=1.0).contains(&c.wetness));
        }
    }

    #[test]
    fn zero_period_is_ignored_instead_of_dividing_by_zero() {
        assert_eq!(Cycle::new(0.0, 10.0).offset(5), 0.0);
    }

    #[test]
    fn constant_preset_matches_constant_environment() {
        assert_eq!(
            Environment::preset(Preset::Constant, 310.0, 6.5),
            Environment::constant(310.0, 6.5)
        );
    }

    #[test]
    fn day_night_swings_temperature_and_ph_in_opposite_directions() {
        let env = Environment::preset(Preset::DayNight, 300.0, 7.0);
        let (day, night) = (
            env.at((DAY_STEPS / 4.0) as u64),
            env.at((DAY_STEPS * 3.0 / 4.0) as u64),
        );
        assert!(day.temperature > 320.0 && night.temperature < 280.0);
        assert!(day.ph < 6.6 && night.ph > 7.4);
        assert_eq!(day.wetness, 1.0);
    }

    #[test]
    fn wet_dry_pool_starts_full_and_dries_out() {
        let env = Environment::preset(Preset::WetDry, 300.0, 7.0);
        assert_eq!(env.at(0).wetness, 1.0);
        let driest = env.at((POOL_STEPS / 2.0) as u64).wetness;
        assert!(driest < 0.1, "driest point was {driest}");
        assert_eq!(env.at(0).temperature, 300.0);
    }

    #[test]
    fn both_combines_the_cycles() {
        let env = Environment::preset(Preset::Both, 300.0, 7.0);
        assert!(
            env.temperature_cycle.is_some()
                && env.ph_cycle.is_some()
                && env.wetness_cycle.is_some()
        );
    }

    #[test]
    fn same_step_gives_same_conditions() {
        let env = Environment {
            wetness_cycle: Some(Cycle::new(60.0, 0.5)),
            ..Environment::default()
        };
        assert_eq!(env.at(42), env.at(42));
    }
}
