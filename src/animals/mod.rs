//! Stage 6: animals from sea to land. A simplified, honest evolving-animal model.
//!
//! Design choices (everything is a deliberately simple stand-in, not a biology textbook):
//! - **A body is a handful of dials.** A body plan (radial or bilateral) and eleven organ
//!   levels from 0 to 1 (`traits`). Some organs can only grow once the ones they come from
//!   exist, which is how real organs build on older parts.
//! - **The shore is four zones.** Open sea, shallows, tidal flats and land (`zone`), with
//!   qualitative differences in oxygen, support, drying, food and predators. Three dials
//!   (oxygen, slope, predators) change the world: the "what if" hook for the scenario lab.
//! - **Fitness is a product of trade-offs** (`fitness`), so no organ is free and none is
//!   useless everywhere.
//! - **A run is a pure function of (environment, seed)** (`run`): individuals, births in
//!   proportion to fitness, mutation, a little wandering between zones. Nothing steers the
//!   lineage toward land.
//! - **Milestones** (`milestones`) are detected in the run and drive the side-by-side story,
//!   whose chapters (`story`) retell the real history, qualitatively, as a story and not as data.
//! - **The picture** (`crate::animalviz`) is a function of the same numbers.

mod fitness;
mod milestones;
mod run;
mod story;
mod traits;
mod zone;

pub use fitness::{Fitness, fitness, sense, speed};
pub use milestones::{FOOTHOLD, Milestone, Order, SETTLED, order_of, reached};
pub use run::{
    Animal, CAPACITY, DEFAULT_SEED, Frame, GENERATIONS, Run, SAMPLES, Sample, Summary, room,
};
pub use story::{CHAPTERS, Chapter, chapter_for, simulation_note};
pub use traits::{BILATERAL_UPKEEP, Mean, ORGANS, Organ, Traits};
pub use zone::{AIR_OXYGEN, Environment, ZONES, Zone};

#[cfg(test)]
mod tests;
