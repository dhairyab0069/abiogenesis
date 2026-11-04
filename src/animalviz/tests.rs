use super::*;
use crate::animals::{DEFAULT_SEED, Environment, GENERATIONS, Run, Zone};

fn fish() -> Traits {
    let mut t = Traits::plain();
    t.bilateral = true;
    for (o, l) in [
        (Organ::Gut, 0.9),
        (Organ::Nerve, 0.7),
        (Organ::Eyes, 0.6),
        (Organ::Muscle, 0.8),
        (Organ::Fins, 0.8),
        (Organ::Skeleton, 0.7),
        (Organ::Gills, 0.9),
        (Organ::Jaws, 0.6),
    ] {
        t.set(o, l);
    }
    t
}

fn walker() -> Traits {
    let mut t = fish();
    for (o, l) in [
        (Organ::Limbs, 0.9),
        (Organ::Fins, 0.2),
        (Organ::Lungs, 0.8),
        (Organ::Skin, 0.9),
    ] {
        t.set(o, l);
    }
    t
}

#[test]
fn an_organ_fades_in_then_grows() {
    assert_eq!(visibility(0.0), 0.0);
    assert_eq!(visibility(APPEAR_AT - 0.001), 0.0);
    assert!(visibility(APPEAR_AT + 0.01) > 0.0 && visibility(APPEAR_AT + 0.01) < 0.5);
    assert_eq!(visibility(OPAQUE_AT), 1.0);
    assert_eq!(visibility(1.0), 1.0);
    let mut last = 0.0;
    for k in 0..=100 {
        let v = visibility(k as f32 / 100.0);
        assert!(v >= last, "visibility never falls as the level rises");
        last = v;
    }
    assert_eq!(size(0.0), 0.0);
    assert!(size(0.1) > 0.35 && size(1.0) == 1.0 && size(0.5) < size(0.9));
}

#[test]
fn the_backbone_goes_from_nothing_to_a_rod_to_bone() {
    assert_eq!(backbone(0.1), Backbone::None);
    assert_eq!(backbone(0.3), Backbone::Notochord);
    assert_eq!(backbone(0.7), Backbone::Bone);
    let mut t = fish();
    t.set(Organ::Skeleton, 0.4);
    assert!(
        backbone_line(&t, 0.0).1.is_empty(),
        "no vertebrae before bone"
    );
    t.set(Organ::Skeleton, 0.8);
    assert!(!backbone_line(&t, 0.0).1.is_empty());
}

#[test]
fn a_bilateral_body_has_a_head_and_a_tail_and_mirrors_across_its_axis() {
    let t = fish();
    let pts = outline(&t, 0.0, 20);
    let head = pts.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let tail = pts.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    assert!((head - 0.5).abs() < 1e-5 && (tail + 0.5).abs() < 1e-5);
    // back and belly are mirror images about the spine, which itself only sways
    let n = pts.len() / 2;
    for k in 0..n {
        let back = pts[k];
        let belly = pts[pts.len() - 1 - k];
        assert!((back.x - belly.x).abs() < 1e-5);
        let s = k as f32 / (n - 1) as f32;
        assert!(((back.y + belly.y) / 2.0 - undulation(&t, s, 0.0)).abs() < 1e-5);
    }
    assert!(
        half_height(&t, 0.0) < half_height(&t, 0.4) && half_height(&t, 1.0) < half_height(&t, 0.4),
        "fullest in the middle, pointed at both ends"
    );
    let strong = {
        let mut m = t;
        m.set(Organ::Muscle, 1.0);
        m
    };
    assert!(half_height(&strong, 0.4) > half_height(&Traits::plain(), 0.4));
}

#[test]
fn the_tail_swings_most_and_the_head_stays_put() {
    let t = fish();
    assert!(undulation(&t, 1.0, 1.0).abs() < 1e-6);
    let swing = |s: f32| {
        (0..40)
            .map(|k| undulation(&t, s, k as f32 * 0.2).abs())
            .fold(0.0, f32::max)
    };
    assert!(swing(0.0) > swing(0.5) && swing(0.5) > swing(0.9));
    assert!(
        swing(0.0) > undulation(&Traits::plain(), 0.0, 1.0).abs(),
        "muscle swings it harder"
    );
}

