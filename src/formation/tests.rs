use super::*;
use rand::{SeedableRng, rngs::StdRng};

fn scene() -> Formation {
    Formation::default()
}

#[test]
fn the_default_gene_spells_mafkd() {
    let f = scene();
    assert_eq!(f.protein(), "MAFKD");
    assert_eq!(f.frame().start, 2);
    assert!(f.frame().is_closed());
    assert_eq!(f.copy(), complement_strand(DEFAULT_GENE));
    assert!((f.fitness() - protein_fitness("MAFKD")).abs() < 1e-12);
}

#[test]
fn a_strand_with_no_gene_is_refused() {
    assert!(Formation::with_gene("GGGCCC").is_none());
    assert!(
        Formation::with_gene("CCAUGUAA").is_none(),
        "a lone M is not a protein"
    );
    let mut f = scene();
    assert!(!f.set_gene("CCCCC"));
    assert_eq!(f.gene(), DEFAULT_GENE);
}

#[test]
fn the_phases_come_in_order_and_cover_the_loop() {
    let f = scene();
    let phases: Vec<Phase> = f.durations().iter().map(|(p, _)| *p).collect();
    assert_eq!(phases, Phase::ALL.to_vec());
    assert!(f.durations().iter().all(|(_, d)| *d > 0.0));
    let mut last = Phase::Assemble;
    let mut t = 0.0;
    while t < f.total_time() {
        let (p, progress) = f.locate_at(t);
        assert!(p >= last, "phases never go backwards");
        assert!((0.0..=1.0).contains(&progress));
        last = p;
        t += 0.05;
    }
    assert_eq!(last, Phase::Rest);
}

#[test]
fn beads_join_one_at_a_time() {
    let f = scene();
    let (_, d) = f.durations()[0];
    let start = f.snapshot_at(0.0);
    assert_eq!((start.phase, start.joined), (Phase::Assemble, 0.0));
    let mid = f.snapshot_at(d / 2.0);
    assert!((mid.joined - DEFAULT_GENE.len() as f32 / 2.0).abs() < 0.01);
    let mut last = 0.0;
    let mut t = 0.0;
    while t < d {
        let s = f.snapshot_at(t);
        assert!(
            s.joined >= last && s.joined - last < 1.0,
            "bonds form one at a time"
        );
        last = s.joined;
        t += 0.05;
    }
    assert_eq!(f.snapshot_at(d + 0.01).joined, DEFAULT_GENE.len() as f32);
}

#[test]
fn pairing_docks_the_complement_and_the_helix_twists_then_unzips() {
    let f = scene();
    let n = DEFAULT_GENE.len() as f32;
    let d = f.durations();
    let after_assemble = d[0].1;
    assert_eq!(f.snapshot_at(after_assemble - 0.01).docked, 0.0);
    let mid_pair = f.snapshot_at(after_assemble + d[1].1 / 2.0);
    assert_eq!(mid_pair.phase, Phase::Pair);
    assert!((mid_pair.docked - n / 2.0).abs() < 0.05);
    let twist_start = after_assemble + d[1].1;
    assert_eq!(f.snapshot_at(twist_start - 0.01).twist, 0.0);
    let half_twist = f.snapshot_at(twist_start + d[2].1 / 2.0);
    assert!((half_twist.twist - 0.5).abs() < 0.01 && half_twist.docked == n);
    let unzip_start = twist_start + d[2].1;
    let mid_unzip = f.snapshot_at(unzip_start + d[3].1 / 2.0);
    assert_eq!(mid_unzip.phase, Phase::Unzip);
    assert!(mid_unzip.unzipped > 0.0 && mid_unzip.unzipped < n);
    assert_eq!(f.snapshot_at(unzip_start + d[3].1 + 0.01).unzipped, n);
}

#[test]
fn the_ribosome_scans_to_the_start_codon_then_reads_in_threes() {
    let mut f = scene();
    f.seek(Phase::Translate);
    let begin = f.snapshot();
    assert_eq!((begin.phase, begin.codons_read), (Phase::Translate, 0.0));
    assert!(begin.ribosome_at < 0.05);
    assert!(begin.codon.is_none(), "still scanning, no codon yet");
    let start = f.frame().start as f32;
    let t0 = f.time();
    let at_aug = f.snapshot_at(t0 + start * SCAN_TIME);
    assert!((at_aug.ribosome_at - start).abs() < 0.01);
    assert_eq!(at_aug.codon.as_ref().map(|(c, _)| c.as_str()), Some("AUG"));
    let after_two = f.snapshot_at(t0 + start * SCAN_TIME + 2.0 * CODON_TIME + 0.01);
    assert!(
        (after_two.ribosome_at - (start + 6.0)).abs() < 0.1,
        "three bases per codon"
    );
    assert_eq!(after_two.amino_acids, 2);
    assert_eq!(
        after_two.codon,
        Some(("UUC".to_string(), Codon::Amino('F')))
    );
}

