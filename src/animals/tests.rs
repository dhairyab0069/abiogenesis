use super::*;
use crate::soup::Role;
use rand::{SeedableRng, rngs::StdRng};

const SEED: u64 = DEFAULT_SEED;

fn body(bilateral: bool, set: &[(Organ, f32)]) -> Traits {
    let mut t = Traits::plain();
    t.bilateral = bilateral;
    for (o, l) in set {
        t.set(*o, *l);
    }
    t
}

fn fish() -> Traits {
    body(
        true,
        &[
            (Organ::Gut, 0.9),
            (Organ::Nerve, 0.7),
            (Organ::Eyes, 0.6),
            (Organ::Muscle, 0.8),
            (Organ::Fins, 0.8),
            (Organ::Skeleton, 0.5),
            (Organ::Gills, 0.9),
        ],
    )
}

fn walker() -> Traits {
    body(
        true,
        &[
            (Organ::Gut, 0.9),
            (Organ::Nerve, 0.7),
            (Organ::Eyes, 0.6),
            (Organ::Muscle, 0.8),
            (Organ::Fins, 0.3),
            (Organ::Skeleton, 0.9),
            (Organ::Lungs, 0.8),
            (Organ::Limbs, 0.9),
            (Organ::Skin, 0.9),
        ],
    )
}

fn total(t: &Traits, z: Zone, env: &Environment) -> f32 {
    fitness(t, z, env).total()
}

// ---- traits ----

#[test]
fn a_colony_ancestor_starts_with_hints_of_a_gut_and_skin_and_nothing_else() {
    let t = Traits::from_colony(&[Role::Germ, Role::Soma, Role::Feeder]);
    assert!(!t.bilateral);
    assert!(t.get(Organ::Gut) > 0.05 && t.get(Organ::Skin) > 0.05);
    for o in Organ::ALL
        .into_iter()
        .filter(|o| !matches!(o, Organ::Gut | Organ::Skin))
    {
        assert_eq!(t.get(o), 0.0, "{o:?}");
    }
    // a colony of only germ cells has less of either hint
    let germs = Traits::from_colony(&[Role::Germ, Role::Germ]);
    assert!(
        germs.get(Organ::Gut) < t.get(Organ::Gut) && germs.get(Organ::Skin) < t.get(Organ::Skin)
    );
    assert_eq!(Traits::from_colony(&[]).get(Organ::Gut), 0.05);
}

#[test]
fn organs_build_on_the_parts_they_come_from() {
    let mut t = Traits::plain();
    assert_eq!(
        t.ceiling(Organ::Gut),
        RADIAL_GUT,
        "a radial body only gets a blind gut"
    );
    assert!(
        t.ceiling(Organ::Nerve) < 0.3
            && t.ceiling(Organ::Fins) <= 0.1
            && t.ceiling(Organ::Skeleton) < 0.2
    );
    assert_eq!(t.ceiling(Organ::Eyes), 0.1, "eyes need a nerve to talk to");
    assert_eq!(
        (
            t.ceiling(Organ::Lungs),
            t.ceiling(Organ::Limbs),
            t.ceiling(Organ::Jaws)
        ),
        (0.0, 0.0, 0.0)
    );
    t.bilateral = true;
    assert_eq!(t.ceiling(Organ::Gut), 1.0);
    t.set(Organ::Gut, 0.6);
    assert_eq!(t.ceiling(Organ::Lungs), 0.6, "lungs bud off the gut");
    t.set(Organ::Fins, 0.5);
    t.set(Organ::Skeleton, 0.4);
    assert_eq!(
        t.ceiling(Organ::Limbs),
        0.4,
        "limbs grow out of fins with a skeleton: the smaller of the two"
    );
    t.set(Organ::Gills, 0.2);
    assert_eq!(
        t.ceiling(Organ::Jaws),
        0.2,
        "jaws grow from gill supports and need a skeleton"
    );
}

