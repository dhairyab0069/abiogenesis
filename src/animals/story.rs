//! The real-history story, retold in short chapters. It is *written text*, a qualitative
//! story and not data: it says "roughly" where it names a time and gives no measurements.
//!
//! Each chapter has the same four parts (setting, problem, adaptation, result) and a note
//! on what the simulation did the same or differently. The static note is about the model;
//! [`simulation_note`] adds what this particular run did.

use super::milestones::{Milestone, Order, order_of};

pub struct Chapter {
    /// The simulation milestone that opens this chapter (`None` for the prologue).
    pub milestone: Option<Milestone>,
    pub title: &'static str,
    pub setting: &'static str,
    pub problem: &'static str,
    pub adaptation: &'static str,
    pub result: &'static str,
    /// What the model does the same or differently, in general.
    pub model: &'static str,
}

pub const CHAPTERS: [Chapter; 9] = [
    Chapter {
        milestone: None,
        title: "Colonies in the sea",
        setting: "Warm shallow seas, with colonies of cells like the ones from the cells stage drifting and feeding.",
        problem: "A colony is small and slow, and every cell has to get its own food and oxygen by soaking them in.",
        adaptation: "Cells in a colony specialised: some fed the others, some protected, some reproduced.",
        result: "A group of cells began to act like one body. Animals are thought to descend from colonies like these.",
        model: "The simulated lineage starts from just such a colony: its feeder cells give a first hint of a gut and its soma cells a first hint of skin.",
    },
    Chapter {
        milestone: Some(Milestone::SymmetryAndGut),
        title: "A front, a back and a gut",
        setting: "The seas of roughly half a billion years ago, when many body plans appeared in a geologically short time (the Cambrian).",
        problem: "A round body has no front, so it can only wait for food to drift by. Moving toward things needs a head end.",
        adaptation: "Bilateral symmetry: a head end, a tail end and a gut that runs through, so food goes in one end and waste leaves the other.",
        result: "Animals could crawl and swim in a direction, and a through-gut can work like a conveyor, eating while digesting.",
        model: "Radial animals can only grow a blind gut and a nerve net here; going bilateral lifts those limits. Real jellyfish and anemones are radial and thrive, as radial animals do in the simulation's open sea.",
    },
    Chapter {
        milestone: Some(Milestone::EyesAndNerveCord),
        title: "Eyes and a nerve cord",
        setting: "Seas full of newly mobile hunters and hunted.",
        problem: "Moving toward food and away from danger is no use without knowing where they are.",
        adaptation: "Light-sensitive patches deepened into eyes, and nerves gathered into a cord with a swelling at the head, a first brain.",
        result: "Faster decisions: seeing a shadow and turning away, or a meal and turning toward it.",
        model: "Here eyes can only grow as far as the nerve allows, and both cost oxygen and upkeep. Real eyes and brains have many more parts and a far longer history.",
    },
    Chapter {
        milestone: Some(Milestone::Backbone),
        title: "A stiffening rod",
        setting: "Active swimmers in open water.",
        problem: "A soft body can only swim so fast: muscle needs something firm to pull against.",
        adaptation: "A flexible rod along the back (a notochord), later wrapped in cartilage and then bone as a backbone.",
        result: "Stronger, longer swimmers, and a frame that could one day carry weight.",
        model: "The simulation has one dial for the whole skeleton, so the rod and the bone are two stretches of the same dial.",
    },
    Chapter {
        milestone: Some(Milestone::Jaws),
        title: "Jaws",
        setting: "Early fish without jaws, sucking and filtering their food. Jawed fish came roughly after them.",
        problem: "A mouth that cannot bite limits what can be eaten.",
        adaptation: "Parts of the supports that held up the gills became hinged jaws.",
        result: "Fish could bite, grab and crush, and a new range of foods and a new sort of predator appeared.",
        model: "In the model jaws can only grow where gills and a skeleton already exist, a loose echo of jaws growing from gill supports.",
    },
    Chapter {
        milestone: Some(Milestone::LimbLikeFins),
        title: "Fins that can push",
        setting: "Shallow, plant-choked waters, among them lobe-finned fish with fleshy fins around bones, and Tiktaalik-like transitional forms, roughly three hundred and seventy million years ago.",
        problem: "In shallow water, thin fins are poor at pushing along the bottom or past plants.",
        adaptation: "Fins with bones inside, a strong base and joints that could bend like an elbow.",
        result: "Fish could prop themselves up and walk a little on the bottom. The first limbs were still used in water.",
        model: "Limbs here only grow out of fins that have a skeleton. In real history limbs with digits seem to have appeared in animals that still lived in water, as in the model.",
    },
    Chapter {
        milestone: Some(Milestone::Lungs),
        title: "Breathing air",
        setting: "Warm, still shallows where water holds little oxygen, and the air above holds much more.",
        problem: "Gills starve in stale water.",
        adaptation: "Many bony fish had lung-like organs that gulp air at the surface. The details of which came first are debated, and a swim bladder is a close relative.",
        result: "A fish that can breathe air can live where water is poor in oxygen, and survive being briefly out of it.",
        model: "Here lungs bud off the gut and only help in zones where air is within reach, which is why low-oxygen worlds change when they appear.",
    },
    Chapter {
        milestone: Some(Milestone::LeavingWater),
        title: "Leaving the water",
        setting: "Tidal flats and muddy margins, wet at high tide and drying at low tide.",
        problem: "Stranded animals dry out, feel their full weight, and cannot swim.",
        adaptation: "Limbs to shove along the mud, lungs to keep breathing, and skin that loses less water.",
        result: "Short trips out of the water for food, or to escape predators, which then became longer trips.",
        model: "The tides on the flats dry and weigh animals down in the model. It does not show why leaving was worth it beyond food and fewer predators. Real reasons are still discussed.",
    },
    Chapter {
        milestone: Some(Milestone::FirstLandAnimals),
        title: "The first land animals",
        setting: "Open ground at the water's edge, with food and few large predators. The first tetrapods (four-limbed animals) came roughly three hundred and sixty million years ago.",
        problem: "No water to hold the body up, a constant risk of drying out and air to breathe.",
        adaptation: "Strong limbs and a backbone to carry the body, lungs for air and a sealed, waterproof skin.",
        result: "A new world opened to life. Their descendants include amphibians, reptiles, birds and mammals.",
        model: "The simulation reaches this milestone when a group of limbed, air-breathing animals lives on land. It has no eggs, no growing up, no plants and no reasons for animals to stay on land, which are all large parts of the real story.",
    },
];

/// The chapter for a milestone.
pub fn chapter_for(milestone: Milestone) -> &'static Chapter {
    CHAPTERS
        .iter()
        .find(|c| c.milestone == Some(milestone))
        .expect("every milestone has a chapter")
}

/// What this run did at `milestone`, given the milestones reached so far (with their generations):
/// when it came and whether the order matched the story.
pub fn simulation_note(milestone: Milestone, reached: &[(Milestone, u32)]) -> String {
    let Some(generation) = reached
        .iter()
        .find(|(m, _)| *m == milestone)
        .map(|(_, g)| *g)
    else {
        return "Not reached yet in this run.".to_string();
    };
    let order = match order_of(milestone, reached) {
        Some(Order::Same) => "the same place in the sequence as the story".to_string(),
        Some(Order::Earlier(n)) => format!(
            "{n} place{} earlier than in the story",
            if n == 1 { "" } else { "s" }
        ),
        Some(Order::Later(n)) => format!(
            "{n} place{} later than in the story",
            if n == 1 { "" } else { "s" }
        ),
        None => String::new(),
    };
    format!("Reached at generation {generation}, {order}.")
}