#[test]
fn a_radial_body_is_round_and_has_arms_all_around() {
    let t = Traits::plain();
    let r = radial_radius(&t, 0.0);
    assert!((0.2..0.4).contains(&r));
    for k in 0..ARMS {
        let a = arm(&t, k, 0.0, 6);
        assert_eq!(a.len(), 7);
        assert!((a[0].len() - r).abs() < 1e-4, "arms start on the rim");
        assert!(a.last().unwrap().len() > r, "and reach outward");
    }
    // rotationally symmetric: the arm roots are evenly spread around
    let roots: Vec<f32> = (0..ARMS)
        .map(|k| arm(&t, k, 0.0, 4)[0])
        .map(|p| p.y.atan2(p.x))
        .collect();
    for pair in roots.windows(2) {
        let gap = (pair[1] - pair[0]).rem_euclid(std::f32::consts::TAU);
        assert!((gap - std::f32::consts::TAU / ARMS as f32).abs() < 1e-3);
    }
}

#[test]
fn organs_sit_where_the_story_says() {
    let t = fish();
    let (brain_at, brain_r) = brain(&t, 0.0);
    let (eye_at, eye_r) = eye(&t, 0.0);
    assert!(
        brain_at.x > 0.2 && eye_at.x > 0.2,
        "brain and eyes are at the head end"
    );
    assert!(brain_r > 0.02 && eye_r > 0.02);
    let nerve = nerve_line(&t, 0.0);
    let gut = gut_line(&t, 0.0);
    assert!(
        nerve[0].x > nerve[8].x,
        "the nerve cord runs from the head back"
    );
    assert!(
        nerve.iter().zip(&gut).all(|(n, g)| n.y < g.y),
        "the nerve cord is above the gut"
    );
    assert!(gut[0].x > 0.3, "the gut starts at the mouth");
    assert!(gut[0].x > gut[8].x);
    let slits = gill_slits(&t, 0.0);
    assert!(
        slits.len() >= 3
            && slits
                .iter()
                .all(|s| s[0].x < 0.3 && s[0].x > -0.1 && s[0].y < s[1].y),
        "gills sit behind the head, back to belly"
    );
    let (lung_at, radii) = lung(&walker(), 0.0);
    assert!(lung_at.y < gut[3].y + 0.1 && radii.x > 0.05);
    assert!(lung(&walker(), 0.0).1.x > lung(&fish(), 0.0).1.x);
}

#[test]
fn a_blind_gut_is_short_and_a_through_gut_reaches_the_tail_end() {
    assert!(
        gut_line(&Traits::plain(), 0.0).is_empty(),
        "a radial animal has a pouch, not a tube, in the picture"
    );
    let mut short = fish();
    short.set(Organ::Gut, 0.4);
    let mut full = fish();
    full.set(Organ::Gut, 1.0);
    let reach = |t: &Traits| gut_line(t, 0.0).last().unwrap().x;
    assert!(reach(&full) < reach(&short));
    assert!(reach(&full) < -0.35);
}

#[test]
fn nerves_grow_longer_and_the_brain_swells() {
    let mut a = fish();
    a.set(Organ::Nerve, 0.3);
    let mut b = fish();
    b.set(Organ::Nerve, 0.9);
    assert!(nerve_line(&b, 0.0).last().unwrap().x < nerve_line(&a, 0.0).last().unwrap().x);
    assert!(brain(&b, 0.0).1 > brain(&a, 0.0).1);
}

#[test]
fn jaws_start_as_a_plain_mouth_then_open_and_grow_teeth() {
    let mut t = fish();
    t.set(Organ::Jaws, 0.0);
    let plain = jaws(&t, 1.0, 0.0);
    assert!(!plain.teeth);
    assert_eq!(plain.upper, plain.lower, "no jaws: the mouth does not open");
    t.set(Organ::Jaws, 0.6);
    let shut = jaws(&t, 0.0, 0.0);
    let wide = jaws(&t, 1.0, 0.0);
    assert!(wide.teeth);
    assert!(
        wide.lower[2].y - wide.upper[2].y > shut.lower[2].y - shut.upper[2].y,
        "biting opens the jaws"
    );
    assert!(wide.upper[0] == wide.lower[0], "both jaws share the hinge");
}

#[test]
fn a_fin_becomes_a_limb() {
    assert!(limb_shape(0.0, 0.0).is_none());
    assert!(limb_shape(0.02, 0.02).is_none());
    let fin = limb_shape(0.8, 0.0).unwrap();
    assert_eq!((fin.joints, fin.toes), (0, false));
    assert!(fin.web > 0.9 && fin.blend < 0.05);
    let mid = limb_shape(0.6, 0.4).unwrap();
    assert!(
        mid.joints >= 1 && mid.web < fin.web && mid.web > 0.0,
        "a lobe fin: jointed and still webbed"
    );
    let limb = limb_shape(0.1, 0.95).unwrap();
    assert_eq!(limb.joints, 2);
    assert!(limb.toes && limb.web < 0.05);
    assert!(limb.length > fin.length, "limbs are longer than fins");
    // the change is gradual
    let mut last = 0.0;
    for k in 0..=20 {
        let b = limb_shape(0.5, k as f32 / 20.0 * 0.9 + 0.05).unwrap().blend;
        assert!(b >= last);
        last = b;
    }
}

