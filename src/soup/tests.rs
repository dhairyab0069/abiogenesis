use super::generation::INITIAL_AMINO_ACIDS;
use super::recombination::MAX_COMBINATIONS_PER_STEP;
use super::*;
use crate::events::{ChainEvent, Mutation};

#[test]
fn starts_with_seed_chains_inside_bounds() {
    let soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 1);
    assert_eq!(soup.molecules.len(), INITIAL_AMINO_ACIDS);
    assert!(soup.molecules.iter().all(|m| m.is_amino_acid() && soup.is_valid_position(&m.position)));
}

#[test]
fn standard_soup_adds_water_and_methane() {
    let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 2);
    soup.add_small_molecules();
    let counts = soup.counts();
    let get = |n: &str| counts.iter().find(|(k, _)| k == n).map(|(_, c)| *c);
    assert_eq!(get("H2O"), Some(50));
    assert_eq!(get("CH4"), Some(50));
    assert_eq!(soup.amino_acid_count(), INITIAL_AMINO_ACIDS);
}

#[test]
fn molecules_stay_in_bounds_and_chains_stay_capped() {
    let mut soup = PrimordialSoup::with_seed(200.0, 100.0, 3000.0, 7.0, 3);
    for _ in 0..5_000 {
        soup.simulate_step();
    }
    assert!(soup.molecules.iter().all(|m| (0.0..=200.0).contains(&m.position.x)
        && (0.0..=100.0).contains(&m.position.y)));
    // Generation stops at the cap; recombination can add at most 10 past it.
    assert!(soup.amino_acid_count() <= MAX_AMINO_ACIDS + MAX_COMBINATIONS_PER_STEP);
}

#[test]
fn recombination_terminates_with_too_few_chains() {
    let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 4);
    soup.molecules.truncate(1);
    soup.molecules.push(Molecule::water());
    soup.simulate_step();
}

#[test]
fn resize_rescales_positions() {
    let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 5);
    soup.molecules[0].position = Position::new(50.0, 25.0);
    soup.resize(200.0, 50.0);
    assert_eq!(soup.molecules[0].position, Position::new(100.0, 12.5));
}

#[test]
fn reports_target_pattern() {
    let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 6);
    let mut m = Molecule::amino_acid("XXMETHINKSXX".into(), 1.0);
    m.position = Position::new(10.0, 10.0);
    soup.molecules = vec![m];
    let found = soup.simulate_step();
    assert_eq!(found, vec!["XXMETHINKSXX".to_string()]);
}

fn busy_soup(seed: u64) -> PrimordialSoup {
    // Hot and long, so all three mutation kinds and recombination occur.
    let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 3000.0, 7.0, seed);
    for _ in 0..3_000 {
        soup.simulate_step();
    }
    soup
}

fn apply(parent: &str, m: &Mutation) -> String {
    let mut chars: Vec<char> = parent.chars().collect();
    match *m {
        Mutation::Substitution { index, from, to } => {
            assert_eq!(chars[index], from);
            chars[index] = to;
        }
        Mutation::Insertion { index, residue } => chars.insert(index, residue),
        Mutation::Deletion { index, residue } => assert_eq!(chars.remove(index), residue),
    }
    chars.into_iter().collect()
}

#[test]
fn every_chain_has_a_creation_event() {
    let soup = busy_soup(21);
    for m in soup.molecules.iter().filter(|m| m.is_amino_acid()) {
        let created = soup.events.creation_of(m.id).expect("chain has no creation event");
        assert_eq!(created.event.sequence(), m.sequence().unwrap());
    }
    // Seed chains are logged at step 0 and the step counter advanced.
    assert_eq!(soup.step, 3_000);
    assert!(soup.events.since(0)[..INITIAL_AMINO_ACIDS].iter().all(|s| s.step == 0));
}

#[test]
fn parents_are_created_before_their_children() {
    let soup = busy_soup(22);
    let all = soup.events.since(0);
    let (mut mutated, mut recombined) = (0, 0);
    for (i, stamped) in all.iter().enumerate() {
        match stamped.event {
            ChainEvent::Mutated { .. } => mutated += 1,
            ChainEvent::Recombined { .. } => recombined += 1,
            ChainEvent::Generated { .. } | ChainEvent::Translated { .. } => {}
        }
        for parent in stamped.event.parents() {
            assert!(
                all[..i].iter().any(|s| s.event.id() == parent),
                "parent {parent} of event {i} not created earlier"
            );
        }
    }
    assert!(mutated > 0 && recombined > 0, "run too quiet to test anything");
}

