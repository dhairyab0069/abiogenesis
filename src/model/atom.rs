//! Atoms and the element constants.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atom {
    pub element: &'static str,
    pub atomic_number: u8,
    pub atomic_mass: f64,
}

pub const HYDROGEN: Atom = Atom { element: "H", atomic_number: 1, atomic_mass: 1.008 };
pub const CARBON: Atom = Atom { element: "C", atomic_number: 6, atomic_mass: 12.011 };
pub const NITROGEN: Atom = Atom { element: "N", atomic_number: 7, atomic_mass: 14.007 };
pub const OXYGEN: Atom = Atom { element: "O", atomic_number: 8, atomic_mass: 15.999 };
