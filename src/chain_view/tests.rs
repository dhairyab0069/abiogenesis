use super::*;
use crate::events::{ChainEvent, ChainId, EventLog, Mutation, UNRECORDED};
use crate::soup::PrimordialSoup;

fn generated(log: &mut EventLog, seq: &str) -> ChainId {
    let id = log.new_id();
    log.record(0, ChainEvent::Generated { id, sequence: seq.into() });
    id
}

fn mutated(log: &mut EventLog, parent: ChainId, mutation: Mutation, seq: &str) -> ChainId {
    let id = log.new_id();
    log.record(1, ChainEvent::Mutated { id, parent, mutation, sequence: seq.into() });
    id
}

fn settle(view: &mut ChainView) {
    view.advance(1000.0);
}

fn seqs(view: &ChainView) -> Vec<String> {
    view.rows().iter().map(Row::sequence).collect()
}

#[test]
fn rows_grow_one_tile_at_a_time_to_full_length() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "METHINKS");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    assert_eq!(view.rows()[0].visible_len(), 1, "the first tile starts at once");
    let mut last = view.rows()[0].visible_len();
    for _ in 0..200 {
        view.advance(0.01);
        let now = view.rows()[0].visible_len();
        assert!(now >= last && now - last <= 1, "{last} -> {now}");
        last = now;
    }
    assert_eq!(last, 8);
    assert!(!view.is_animating());
}

#[test]
fn substitution_flashes_exactly_the_recorded_index() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "MET");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    settle(&mut view);
    let b = mutated(&mut log, a, Mutation::Substitution { index: 1, from: 'E', to: 'A' }, "MAT");
    view.sync(&log, [b]);
    let tiles = &view.rows()[0].tiles;
    let flashing: Vec<usize> = (0..3).filter(|&i| tiles[i].flash().is_some()).collect();
    assert_eq!(flashing, vec![1]);
    assert_eq!(tiles[1].flash().map(|f| f.0), Some(EditKind::Substitution));
    assert_eq!(tiles[1].shown_letter(), 'E', "old letter first");
    view.advance(FLASH_SECONDS * 0.75);
    assert_eq!(view.rows()[0].tiles[1].shown_letter(), 'A');
    settle(&mut view);
    assert_eq!(seqs(&view), vec!["MAT"]);
    assert!(view.rows()[0].tiles[1].flash().is_none());
}

#[test]
fn insertion_adds_a_tile_at_the_recorded_index() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "MT");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    settle(&mut view);
    let b = mutated(&mut log, a, Mutation::Insertion { index: 1, residue: 'E' }, "MET");
    view.sync(&log, [b]);
    let tiles = &view.rows()[0].tiles;
    assert_eq!(tiles.len(), 3);
    assert_eq!(tiles[1].letter, 'E');
    assert_eq!(tiles[1].flash().map(|f| f.0), Some(EditKind::Insertion));
    assert!(tiles[0].flash().is_none() && tiles[2].flash().is_none());
    settle(&mut view);
    assert_eq!(seqs(&view), vec!["MET"]);
}

#[test]
fn deletion_removes_the_recorded_index_after_shrinking() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "MEET");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    settle(&mut view);
    let b = mutated(&mut log, a, Mutation::Deletion { index: 2, residue: 'E' }, "MET");
    view.sync(&log, [b]);
    assert_eq!(view.rows()[0].tiles.len(), 4, "still there, shrinking");
    assert_eq!(view.rows()[0].sequence(), "MET");
    view.advance(SHRINK_SECONDS / 2.0);
    let s = view.rows()[0].tiles[2].scale();
    assert!(s > 0.0 && s < 1.0);
    view.advance(SHRINK_SECONDS);
    assert_eq!(view.rows()[0].tiles.len(), 3);
    assert_eq!(seqs(&view), vec!["MET"]);
}

#[test]
fn an_edit_during_a_shrink_still_lines_up() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "MEET");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    settle(&mut view);
    let b = mutated(&mut log, a, Mutation::Deletion { index: 0, residue: 'M' }, "EET");
    view.sync(&log, [b]);
    let c = mutated(&mut log, b, Mutation::Substitution { index: 2, from: 'T', to: 'K' }, "EEK");
    view.sync(&log, [c]);
    assert_eq!(view.rows().len(), 1);
    settle(&mut view);
    assert_eq!(seqs(&view), vec!["EEK"]);
}