#[test]
fn event_steps_never_go_backwards() {
    let soup = busy_soup(23);
    let steps: Vec<u64> = soup.events.since(0).iter().map(|s| s.step).collect();
    assert!(steps.windows(2).all(|w| w[0] <= w[1]));
    assert!(*steps.last().unwrap() > 0);
}

#[test]
fn recorded_edits_explain_the_children() {
    let soup = busy_soup(24);
    let seq_of = |id| soup.events.creation_of(id).unwrap().event.sequence().to_string();
    let (mut sub, mut ins, mut del) = (0, 0, 0);
    for stamped in soup.events.since(0) {
        match &stamped.event {
            ChainEvent::Mutated { parent, mutation, sequence, .. } => {
                assert_eq!(&apply(&seq_of(*parent), mutation), sequence);
                match mutation {
                    Mutation::Substitution { .. } => sub += 1,
                    Mutation::Insertion { .. } => ins += 1,
                    Mutation::Deletion { .. } => del += 1,
                }
            }
            ChainEvent::Recombined { parents, cut, sequence, .. } => {
                let expected = format!("{}{}", &seq_of(parents.0)[..cut.0], &seq_of(parents.1)[cut.1..]);
                assert_eq!(&expected, sequence);
            }
            ChainEvent::Generated { .. } | ChainEvent::Translated { .. } => {}
        }
    }
    assert!(sub > 0 && ins > 0 && del > 0, "saw sub {sub}, ins {ins}, del {del}");
}

#[test]
fn mutated_child_replaces_parent_with_new_id() {
    let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 25);
    let before: Vec<_> = soup.molecules.iter().map(|m| m.id).collect();
    for _ in 0..2_000 {
        soup.simulate_step();
    }
    let mutated_parents: Vec<_> = soup
        .events
        .since(0)
        .iter()
        .filter_map(|s| match s.event {
            ChainEvent::Mutated { parent, .. } => Some(parent),
            _ => None,
        })
        .collect();
    assert!(!mutated_parents.is_empty());
    // A parent that was mutated is gone from the soup (unless recombination
    // alone kept a copy, which it never does: children are new chains).
    assert!(soup.molecules.iter().all(|m| !mutated_parents.contains(&m.id)));
    assert!(before.iter().any(|id| mutated_parents.contains(id)));
}

#[test]
fn unrecorded_chains_do_not_break_the_log() {
    let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 3000.0, 7.0, 26);
    let mut extra = Molecule::amino_acid("MMMMM".into(), 1.0);
    extra.position = Position::new(5.0, 5.0);
    soup.add_molecule(extra);
    soup.molecules.push(Molecule::amino_acid("KKK".into(), 1.0));
    for _ in 0..2_000 {
        soup.simulate_step();
    }
    let last = soup.events.len().saturating_sub(1);
    let _ = soup.events.ancestors(soup.events.since(last)[0].event.id());
}

#[test]
fn same_seed_gives_same_log() {
    assert_eq!(busy_soup(27).events.since(0), busy_soup(27).events.since(0));
}

/// Fingerprint of the final population: sequence, energy and position of every
/// molecule (FNV-1a), so it also catches any change in RNG draw order.
fn fingerprint(soup: &PrimordialSoup) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for m in &soup.molecules {
        let t = format!("{:?}|{:.6}|{:.6}|{:.6};", m.sequence(), m.energy, m.position.x, m.position.y);
        for b in t.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
    }
    h
}

/// The expected values were produced by the code at HEAD, before event
/// logging existed (same seeds, 3000 steps, 300x200 soup).
#[test]
fn seeded_runs_match_the_pre_event_log_code() {
    for (seed, temp, len, hash) in
        [(11, 300.0, 50, 0xdfe75a6faf00ccc8), (12, 3000.0, 50, 0xa74eda001e6d664f)]
    {
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, temp, 7.0, seed);
        for _ in 0..3_000 {
            soup.simulate_step();
        }
        assert_eq!(soup.molecules.len(), len);
        assert_eq!(fingerprint(&soup), hash, "seed {seed} diverged from the old behaviour");
    }
}
