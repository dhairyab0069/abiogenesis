//! Stage 5 cells: the data and the rules that need no soup around them.
//!
//! Design choices (everything is a deliberately simple stand-in):
//! - **A cell owns its contents.** It takes strands, lipids and nucleotides out of
//!   the soup, keeps them in its own lists and counters, and gives them back when
//!   it dies. Nothing is created or destroyed behind the reader's back, so "the
//!   daughters inherit the contents" is literally true: the mother's list is split
//!   in two.
//! - **Size comes from contents.** A cell with more lipids (membrane) and strands
//!   is bigger. Growing and dividing are therefore one story, not two rules.
//! - **Selection acts on whole cells.** A cell's fitness is its strands' mean
//!   fitness blended with whether it has enough membrane. Fitter cells divide more
//!   often and die less often, so cells whose contents are better spread.
//! - **Eukaryotes need two events.** A big cell swallows a small one: the guest
//!   becomes a *mitochondrion* (more energy: faster copying and dividing) and the
//!   host's strands get a *nucleus* (slower decay, fewer copying errors). A cell
//!   with both is a eukaryote. It is an endosymbiosis story told in two numbers.
//! - **Colonies are eukaryotes that do not let go.** When a eukaryote divides, the
//!   daughter may stay attached. Members of a colony move as one body and take on
//!   roles: *germ* cells do the dividing, *soma* cells are protected and never
//!   divide, *feeder* cells reach further for food and share it. A colony with more
//!   than one role gets a division-of-labour bonus.
//! - **Everything is capped** (cells, strands per cell, mitochondria, colony size).

use crate::model::Molecule;

pub const MAX_CELLS: usize = 24;
pub const MAX_CELL_STRANDS: usize = 10;
pub const MAX_MITOCHONDRIA: u32 = 3;
pub const MAX_COLONY_SIZE: usize = 8;
pub const LIPID_CAP: u32 = 14;
pub const NUCLEOTIDE_CAP: u32 = 30;
/// A cell divides once it holds at least this much (and is fit enough, by chance).
pub const DIVIDE_LIPIDS: u32 = 8;
pub const DIVIDE_STRANDS: usize = 4;

const BASE_RADIUS: f64 = 17.0;
const MAX_RADIUS: f64 = 40.0;
/// Extra energy per mitochondrion.
const MITO_ENERGY: f64 = 0.35;
const NUCLEUS_DECAY: f64 = 0.4;
const NUCLEUS_ERROR: f64 = 0.3;
const BASE_DIVISION: f64 = 0.012;
const BASE_DEATH: f64 = 0.0004;
/// How much of a cell's fitness is its strands (the rest is membrane).
const STRAND_SHARE: f64 = 0.75;
const GERM_DIVISION: f64 = 2.0;
const SOMA_DEATH: f64 = 0.25;
/// A eukaryote's daughter stays attached with this chance; inside a colony, more.
pub const STICK_CHANCE: f64 = 0.35;
pub const COLONY_STICK_CHANCE: f64 = 0.8;
/// Each extra role in a colony: this much more dividing and this much less dying.
const LABOUR_STEP: f64 = 0.2;

/// What a colony member does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    /// A free cell, or a cell in a colony of one.
    #[default]
    Generalist,
    /// Divides twice as often.
    Germ,
    /// Dies a quarter as often and never divides.
    Soma,
    /// Reaches twice as far for food and shares it with the colony.
    Feeder,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Role::Generalist => "generalist",
            Role::Germ => "germ",
            Role::Soma => "soma",
            Role::Feeder => "feeder",
        }
    }
}

/// The role a new colony member takes, given how many of each the colony has:
/// first a germ, then a soma, then a feeder, and after that whichever is rarest
/// (ties go to germ, soma, feeder in that order).
pub fn next_role(germ: usize, soma: usize, feeder: usize) -> Role {
    if germ == 0 {
        Role::Germ
    } else if soma == 0 {
        Role::Soma
    } else if feeder == 0 {
        Role::Feeder
    } else if germ <= soma && germ <= feeder {
        Role::Germ
    } else if soma <= feeder {
        Role::Soma
    } else {
        Role::Feeder
    }
}

/// The multiplier a colony with `roles` different roles gets: 1 for no
/// specialization, then 1.2, 1.4: more dividing, less dying.
pub fn labour_bonus(roles: usize) -> f64 {
    1.0 + LABOUR_STEP * roles.saturating_sub(1) as f64
}

/// The chance a eukaryote's daughter stays attached.
pub fn stick_probability(in_colony: bool) -> f64 {
    if in_colony {
        COLONY_STICK_CHANCE
    } else {
        STICK_CHANCE
    }
}

/// One cell.
#[derive(Clone, Debug)]
pub struct Cell {
    pub id: u64,
    /// The cell it divided from, if any.
    pub parent: Option<u64>,
    /// 0 for a cell that formed from the soup.
    pub generation: u32,
    pub born_step: u64,
    pub x: f64,
    pub y: f64,
    pub strands: Vec<Molecule>,
    pub lipids: u32,
    pub nucleotides: u32,
    pub mitochondria: u32,
    pub nucleus: bool,
    pub colony: Option<u64>,
    pub role: Role,
}