#[test]
fn recombination_attributes_prefix_and_suffix_by_the_cuts() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "METHINKS");
    let b = generated(&mut log, "WQRPV");
    let c = log.new_id();
    log.record(2, ChainEvent::Recombined { id: c, parents: (a, b), cut: (3, 2), sequence: "METRPV".into() });
    let mut view = ChainView::new();
    view.sync(&log, [a, b, c]);
    let row = &view.rows()[2];
    let origins: Vec<Origin> = row.tiles.iter().map(|t| t.origin).collect();
    let (pa, pb) = (Origin::ParentA, Origin::ParentB);
    assert_eq!(origins, vec![pa, pa, pa, pb, pb, pb]);
    assert_eq!(row.origin_strength(), 1.0);
    view.advance(RECOMBINE_SECONDS + 0.01);
    let row = &view.rows()[2];
    assert!(row.tiles.iter().all(|t| t.origin == Origin::None), "settles back");
    assert_eq!(row.sequence(), "METRPV");
    assert_eq!(view.rows().len(), 3, "a recombination adds a row");
}

#[test]
fn a_mutation_replaces_its_parents_row_in_place() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "AA");
    let b = generated(&mut log, "CC");
    let d = generated(&mut log, "DD");
    let b2 = mutated(&mut log, b, Mutation::Insertion { index: 2, residue: 'K' }, "CCK");
    let mut view = ChainView::new();
    view.sync(&log, [a, b2, d]);
    settle(&mut view);
    assert_eq!(seqs(&view), vec!["AA", "CCK", "DD"]);
    assert_eq!(view.rows()[1].id, b2);
}

#[test]
fn rows_vanish_when_their_chain_is_no_longer_live() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "AA");
    let b = generated(&mut log, "CC");
    let mut view = ChainView::new();
    view.sync(&log, [a, b]);
    view.sync(&log, [b]);
    assert_eq!(seqs(&view), vec!["CC"]);
    view.sync(&log, [UNRECORDED, 99]);
    assert!(view.rows().is_empty());
}

#[test]
fn the_cursor_never_replays_old_events() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "AA");
    let mut view = ChainView::new();
    view.sync(&log, [a]);
    view.sync(&log, [a]);
    assert_eq!(view.rows().len(), 1);
    let b = generated(&mut log, "CC");
    view.sync(&log, [a, b]);
    view.sync(&log, [a, b]);
    assert_eq!(seqs(&view), vec!["AA", "CC"]);
}

#[test]
fn a_mismatched_parent_row_is_rebuilt_not_shown_wrong() {
    let mut log = EventLog::new();
    let b = mutated(&mut log, 77, Mutation::Substitution { index: 0, from: 'A', to: 'C' }, "CAT");
    let mut view = ChainView::new();
    view.sync(&log, [b]);
    settle(&mut view);
    assert_eq!(seqs(&view), vec!["CAT"]);
}

#[test]
fn huge_dt_settles_every_row() {
    let mut log = EventLog::new();
    let a = generated(&mut log, "METHINKS");
    let b = mutated(&mut log, a, Mutation::Insertion { index: 0, residue: 'W' }, "WMETHINKS");
    let c = generated(&mut log, "KK");
    let d = log.new_id();
    log.record(2, ChainEvent::Recombined { id: d, parents: (b, c), cut: (4, 1), sequence: "WMETK".into() });
    let mut view = ChainView::new();
    view.sync(&log, [b, c, d]);
    assert!(view.is_animating());
    view.advance(1.0e9);
    assert!(!view.is_animating());
    assert!(view.rows().iter().all(|r| r.tiles.iter().all(|t| t.motion == Motion::Settled && t.scale() == 1.0)));
}

#[test]
fn rows_are_capped() {
    let mut log = EventLog::new();
    let ids: Vec<_> = (0..MAX_ROWS + 5).map(|_| generated(&mut log, "A")).collect();
    let mut view = ChainView::new();
    view.sync(&log, ids.clone());
    assert_eq!(view.rows().len(), MAX_ROWS);
    assert_eq!(view.rows()[0].id, ids[5], "the oldest rows go first");
}

#[test]
fn follows_a_real_soup_for_two_thousand_steps() {
    for seed in [1, 2, 3] {
        let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, seed);
        let mut view = ChainView::new();
        for _ in 0..2000 {
            soup.simulate_step();
            let live = soup.molecules.iter().filter(|m| m.is_amino_acid()).map(|m| m.id);
            view.sync(&soup.events, live);
            view.advance(1.0 / 20.0);
        }
        assert!(view.rows().len() <= MAX_ROWS);
        let mut shown = seqs(&view);
        let mut actual: Vec<String> =
            soup.molecules.iter().filter_map(|m| m.sequence()).map(str::to_string).collect();
        shown.sort();
        actual.sort();
        assert_eq!(shown, actual, "seed {seed}");
    }
}