#[test]
fn amino_acids_never_decrease_and_end_at_the_protein_length() {
    let f = scene();
    let mut last = 0;
    let mut seen_stop = false;
    let mut t = 0.0;
    while t < f.total_time() {
        let s = f.snapshot_at(t);
        assert!(s.amino_acids >= last);
        last = s.amino_acids;
        if let Some((_, Codon::Stop)) = s.codon {
            seen_stop = true;
            assert!(
                !s.released,
                "the chain is released only after the stop codon is read"
            );
        }
        t += 0.02;
    }
    assert!(seen_stop, "the stop codon is read");
    assert_eq!(last, f.protein().len());
}

#[test]
fn the_chain_is_released_at_the_stop_codon_and_then_folds() {
    let mut f = scene();
    f.seek(Phase::Fold);
    let s = f.snapshot();
    assert!(s.released && s.fold < 0.01);
    f.advance(1.5);
    assert!(f.snapshot().fold > 0.3);
    f.seek(Phase::Rest);
    assert_eq!(f.snapshot().fold, 1.0);
    f.seek(Phase::Assemble);
    assert!(!f.snapshot().released);
}

#[test]
fn the_clock_loops_pauses_and_scales_with_speed() {
    let mut f = scene();
    let total = f.total_time();
    f.advance(1.0);
    assert!((f.time() - 1.0).abs() < 1e-6);
    f.set_speed(2.0);
    f.advance(1.0);
    assert!((f.time() - 3.0).abs() < 1e-6);
    f.paused = true;
    f.advance(5.0);
    assert!((f.time() - 3.0).abs() < 1e-6, "paused clocks do not move");
    f.paused = false;
    f.advance(total / 2.0 + 0.1); // 2x speed: more than one loop
    assert!(f.time() < total && f.time() >= 0.0);
    f.set_speed(100.0);
    assert_eq!(f.speed(), MAX_SPEED);
    f.set_speed(0.0);
    assert_eq!(f.speed(), MIN_SPEED);
    f.restart();
    assert_eq!(f.time(), 0.0);
}

#[test]
fn dna_is_only_a_spelling() {
    let mut f = scene();
    assert_eq!(f.shown(Phase::Assemble, 'U'), 'U');
    f.dna = true;
    assert_eq!(f.shown(Phase::Assemble, 'U'), 'T');
    assert_eq!(f.shown(Phase::Unzip, 'U'), 'T');
    assert_eq!(
        f.shown(Phase::Translate, 'U'),
        'U',
        "the ribosome reads mRNA"
    );
    assert_eq!(f.shown(Phase::Pair, 'C'), 'C');
    assert_eq!(f.pair_of('A'), 'T');
    assert_eq!(f.pair_of('C'), 'G');
    f.dna = false;
    assert_eq!(f.pair_of('A'), 'U');
    // the spelling never changes what is built
    assert_eq!(f.protein(), "MAFKD");
}

#[test]
fn hydrogen_bonds_follow_the_pairs() {
    assert_eq!(Formation::bonds_between('A'), 2);
    assert_eq!(Formation::bonds_between('U'), 2);
    assert_eq!(Formation::bonds_between('C'), 3);
    assert_eq!(Formation::bonds_between('G'), 3);
}

#[test]
fn swapping_the_gene_restarts_but_keeps_the_settings() {
    let mut f = scene();
    f.dna = true;
    f.set_speed(2.0);
    f.advance(4.0);
    assert!(f.set_gene("AUGUUUAAGUAA"));
    assert_eq!((f.time(), f.dna, f.speed()), (0.0, true, 2.0));
    assert_eq!(f.protein(), "MFK");
}

#[test]
fn random_genes_always_have_a_protein_and_fit_in_a_strand() {
    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        let gene = random_gene(&mut rng);
        assert!(gene.len() <= MAX_PROTEIN_GENE_BASES, "{gene}");
        let frame = reading_frame(&gene).unwrap();
        assert!(frame.is_closed(), "{gene}");
        let f = Formation::with_gene(&gene).unwrap_or_else(|| panic!("{gene} has no protein"));
        assert_eq!(f.protein(), translate(&gene).unwrap());
        assert!(f.protein().len() >= 3 && f.protein().starts_with('M'));
    }
    let a = random_gene(&mut StdRng::seed_from_u64(5));
    assert_eq!(
        a,
        random_gene(&mut StdRng::seed_from_u64(5)),
        "the same seed gives the same gene"
    );
}

#[test]
fn every_amino_acid_has_a_name() {
    let names: std::collections::HashSet<_> = crate::model::AMINO_ACIDS
        .iter()
        .map(|&c| amino_name(c))
        .collect();
    assert_eq!(names.len(), crate::model::AMINO_ACIDS.len());
    assert!(!names.contains("?"));
}