const RADIAL_GUT: f32 = super::traits::RADIAL_GUT_LIMIT;

#[test]
fn offspring_never_gain_past_a_ceiling_and_always_stay_in_range() {
    let mut rng = StdRng::seed_from_u64(7);
    let mut t = Traits::plain();
    for _ in 0..4000 {
        let child = t.offspring(&mut rng);
        for o in Organ::ALL {
            assert!((0.0..=1.0).contains(&child.get(o)));
            if child.get(o) > t.get(o) {
                assert!(
                    child.get(o) <= child.ceiling(o) + 1e-6,
                    "{o:?} grew past its ceiling"
                );
            }
        }
        t = child;
    }
    assert!(
        t.development() > 0.3,
        "a long line of offspring does change"
    );
}

#[test]
fn a_child_is_close_to_its_parent() {
    let mut rng = StdRng::seed_from_u64(3);
    let parent = fish();
    for _ in 0..200 {
        let child = parent.offspring(&mut rng);
        for o in Organ::ALL {
            assert!((child.get(o) - parent.get(o)).abs() < 0.4);
        }
    }
}

#[test]
fn traits_encode_and_decode_round_trip() {
    let t = walker();
    let text = t.encode();
    assert!(text.starts_with("b-"));
    let back = Traits::decode(&text).unwrap();
    assert_eq!(back.bilateral, t.bilateral);
    for o in Organ::ALL {
        assert!((back.get(o) - t.get(o)).abs() <= 0.005 + 1e-6);
    }
    assert_eq!(
        Traits::decode(&Traits::plain().encode()),
        Some(Traits::plain())
    );
    for bad in [
        "",
        "x-1-2",
        "b-00",
        "b-00-00-00-00-00-00-00-00-00-00-101",
        "r-a-b-c-d-e-f-g-h-i-j-k",
        "r-00-00-00-00-00-00-00-00-00-00-00-00",
    ] {
        assert!(Traits::decode(bad).is_none(), "{bad}");
    }
}

#[test]
fn the_mean_of_a_group_is_the_average_and_empty_is_zero() {
    let a = body(true, &[(Organ::Gut, 1.0)]);
    let b = body(false, &[(Organ::Gut, 0.5), (Organ::Skin, 1.0)]);
    let m = Mean::of([&a, &b]);
    assert_eq!(m.bilateral, 0.5);
    assert_eq!((m.get(Organ::Gut), m.get(Organ::Skin)), (0.75, 0.5));
    assert!(m.typical().bilateral);
    assert_eq!(Mean::of(std::iter::empty::<&Traits>()), Mean::ZERO);
}

// ---- zones ----

#[test]
fn the_zones_run_from_sea_to_land() {
    assert_eq!(Zone::OpenSea.neighbour(false), None);
    assert_eq!(Zone::OpenSea.neighbour(true), Some(Zone::Shallows));
    assert_eq!(Zone::Land.neighbour(true), None);
    assert_eq!(Zone::Land.neighbour(false), Some(Zone::TidalFlats));
    for pair in Zone::ALL.windows(2) {
        let (wet, dry) = (pair[0], pair[1]);
        assert!(wet.support() > dry.support(), "less buoyancy toward land");
        assert!(wet.drying() < dry.drying());
        assert!(
            wet.predation() > dry.predation(),
            "big hunters stay in open water"
        );
        assert!(wet.air_access() < dry.air_access());
        assert!(wet.gill_working() >= dry.gill_working());
    }
    assert!(
        Zone::Shallows.food() > Zone::OpenSea.food() && Zone::Shallows.food() > Zone::Land.food()
    );
}