impl Cell {
    pub fn new(id: u64, x: f64, y: f64, born_step: u64) -> Self {
        Self {
            id,
            parent: None,
            generation: 0,
            born_step,
            x,
            y,
            strands: Vec::new(),
            lipids: 0,
            nucleotides: 0,
            mitochondria: 0,
            nucleus: false,
            colony: None,
            role: Role::Generalist,
        }
    }

    /// How much the cell holds: membrane, genes and organelles.
    pub fn size(&self) -> f64 {
        self.lipids as f64
            + 2.0 * self.strands.len() as f64
            + 3.0 * self.mitochondria as f64
            + if self.nucleus { 3.0 } else { 0.0 }
    }

    /// Grows with contents, like the square root of the area they fill.
    pub fn radius(&self) -> f64 {
        (BASE_RADIUS + 2.4 * self.size().sqrt()).min(MAX_RADIUS)
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        (x - self.x).hypot(y - self.y) <= self.radius()
    }

    /// A nucleus and at least one mitochondrion.
    pub fn is_eukaryote(&self) -> bool {
        self.nucleus && self.mitochondria > 0
    }

    /// 1 plus an energy bonus for each mitochondrion.
    pub fn energy(&self) -> f64 {
        1.0 + MITO_ENERGY * self.mitochondria as f64
    }

    /// From 0 to 1: the strands' mean fitness, blended with how well the membrane
    /// is stocked. A cell with no strands has only its membrane to go on.
    pub fn fitness(&self) -> f64 {
        let strands = if self.strands.is_empty() {
            0.0
        } else {
            self.strands.iter().map(Molecule::fitness).sum::<f64>() / self.strands.len() as f64
        };
        let membrane = (self.lipids as f64 / DIVIDE_LIPIDS as f64).min(1.0);
        (STRAND_SHARE * strands + (1.0 - STRAND_SHARE) * membrane).clamp(0.0, 1.0)
    }

    pub fn division_ready(&self) -> bool {
        self.lipids >= DIVIDE_LIPIDS && self.strands.len() >= DIVIDE_STRANDS
    }

    /// Chance per step that a ready cell divides. Zero for soma cells.
    pub fn division_rate(&self, labour: f64) -> f64 {
        if self.role == Role::Soma {
            return 0.0;
        }
        let role = if self.role == Role::Germ {
            GERM_DIVISION
        } else {
            1.0
        };
        BASE_DIVISION * (0.5 + self.fitness()) * self.energy() * role * labour
    }

    /// Chance per step that the cell dies: more for unfit and empty cells, less
    /// with a nucleus, mitochondria, a soma role or a colony's division of labour.
    pub fn death_rate(&self, labour: f64) -> f64 {
        let empty = if self.strands.is_empty() { 3.0 } else { 1.0 };
        let nucleus = if self.nucleus { 0.7 } else { 1.0 };
        let role = if self.role == Role::Soma {
            SOMA_DEATH
        } else {
            1.0
        };
        BASE_DEATH * (1.2 - self.fitness()) * empty * nucleus * role / (self.energy() * labour)
    }

    /// How much slower strands inside fall apart: a nucleus protects them.
    pub fn decay_scale(&self) -> f64 {
        if self.nucleus { NUCLEUS_DECAY } else { 1.0 }
    }

    /// How much of the usual copying error a strand inside has: a nucleus lowers it.
    pub fn error_scale(&self) -> f64 {
        if self.nucleus { NUCLEUS_ERROR } else { 1.0 }
    }

    /// How far from its edge the cell reaches for food.
    pub fn reach(&self) -> f64 {
        self.radius() + if self.role == Role::Feeder { 24.0 } else { 8.0 }
    }
}

/// Things that happen to cells, for the log in the view.
#[derive(Clone, Debug, PartialEq)]
pub enum CellEvent {
    Formed {
        id: u64,
    },
    Divided {
        mother: u64,
        daughter: u64,
        attached: bool,
    },
    Engulfed {
        host: u64,
        guest: u64,
    },
    BecameEukaryote {
        id: u64,
    },
    ColonyFormed {
        colony: u64,
        founder: u64,
    },
    Died {
        id: u64,
    },
}

