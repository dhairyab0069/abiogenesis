use super::*;
use crate::events::Mutation;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn rng() -> StdRng {
    StdRng::seed_from_u64(7)
}

fn chain(seq: &str) -> Molecule {
    Molecule::amino_acid(seq.to_string(), 2.0)
}

#[test]
fn small_molecules_have_expected_atoms() {
    assert_eq!(Molecule::water().composition(), vec![("H", 2), ("O", 1)]);
    assert_eq!(Molecule::methane().composition(), vec![("C", 1), ("H", 4)]);
    assert_eq!(Molecule::water().structure(), Some("H-O-H (Bent)"));
}

#[test]
fn chain_has_four_backbone_atoms_per_residue() {
    let m = chain("MET");
    assert_eq!(m.name(), "AA3");
    assert_eq!(m.atoms.len(), 12);
    assert_eq!(m.composition(), vec![("C", 3), ("N", 3), ("O", 3), ("H", 3)]);
}

#[test]
fn random_chain_respects_length_and_alphabet() {
    let mut r = rng();
    for _ in 0..200 {
        let m = Molecule::random_chain(&mut r, 1, 3);
        let seq = m.sequence().unwrap();
        assert!((1..=3).contains(&seq.len()));
        assert!(seq.chars().all(|c| AMINO_ACIDS.contains(&c)));
        assert!((m.energy - (1.0 + seq.len() as f64 * 0.1)).abs() < 1e-9);
    }
    for s in random_chains(&mut r, 50) {
        assert!((5..=20).contains(&s.len()));
    }
}

#[test]
fn mutations_change_length_as_expected() {
    let mut r = rng();
    let m = chain("METHINKS");
    assert_eq!(m.mutate_substitution(&mut r).unwrap().0.sequence().unwrap().len(), 8);
    assert_eq!(m.mutate_insertion(&mut r).unwrap().0.sequence().unwrap().len(), 9);
    assert_eq!(m.mutate_deletion(&mut r).unwrap().0.sequence().unwrap().len(), 7);
    assert!(chain("M").mutate_deletion(&mut r).is_none());
}

#[test]
fn mutations_report_the_exact_edit() {
    let mut r = rng();
    let parent = chain("METHINKS");
    for _ in 0..50 {
        let (c, m) = parent.mutate_substitution(&mut r).unwrap();
        let Mutation::Substitution { index, from, to } = m else { panic!("wrong kind") };
        assert_eq!(parent.sequence().unwrap().chars().nth(index), Some(from));
        assert_eq!(c.sequence().unwrap().chars().nth(index), Some(to));

        let (c, m) = parent.mutate_insertion(&mut r).unwrap();
        let Mutation::Insertion { index, residue } = m else { panic!("wrong kind") };
        assert_eq!(c.sequence().unwrap().chars().nth(index), Some(residue));

        let (c, m) = parent.mutate_deletion(&mut r).unwrap();
        let Mutation::Deletion { index, residue } = m else { panic!("wrong kind") };
        assert_eq!(parent.sequence().unwrap().chars().nth(index), Some(residue));
        assert_eq!(c.sequence().unwrap().len(), 7);
    }
}

#[test]
fn edge_cases_report_no_edit_and_use_no_randomness() {
    let mut r = rng();
    let mut untouched = rng();
    let empty = chain("");
    assert!(empty.mutate_substitution(&mut r).is_none());
    assert!(empty.mutate_deletion(&mut r).is_none());
    assert!(empty.crossover(&chain("AA"), &mut r).is_none());
    assert!(chain("AA").crossover(&empty, &mut r).is_none());
    assert_eq!(r.random::<u64>(), untouched.random::<u64>());
    // Inserting into an empty chain is a real edit.
    let (c, m) = empty.mutate_insertion(&mut r).unwrap();
    assert_eq!(c.sequence().unwrap().len(), 1);
    assert!(matches!(m, Mutation::Insertion { index: 0, .. }));
}

#[test]
fn mutations_keep_position_and_energy() {
    let mut r = rng();
    let mut m = chain("AAAA");
    m.position = Position::new(3.0, 4.0);
    let (mutated, _) = m.mutate_insertion(&mut r).unwrap();
    assert_eq!(mutated.position, m.position);
    assert_eq!(mutated.energy, m.energy);
}

#[test]
fn crossover_combine_and_degrade() {
    let mut r = rng();
    let (a, b) = (chain("AAAA"), chain("CCCC"));
    let (child, (p1, p2)) = a.crossover(&b, &mut r).unwrap();
    assert_eq!(child.sequence().unwrap().len(), p1 + 4 - p2);
    let seq = child.sequence().unwrap();
    assert!(seq.trim_start_matches('A').chars().all(|c| c == 'C'));
    assert_eq!(child.energy, 2.0);
    let joined = Molecule::combine(&a, &b);
    assert_eq!(joined.sequence(), Some("AAAACCCC"));
    assert_eq!(joined.energy, 3.5);
    let d = a.degrade(&mut r).unwrap();
    assert!(d.sequence().unwrap().len() < 4);
    assert!(chain("A").degrade(&mut r).is_none());
}

#[test]
fn distance() {
    assert_eq!(Position::new(0.0, 0.0).distance_to(&Position::new(3.0, 4.0)), 5.0);
}