#[test]
fn a_walking_limb_reaches_the_ground_and_a_fin_sweeps_back() {
    let limb = limb_shape(0.1, 0.9).unwrap();
    let attach = Pt::new(0.1, 0.1);
    let [a, e, tip] = limb_chain(&limb, attach, 0.0, true);
    assert_eq!(a, attach);
    assert!(
        tip.y > attach.y + limb.length * 0.7,
        "the foot is well below the shoulder"
    );
    assert!(e.y > a.y && e.y < tip.y);
    let step = (0..8)
        .map(|k| limb_chain(&limb, attach, k as f32 * 0.8, true)[2].x)
        .fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
    assert!(step.1 - step.0 > 0.05, "the foot swings as it walks");
    let fin = limb_shape(0.8, 0.0).unwrap();
    let [fa, _, ftip] = limb_chain(&fin, attach, 0.0, false);
    assert!(ftip.x < fa.x, "a fin sweeps back");
    let flap = (0..8)
        .map(|k| limb_chain(&fin, attach, k as f32 * 0.8, false)[2].y)
        .fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(y), hi.max(y)));
    assert!(flap.1 > flap.0);
}

#[test]
fn only_a_limbed_skeletal_animal_stands() {
    let limb = limb_shape(0.1, 0.9).unwrap();
    assert_eq!(stand_height(None, 0.0, 0.0), 0.0);
    assert_eq!(
        stand_height(Some(&limb), 0.2, 0.9),
        0.0,
        "weak limbs do not lift the body"
    );
    assert!(
        stand_height(Some(&limb), 0.9, 0.9) > stand_height(Some(&limb), 0.9, 0.2),
        "a stronger skeleton holds it higher"
    );
    assert!(stand_height(Some(&limb), 0.9, 0.9) > 0.1);
}

#[test]
fn the_tail_fin_and_dorsal_fin_grow_with_the_fin_level() {
    let mut small = fish();
    small.set(Organ::Fins, 0.1);
    let big = fish();
    let span = |f: [Pt; 4]| f[3].y - f[1].y;
    assert!(span(tail_fin(&big, 0.0)) > span(tail_fin(&small, 0.0)));
    assert!(
        dorsal_fin(&big, 0.0)[1].y < dorsal_fin(&small, 0.0)[1].y,
        "a taller back fin sticks up (smaller y)"
    );
}

#[test]
fn muscle_blocks_multiply_with_the_muscle_level() {
    let mut weak = fish();
    weak.set(Organ::Muscle, 0.1);
    let strong = fish();
    assert!(muscle_blocks(&strong, 0.0).len() > muscle_blocks(&weak, 0.0).len());
    weak.set(Organ::Muscle, 0.0);
    assert!(muscle_blocks(&weak, 0.0).is_empty());
}

// ---- the shore ----

#[test]
fn the_zones_tile_the_width_in_order() {
    let shore = Shore::new(&Environment::default());
    let mut x = 0.0;
    for z in Zone::ALL {
        let (a, b) = shore.span(z);
        assert!((a - x).abs() < 1e-6 && b > a, "{z:?}");
        assert_eq!(shore.zone_at((a + b) / 2.0), z);
        x = b;
    }
    assert_eq!(x, 1.0);
    assert_eq!(shore.zone_at(0.0), Zone::OpenSea);
    assert_eq!(shore.zone_at(2.0), Zone::Land);
}

#[test]
fn a_steep_shore_squeezes_the_shallows() {
    let gentle = Shore::new(&Environment {
        slope: 0.0,
        ..Environment::default()
    });
    let steep = Shore::new(&Environment {
        slope: 1.0,
        ..Environment::default()
    });
    let w = |s: &Shore, z| s.span(z).1 - s.span(z).0;
    assert!(w(&gentle, Zone::Shallows) > 2.0 * w(&steep, Zone::Shallows));
    assert!(w(&gentle, Zone::TidalFlats) > 2.0 * w(&steep, Zone::TidalFlats));
}

#[test]
fn the_ground_rises_from_the_deep_sea_to_dry_land() {
    let shore = Shore::new(&Environment::default());
    let mut last = f32::MAX;
    for k in 0..=100 {
        let g = shore.ground(k as f32 / 100.0);
        assert!(g <= last + 1e-5, "the ground never dips on the way to land");
        last = g;
    }
    assert!(shore.ground(0.0) > 0.85);
    assert!(shore.ground(1.0) < shore.water_level(0.0) + 0.1);
    let land = shore.span(Zone::Land);
    assert!(
        !shore.flooded((land.0 + land.1) / 2.0, 0.0) && !shore.flooded(0.9, 5.0),
        "land stays dry at every tide"
    );
    assert!(shore.flooded(0.1, 0.0), "the sea is always wet");
}