impl CellEvent {
    pub fn describe(&self) -> String {
        match self {
            CellEvent::Formed { id } => format!("cell {id} formed from the soup"),
            CellEvent::Divided {
                mother,
                daughter,
                attached,
            } => {
                format!(
                    "cell {mother} divided into {daughter}{}",
                    if *attached { " (stayed attached)" } else { "" }
                )
            }
            CellEvent::Engulfed { host, guest } => {
                format!("cell {host} swallowed cell {guest}: a mitochondrion")
            }
            CellEvent::BecameEukaryote { id } => format!("cell {id} is now a eukaryote"),
            CellEvent::ColonyFormed { colony, founder } => {
                format!("colony {colony} began with cell {founder}")
            }
            CellEvent::Died { id } => format!("cell {id} died"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_with(strands: &[&str], lipids: u32) -> Cell {
        let mut c = Cell::new(1, 100.0, 100.0, 0);
        c.strands = strands
            .iter()
            .map(|s| Molecule::strand((*s).into()))
            .collect();
        c.lipids = lipids;
        c
    }

    #[test]
    fn cells_grow_with_their_contents_up_to_a_cap() {
        let small = cell_with(&["GGUCAUAC"], 3);
        let big = cell_with(&["GGUCAUAC"; 6], 12);
        assert!(big.radius() > small.radius());
        let huge = cell_with(&["GGUCAUAC"; 200], 500);
        assert_eq!(huge.radius(), MAX_RADIUS);
        assert!(Cell::new(1, 0.0, 0.0, 0).radius() >= BASE_RADIUS);
    }

    #[test]
    fn fitness_comes_from_the_contents_and_stays_in_range() {
        let good = cell_with(&["GAUCAUGC"; 3], DIVIDE_LIPIDS);
        let poor = cell_with(&["AAAAAAAA"; 3], DIVIDE_LIPIDS);
        let bare = cell_with(&[], 0);
        assert!(good.fitness() > poor.fitness() && poor.fitness() > bare.fitness());
        assert_eq!(bare.fitness(), 0.0);
        for c in [&good, &poor, &bare] {
            assert!((0.0..=1.0).contains(&c.fitness()));
        }
        let loose = cell_with(&["GAUCAUGC"; 3], 0);
        assert!(good.fitness() > loose.fitness(), "membrane counts too");
    }

    #[test]
    fn fit_cells_divide_more_and_die_less() {
        let good = cell_with(&["GAUCAUGC"; 4], DIVIDE_LIPIDS);
        let poor = cell_with(&["AAAAAAAA"; 4], DIVIDE_LIPIDS);
        assert!(good.division_rate(1.0) > poor.division_rate(1.0));
        assert!(good.death_rate(1.0) < poor.death_rate(1.0));
        assert!(
            cell_with(&[], 5).death_rate(1.0) > cell_with(&["AAAAAAAA"], 5).death_rate(1.0),
            "empty cells die sooner"
        );
    }

    #[test]
    fn division_needs_enough_contents() {
        assert!(!cell_with(&["GAUCAUGC"; 3], 20).division_ready());
        assert!(!cell_with(&["GAUCAUGC"; 6], 7).division_ready());
        assert!(cell_with(&["GAUCAUGC"; 4], 8).division_ready());
    }

    #[test]
    fn mitochondria_give_energy_and_a_nucleus_protects() {
        let mut c = cell_with(&["GAUCAUGC"; 4], 8);
        let (rate, death, decay, error) = (
            c.division_rate(1.0),
            c.death_rate(1.0),
            c.decay_scale(),
            c.error_scale(),
        );
        assert!(!c.is_eukaryote());
        c.mitochondria = 1;
        assert!(!c.is_eukaryote(), "a mitochondrion alone is not enough");
        assert!(c.division_rate(1.0) > rate && c.death_rate(1.0) < death && c.energy() > 1.0);
        c.nucleus = true;
        assert!(c.is_eukaryote());
        assert!(c.decay_scale() < decay && c.error_scale() < error);
    }

    #[test]
    fn roles_are_handed_out_germ_then_soma_then_feeder() {
        assert_eq!(next_role(0, 0, 0), Role::Germ);
        assert_eq!(next_role(1, 0, 0), Role::Soma);
        assert_eq!(next_role(1, 1, 0), Role::Feeder);
        assert_eq!(next_role(1, 1, 1), Role::Germ);
        assert_eq!(next_role(2, 1, 1), Role::Soma);
        assert_eq!(next_role(2, 2, 1), Role::Feeder);
    }

    #[test]
    fn division_of_labour_helps_and_soma_never_divides() {
        assert_eq!(labour_bonus(0), 1.0);
        assert_eq!(labour_bonus(1), 1.0);
        assert!(labour_bonus(2) > 1.0 && labour_bonus(3) > labour_bonus(2));
        let mut c = cell_with(&["GAUCAUGC"; 4], 8);
        assert!(c.division_rate(labour_bonus(3)) > c.division_rate(1.0));
        assert!(c.death_rate(labour_bonus(3)) < c.death_rate(1.0));
        let generalist = c.division_rate(1.0);
        c.role = Role::Germ;
        assert!(c.division_rate(1.0) > generalist);
        let death = c.death_rate(1.0);
        c.role = Role::Soma;
        assert_eq!(c.division_rate(1.0), 0.0);
        assert!(c.death_rate(1.0) < death);
        c.role = Role::Feeder;
        assert!(c.reach() > cell_with(&[], 0).reach());
    }

    #[test]
    fn colony_members_stick_more_often() {
        assert!(stick_probability(true) > stick_probability(false));
    }

    #[test]
    fn events_describe_themselves() {
        assert!(
            CellEvent::Engulfed { host: 3, guest: 4 }
                .describe()
                .contains("mitochondrion")
        );
        assert!(
            CellEvent::Divided {
                mother: 1,
                daughter: 2,
                attached: true
            }
            .describe()
            .contains("attached")
        );
    }
}