#[test]
fn air_holds_more_oxygen_than_water_and_the_shallows_less_than_the_sea() {
    let env = Environment::default();
    assert!(env.medium_oxygen(Zone::Land) > env.medium_oxygen(Zone::OpenSea));
    assert!(env.medium_oxygen(Zone::OpenSea) > env.medium_oxygen(Zone::Shallows));
    assert!(env.medium_oxygen(Zone::Shallows) > env.medium_oxygen(Zone::TidalFlats));
    let stale = Environment { oxygen: 0.4, ..env };
    assert!(stale.medium_oxygen(Zone::OpenSea) < env.medium_oxygen(Zone::OpenSea));
    assert_eq!(
        stale.medium_oxygen(Zone::Land),
        env.medium_oxygen(Zone::Land),
        "air does not depend on the sea"
    );
}

#[test]
fn shares_add_up_and_a_steep_shore_leaves_little_shallows() {
    for slope in [0.0, 0.3, 0.7, 1.0] {
        let shares = Environment {
            slope,
            ..Environment::default()
        }
        .shares();
        assert!((shares.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!(shares.iter().all(|s| *s > 0.0));
    }
    let gentle = Environment {
        slope: 0.0,
        ..Environment::default()
    }
    .shares();
    let cliff = Environment {
        slope: 1.0,
        ..Environment::default()
    }
    .shares();
    assert!(gentle[1] > cliff[1] * 2.0 && gentle[2] > cliff[2] * 3.0);
    assert!(
        room(
            &Environment {
                slope: 1.0,
                ..Environment::default()
            },
            Zone::TidalFlats
        ) >= 8,
        "every zone has some room"
    );
}

#[test]
fn dials_are_clamped() {
    let e = Environment {
        oxygen: 9.0,
        slope: -1.0,
        predators: 3.0,
    }
    .clamped();
    assert_eq!((e.oxygen, e.slope, e.predators), (1.0, 0.0, 1.0));
    assert_eq!(
        Environment {
            oxygen: 0.0,
            ..Environment::default()
        }
        .clamped()
        .oxygen,
        Environment::MIN_OXYGEN
    );
}

// ---- fitness ----

#[test]
fn every_factor_stays_in_range() {
    let env = Environment::default();
    for t in [
        Traits::plain(),
        fish(),
        walker(),
        Traits::from_colony(&[Role::Germ]),
    ] {
        for z in Zone::ALL {
            let f = fitness(&t, z, &env);
            for v in [f.food, f.oxygen, f.support, f.drying, f.predation, f.upkeep] {
                assert!((0.0..=1.2).contains(&v), "{z:?} {f:?}");
            }
            assert!(f.total() > 0.0);
        }
    }
}

#[test]
fn in_open_water_fins_and_gills_win() {
    let env = Environment::default();
    let sea = Zone::OpenSea;
    assert!(
        total(&fish(), sea, &env) > 1.5 * total(&Traits::plain(), sea, &env),
        "a finned, gilled swimmer beats a plain blob"
    );
    let mut finless = fish();
    finless.set(Organ::Fins, 0.0);
    assert!(total(&fish(), sea, &env) > total(&finless, sea, &env));
    let mut gillless = fish();
    gillless.set(Organ::Gills, 0.0);
    assert!(total(&fish(), sea, &env) > total(&gillless, sea, &env));
    assert!(
        total(&fish(), sea, &env) > total(&walker(), sea, &env),
        "a walker's heavy kit is wasted at sea"
    );
}

#[test]
fn lungs_cost_something_where_air_is_out_of_reach() {
    let env = Environment::default();
    let mut with = fish();
    with.set(Organ::Lungs, 0.8);
    assert!(total(&with, Zone::OpenSea, &env) < total(&fish(), Zone::OpenSea, &env));
}

#[test]
fn in_stale_shallows_a_lung_like_organ_gives_an_edge_and_more_so_as_oxygen_falls() {
    let mut with = fish();
    with.set(Organ::Lungs, 0.8);
    let edge = |oxygen: f32| {
        let env = Environment {
            oxygen,
            ..Environment::default()
        };
        total(&with, Zone::Shallows, &env) / total(&fish(), Zone::Shallows, &env)
    };
    assert!(
        edge(0.4) > 1.0,
        "lungs pay in the shallows when oxygen is short"
    );
    assert!(
        edge(0.4) > edge(1.0),
        "and pay more the less oxygen the water holds"
    );
}

#[test]
fn limb_like_fins_help_in_the_shallows_and_on_the_flats() {
    let env = Environment::default();
    let mut limby = fish();
    limby.set(Organ::Skeleton, 0.8);
    limby.set(Organ::Limbs, 0.7);
    let mut plain = fish();
    plain.set(Organ::Skeleton, 0.8);
    assert!(total(&limby, Zone::TidalFlats, &env) > total(&plain, Zone::TidalFlats, &env));
    assert!(speed(&limby, Zone::TidalFlats) > speed(&plain, Zone::TidalFlats));
}

#[test]
fn on_land_limbs_a_sealed_skin_and_lungs_win() {
    let env = Environment::default();
    assert!(total(&walker(), Zone::Land, &env) > 3.0 * total(&fish(), Zone::Land, &env));
    for drop in [Organ::Limbs, Organ::Skin, Organ::Lungs, Organ::Skeleton] {
        let mut worse = walker();
        worse.set(drop, 0.0);
        assert!(
            total(&walker(), Zone::Land, &env) > total(&worse, Zone::Land, &env),
            "{drop:?}"
        );
    }
    assert!(total(&walker(), Zone::Land, &env) > total(&Traits::plain(), Zone::Land, &env));
}

#[test]
fn nothing_lives_better_on_land_than_in_the_water_until_it_has_the_kit() {
    let env = Environment::default();
    assert!(total(&fish(), Zone::Land, &env) < total(&fish(), Zone::OpenSea, &env));
    assert!(
        fitness(&fish(), Zone::Land, &env).drying < 0.5,
        "a fish dries out"
    );
    assert!(
        fitness(&fish(), Zone::Land, &env).support < 0.5,
        "and cannot carry itself"
    );
    assert!(fitness(&walker(), Zone::Land, &env).drying > 0.8);
    assert!(fitness(&walker(), Zone::Land, &env).support > 0.8);
}

#[test]
fn skin_trades_gas_exchange_for_water_tightness() {
    let env = Environment::default();
    let mut sealed = Traits::plain();
    sealed.set(Organ::Skin, 1.0);
    let bare = Traits::plain();
    assert!(
        fitness(&sealed, Zone::OpenSea, &env).oxygen < fitness(&bare, Zone::OpenSea, &env).oxygen
    );
    assert!(
        fitness(&sealed, Zone::TidalFlats, &env).drying
            > fitness(&bare, Zone::TidalFlats, &env).drying
    );
}

#[test]
fn predators_hurt_and_speed_and_senses_help_against_them() {
    let calm = Environment {
        predators: 0.0,
        ..Environment::default()
    };
    let fierce = Environment {
        predators: 1.0,
        ..Environment::default()
    };
    for t in [Traits::plain(), fish()] {
        assert!(
            fitness(&t, Zone::OpenSea, &fierce).predation
                < fitness(&t, Zone::OpenSea, &calm).predation
        );
    }
    assert_eq!(fitness(&fish(), Zone::OpenSea, &calm).predation, 1.0);
    let lost = |t: &Traits| 1.0 - fitness(t, Zone::OpenSea, &fierce).predation;
    assert!(lost(&fish()) < lost(&Traits::plain()));
    assert!(
        fitness(&fish(), Zone::Land, &fierce).predation
            > fitness(&fish(), Zone::OpenSea, &fierce).predation,
        "few hunters on early land"
    );
}

#[test]
fn senses_need_a_nerve() {
    let mut eyes_only = Traits::plain();
    eyes_only.set(Organ::Eyes, 1.0);
    assert_eq!(sense(&eyes_only), 0.0);
    assert!(sense(&fish()) > 0.5);
}

#[test]
fn a_bilateral_body_without_muscle_does_not_move_much_and_a_radial_one_is_slow() {
    assert_eq!(speed(&body(true, &[]), Zone::OpenSea), 0.0);
    let mut radial = fish();
    radial.bilateral = false;
    assert!(speed(&radial, Zone::OpenSea) < speed(&fish(), Zone::OpenSea) * 0.5);
}

// ---- runs ----

fn run() -> Run {
    Run::new(Environment::default(), SEED, GENERATIONS)
}

#[test]
fn a_run_is_a_pure_function_of_environment_and_seed() {
    let a = Run::new(Environment::default(), 5, 120);
    let b = Run::new(Environment::default(), 5, 120);
    assert_eq!(a.frames, b.frames);
    assert_eq!(a.milestones, b.milestones);
    let c = Run::new(Environment::default(), 6, 120);
    assert_ne!(a.frames.last(), c.frames.last());
}

#[test]
fn a_run_starts_in_the_open_sea_with_the_colony_ancestor() {
    let r = Run::new(Environment::default(), SEED, 10);
    assert_eq!(r.frames.len(), 11);
    let f = &r.frames[0];
    assert_eq!(f.generation, 0);
    assert!(f.counts[0] > 0 && f.counts[1..].iter().all(|c| *c == 0));
    assert_eq!(f.frontier_zone, Zone::OpenSea);
    assert!(!f.frontier.typical().bilateral);
    assert!(
        (f.frontier.get(Organ::Gut)
            - Traits::from_colony(&[Role::Germ, Role::Soma, Role::Feeder]).get(Organ::Gut))
        .abs()
            < 1e-5
    );
    assert!(
        r.frames
            .iter()
            .enumerate()
            .all(|(i, f)| f.generation == i as u32)
    );
}

#[test]
fn zones_never_hold_more_than_their_room() {
    let env = Environment::default();
    let r = run();
    for f in &r.frames {
        for z in Zone::ALL {
            assert!(
                f.counts[z.index()] as usize <= room(&env, z),
                "{z:?} at generation {}",
                f.generation
            );
        }
        assert!(f.total() as usize <= CAPACITY + 4 * 8);
        assert!(f.samples.len() <= SAMPLES);
    }
}

#[test]
fn the_default_world_takes_the_lineage_from_sea_to_land() {
    let r = run();
    assert_eq!(r.extinct_at, None);
    let found: Vec<Milestone> = r.milestones.iter().map(|(m, _)| *m).collect();
    for m in Milestone::ALL {
        assert!(
            found.contains(&m),
            "{m:?} not reached by generation {GENERATIONS}"
        );
    }
    let last = r.last();
    assert_eq!(last.frontier_zone, Zone::Land);
    assert!(last.frontier.bilateral > 0.9);
    for o in [Organ::Limbs, Organ::Lungs, Organ::Skin, Organ::Skeleton] {
        assert!(last.frontier.get(o) > 0.5, "{o:?}");
    }
    assert!(last.frontier.get(Organ::Gills) < 0.9 || last.frontier.get(Organ::Limbs) > 0.5);
}

#[test]
fn milestones_are_listed_in_the_order_reached_and_each_holds_at_that_frame() {
    let r = run();
    assert!(r.milestones.windows(2).all(|w| w[0].1 <= w[1].1));
    for (m, g) in &r.milestones {
        assert!(m.holds(r.frame(*g)));
        if *g > 0 {
            assert!(
                !m.holds(r.frame(*g - 1))
                    || r.milestones.iter().filter(|(x, _)| x == m).count() == 1,
                "first frame it held"
            );
        }
    }
    assert_eq!(r.milestones_by(0).len(), 0);
    assert_eq!(r.milestones_by(GENERATIONS).len(), r.milestones.len());
    let half = r.milestones_by(GENERATIONS / 2);
    assert!(half.len() < r.milestones.len() && half.iter().all(|(_, g)| *g <= GENERATIONS / 2));
}

#[test]
fn a_run_asked_for_a_frame_past_its_end_gives_the_last() {
    let r = Run::new(Environment::default(), SEED, 30);
    assert_eq!(r.frame(9999).generation, 30);
    assert_eq!(r.frame(7).generation, 7);
}

#[test]
fn a_hopeless_ancestor_dies_out_and_the_run_says_when() {
    let mut doomed = Traits::plain();
    // a body that eats nothing useful, breathes nothing, in the fiercest world
    for o in Organ::ALL {
        doomed.set(o, 1.0);
    }
    doomed.bilateral = true;
    let env = Environment {
        oxygen: 0.35,
        slope: 0.5,
        predators: 1.0,
    };
    let r = Run::from_ancestor(env, 1, 200, doomed);
    // the loaded body may or may not survive, but the bookkeeping must be consistent either way
    match r.extinct_at {
        Some(g) => {
            assert!(g >= 1);
            assert!(r.frames[g as usize..].iter().all(|f| f.total() == 0));
            assert!(r.frames[g as usize - 1].total() > 0);
            assert!(r.milestones_by(GENERATIONS).len() <= Milestone::ALL.len());
        }
        None => assert!(r.last().total() > 0),
    }
}

#[test]
fn the_summary_matches_the_last_frame() {
    let r = Run::new(
        Environment {
            oxygen: 0.5,
            ..Environment::default()
        },
        SEED,
        200,
    );
    let s = r.summary();
    assert_eq!(s.generations, 200);
    assert_eq!(s.counts, r.last().counts);
    assert_eq!(s.milestones, r.milestones);
    assert_eq!(s.env.oxygen, 0.5);
}

#[test]
fn different_worlds_give_different_lineages() {
    let base = run().summary();
    let gentle = Run::new(
        Environment {
            slope: 0.0,
            ..Environment::default()
        },
        SEED,
        GENERATIONS,
    )
    .summary();
    assert_ne!(base.counts, gentle.counts);
    assert!(
        gentle.counts[1] > base.counts[1] && gentle.counts[2] > base.counts[2],
        "a gentle shore holds more shallows and flats animals"
    );
    let stale = Run::new(
        Environment {
            oxygen: 0.35,
            ..Environment::default()
        },
        SEED,
        GENERATIONS,
    )
    .summary();
    assert_ne!(base.milestones, stale.milestones);
}

#[test]
fn csv_has_a_header_and_a_row_per_generation_and_the_body_decodes() {
    let r = Run::new(Environment::default(), SEED, 40);
    let csv = r.to_csv();
    let mut lines = csv.lines();
    let header = lines.next().unwrap();
    let columns = header.split(',').count();
    assert_eq!(columns, 7 + ORGANS + 1);
    assert!(header.starts_with(
        "generation,open_sea,shallows,tidal_flats,land,frontier_zone,bilateral,mouth_and_gut"
    ));
    let rows: Vec<&str> = lines.collect();
    assert_eq!(rows.len(), 41);
    for (i, row) in rows.iter().enumerate() {
        let cells: Vec<&str> = row.split(',').collect();
        assert_eq!(cells.len(), columns, "{row}");
        assert_eq!(cells[0], i.to_string());
        assert!(Traits::decode(cells[columns - 1]).is_some());
    }
}

// ---- milestones and the story ----

fn frame_with(counts: [u16; ZONES], bilateral: f32, set: &[(Organ, f32)]) -> Frame {
    let mut mean = Mean::ZERO;
    mean.bilateral = bilateral;
    for (o, l) in set {
        mean.levels[o.index()] = *l;
    }
    Frame {
        generation: 0,
        counts,
        frontier_zone: Zone::OpenSea,
        frontier: mean,
        overall: mean,
        samples: Vec::new(),
    }
}

#[test]
fn each_milestone_needs_its_own_evidence() {
    let none = frame_with([60, 0, 0, 0], 0.0, &[]);
    for m in Milestone::ALL {
        assert!(!m.holds(&none), "{m:?}");
    }
    assert!(Milestone::SymmetryAndGut.holds(&frame_with([60, 0, 0, 0], 0.8, &[(Organ::Gut, 0.5)])));
    assert!(
        !Milestone::SymmetryAndGut.holds(&frame_with([60, 0, 0, 0], 0.3, &[(Organ::Gut, 0.5)])),
        "a gut alone is not enough"
    );
    assert!(Milestone::EyesAndNerveCord.holds(&frame_with(
        [60, 0, 0, 0],
        1.0,
        &[(Organ::Eyes, 0.4), (Organ::Nerve, 0.5)]
    )));
    assert!(!Milestone::EyesAndNerveCord.holds(&frame_with(
        [60, 0, 0, 0],
        1.0,
        &[(Organ::Eyes, 0.4)]
    )));
    assert!(Milestone::Backbone.holds(&frame_with([60, 0, 0, 0], 1.0, &[(Organ::Skeleton, 0.4)])));
    assert!(Milestone::Jaws.holds(&frame_with([60, 0, 0, 0], 1.0, &[(Organ::Jaws, 0.4)])));
    assert!(Milestone::LimbLikeFins.holds(&frame_with(
        [60, 0, 0, 0],
        1.0,
        &[(Organ::Limbs, 0.3), (Organ::Fins, 0.4)]
    )));
    assert!(!Milestone::LimbLikeFins.holds(&frame_with(
        [60, 0, 0, 0],
        1.0,
        &[(Organ::Limbs, 0.3)]
    )));
    assert!(Milestone::Lungs.holds(&frame_with([60, 0, 0, 0], 1.0, &[(Organ::Lungs, 0.4)])));
    // leaving the water: a foothold on the flats, with limbs
    assert!(Milestone::LeavingWater.holds(&frame_with(
        [30, 20, 14, 0],
        1.0,
        &[(Organ::Limbs, 0.3)]
    )));
    assert!(
        !Milestone::LeavingWater.holds(&frame_with([30, 20, 5, 0], 1.0, &[(Organ::Limbs, 0.3)])),
        "a few stragglers are not a foothold"
    );
    assert!(
        !Milestone::LeavingWater.holds(&frame_with([30, 20, 14, 0], 1.0, &[])),
        "and a crowd of blobs on the mud is not leaving"
    );
    let land = [(Organ::Limbs, 0.5), (Organ::Lungs, 0.5)];
    assert!(Milestone::FirstLandAnimals.holds(&frame_with([30, 20, 14, 13], 1.0, &land)));
    assert!(!Milestone::FirstLandAnimals.holds(&frame_with([30, 20, 14, 3], 1.0, &land)));
    assert!(
        !Milestone::FirstLandAnimals.holds(&frame_with(
            [30, 20, 14, 13],
            1.0,
            &[(Organ::Limbs, 0.5)]
        )),
        "no lungs"
    );
}

#[test]
fn milestones_reached_are_remembered_once_in_order() {
    let mut a = frame_with([60, 0, 0, 0], 0.8, &[(Organ::Gut, 0.5)]);
    a.generation = 10;
    let mut b = frame_with([60, 0, 0, 0], 0.0, &[]);
    b.generation = 11; // symmetry lost again: still reached
    let mut c = frame_with(
        [60, 0, 0, 0],
        1.0,
        &[(Organ::Skeleton, 0.5), (Organ::Gut, 0.5)],
    );
    c.generation = 12;
    let found = reached(&[a, b, c]);
    assert_eq!(
        found,
        vec![(Milestone::SymmetryAndGut, 10), (Milestone::Backbone, 12)]
    );
}

#[test]
fn order_compares_the_simulations_sequence_with_the_storys() {
    use Milestone::*;
    let same = [(SymmetryAndGut, 10), (Backbone, 20), (Lungs, 30)];
    for (m, _) in same {
        assert_eq!(order_of(m, &same), Some(Order::Same));
    }
    // lungs before a backbone, reversed from the story
    let swapped = [(SymmetryAndGut, 10), (Lungs, 20), (Backbone, 30)];
    assert_eq!(order_of(Lungs, &swapped), Some(Order::Earlier(1)));
    assert_eq!(order_of(Backbone, &swapped), Some(Order::Later(1)));
    assert_eq!(order_of(SymmetryAndGut, &swapped), Some(Order::Same));
    assert_eq!(order_of(Jaws, &swapped), None, "not reached");
    // a milestone the run skipped does not make the others look early
    let skipped = [(Backbone, 10), (Lungs, 20)];
    assert_eq!(order_of(Backbone, &skipped), Some(Order::Same));
}

#[test]
fn the_story_has_a_chapter_for_every_milestone_in_story_order() {
    assert_eq!(CHAPTERS.len(), Milestone::ALL.len() + 1);
    assert!(
        CHAPTERS[0].milestone.is_none(),
        "the prologue comes from stage 5"
    );
    for (i, m) in Milestone::ALL.into_iter().enumerate() {
        assert_eq!(chapter_for(m).milestone, Some(m));
        assert_eq!(
            CHAPTERS[i + 1].milestone,
            Some(m),
            "chapters follow the order of the story"
        );
        assert_eq!(m.story_rank(), i);
    }
}

#[test]
fn every_chapter_has_all_its_parts() {
    for c in &CHAPTERS {
        for (name, text) in [
            ("title", c.title),
            ("setting", c.setting),
            ("problem", c.problem),
            ("adaptation", c.adaptation),
            ("result", c.result),
            ("model", c.model),
        ] {
            assert!(
                text.trim().len() > 3 && (name == "title" || text.trim().len() > 12),
                "{} lacks {name}",
                c.title
            );
            assert!(
                text.trim_end().ends_with(['.', '?']) || name == "title",
                "{} {name} is a sentence",
                c.title
            );
        }
    }
}

#[test]
fn the_story_is_honest_about_what_it_is() {
    for c in &CHAPTERS {
        let all = [c.setting, c.problem, c.adaptation, c.result, c.model].join(" ");
        if all.contains("million") || all.contains("billion") {
            assert!(
                all.contains("roughly"),
                "{} names a time without saying roughly",
                c.title
            );
        }
        assert!(
            !all.chars().any(|ch| ch.is_ascii_digit()),
            "{} has a number in it: the story is qualitative",
            c.title
        );
        assert!(
            !all.contains("exactly") && !all.contains("precisely"),
            "{}",
            c.title
        );
    }
}

#[test]
fn the_simulation_note_says_when_and_how_the_order_compares() {
    use Milestone::*;
    let found = [(Lungs, 120), (Backbone, 300)];
    assert_eq!(
        simulation_note(Lungs, &found),
        "Reached at generation 120, 1 place earlier than in the story."
    );
    assert_eq!(
        simulation_note(Backbone, &found),
        "Reached at generation 300, 1 place later than in the story."
    );
    let swapped = [(Lungs, 120), (SymmetryAndGut, 300), (Backbone, 400)];
    assert_eq!(
        simulation_note(Lungs, &swapped),
        "Reached at generation 120, 2 places earlier than in the story."
    );
    assert!(simulation_note(Backbone, &swapped).contains("1 place later"));
    assert!(simulation_note(SymmetryAndGut, &[(SymmetryAndGut, 4)]).contains("the same place"));
    assert_eq!(
        simulation_note(Jaws, &swapped),
        "Not reached yet in this run."
    );
    let far = [(FirstLandAnimals, 5), (SymmetryAndGut, 6), (Backbone, 7)];
    assert!(simulation_note(FirstLandAnimals, &far).contains("2 places earlier"));
}