#[test]
fn the_tide_floods_and_drains_the_flats() {
    let shore = Shore::new(&Environment::default());
    let (a, b) = shore.span(Zone::TidalFlats);
    let x = (a + b) / 2.0;
    let states: Vec<bool> = (0..200).map(|k| shore.flooded(x, k as f32 * 0.1)).collect();
    assert!(
        states.iter().any(|f| *f) && states.iter().any(|f| !*f),
        "the middle of the flats is sometimes wet and sometimes dry"
    );
    let high = (0..200)
        .map(|k| shore.water_level(k as f32 * 0.1))
        .fold(f32::MIN, f32::max);
    let low = (0..200)
        .map(|k| shore.water_level(k as f32 * 0.1))
        .fold(f32::MAX, f32::min);
    assert!(high - low > TIDE_RANGE && high - low <= 2.0 * TIDE_RANGE + 1e-4);
}

#[test]
fn animals_are_placed_in_their_zone_and_the_right_medium() {
    let shore = Shore::new(&Environment::default());
    for z in Zone::ALL {
        let (a, b) = shore.span(z);
        for post in [
            Posture::Swim,
            Posture::Prop,
            Posture::Stand,
            Posture::Stranded,
        ] {
            for n in [1, 5, 14] {
                for k in 0..n {
                    for t in [0.0, 3.3, 9.1] {
                        let p = shore.place(z, k, n, t, post);
                        assert!(p.at.x >= a && p.at.x <= b, "{z:?} {post:?} {p:?}");
                        assert!(p.facing.abs() == 1.0);
                        let ground = shore.ground(p.at.x);
                        match post {
                            Posture::Swim => {
                                assert!(
                                    p.at.y > shore.water_level(t) - 0.001,
                                    "a swimmer is under the surface: {p:?}"
                                );
                                assert!(
                                    p.at.y <= ground.max(shore.water_level(t) + 0.06) + 0.02,
                                    "and above the floor: {p:?} {ground}"
                                );
                            }
                            _ => assert!(p.at.y < ground, "walkers rest on the ground: {p:?}"),
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_stranded_animal_does_not_move() {
    let shore = Shore::new(&Environment::default());
    let a = shore.place(Zone::Land, 2, 6, 0.0, Posture::Stranded);
    let b = shore.place(Zone::Land, 2, 6, 7.0, Posture::Stranded);
    assert_eq!(a.at.x, b.at.x);
    let swimmer_a = shore.place(Zone::OpenSea, 2, 6, 0.0, Posture::Swim);
    let swimmer_b = shore.place(Zone::OpenSea, 2, 6, 3.0, Posture::Swim);
    assert_ne!(swimmer_a.at, swimmer_b.at);
}

#[test]
fn posture_follows_the_body_and_the_zone() {
    assert_eq!(posture(&fish(), Zone::OpenSea), Posture::Swim);
    assert_eq!(
        posture(&walker(), Zone::OpenSea),
        Posture::Swim,
        "limbs do not stop an animal swimming in deep water"
    );
    assert_eq!(posture(&fish(), Zone::Shallows), Posture::Swim);
    assert_eq!(posture(&walker(), Zone::Shallows), Posture::Prop);
    assert_eq!(
        posture(&fish(), Zone::Land),
        Posture::Stranded,
        "a fish on land is stranded"
    );
    assert_eq!(posture(&walker(), Zone::Land), Posture::Stand);
    assert_eq!(posture(&walker(), Zone::TidalFlats), Posture::Stand);
}

#[test]
fn a_real_run_shows_animals_changing_from_swimmers_to_walkers() {
    let run = Run::new(Environment::default(), DEFAULT_SEED, GENERATIONS);
    let first = run.frame(0).frontier.typical();
    let last = run.last().frontier.typical();
    assert!(limb_shape(first.get(Organ::Fins), first.get(Organ::Limbs)).is_none());
    let shape = limb_shape(last.get(Organ::Fins), last.get(Organ::Limbs)).unwrap();
    assert!(shape.blend > 0.5, "the frontier's fins ended as limbs");
    assert!(gut_line(&last, 0.0).len() > gut_line(&first, 0.0).len());
    assert!(visibility(last.get(Organ::Lungs)) > visibility(first.get(Organ::Lungs)));
}
