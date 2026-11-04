use super::*;

#[test]
fn leaflets_face_the_right_way_and_tails_meet_in_the_middle() {
    let r = 40.0;
    let s = bilayer_spec(r);
    let glyphs = bilayer(r);
    assert!(glyphs.iter().any(|g| g.leaflet == Leaflet::Outer) && glyphs.iter().any(|g| g.leaflet == Leaflet::Inner));
    for g in &glyphs {
        let head = g.head.len();
        for (base, end) in g.tails {
            // a tail runs from the head toward the middle plane
            assert!((end.len() - s.mid).abs() < s.head_radius * 0.2, "tail ends at the middle");
            match g.leaflet {
                Leaflet::Outer => assert!(head > s.mid && base.len() > end.len() && head > base.len()),
                Leaflet::Inner => assert!(head < s.mid && base.len() < end.len() && head < base.len()),
            }
        }
        // the head sits on the ray from the centre at the glyph's angle
        let ray = Pt::polar(g.angle, head);
        assert!(ray.dist(g.head) < 1e-3);
    }
    assert!(s.inner_edge > 0.0 && s.inner_edge < s.mid && s.mid < r);
}

#[test]
fn outer_heads_point_out_and_inner_heads_point_in() {
    let s = bilayer_spec(30.0);
    for g in bilayer(30.0) {
        let out = g.head.len() > s.mid;
        assert_eq!(out, g.leaflet == Leaflet::Outer);
    }
}

#[test]
fn rings_are_evenly_spread_and_capped() {
    for r in [8.0, 22.0, 40.0, 120.0, 400.0] {
        let g = bilayer(r);
        let outer: Vec<&Glyph> = g.iter().filter(|g| g.leaflet == Leaflet::Outer).collect();
        let inner = g.len() - outer.len();
        assert!(outer.len() <= MAX_PER_LEAFLET && inner <= MAX_PER_LEAFLET && outer.len() >= 10);
        assert_eq!(g.len(), glyph_total(r));
        assert!(inner <= outer.len(), "the inner ring is the smaller one");
        let step = TAU / outer.len() as f32;
        assert!(outer.windows(2).all(|w| (w[1].angle - w[0].angle - step).abs() < 1e-4));
    }
}

#[test]
fn small_cells_and_a_spent_budget_get_a_smooth_line() {
    assert_eq!(detail(GLYPH_LOD_RADIUS - 1.0, 10_000), Detail::Smooth);
    assert_eq!(detail(GLYPH_LOD_RADIUS + 5.0, 10_000), Detail::Glyphs);
    let need = glyph_total(30.0);
    assert_eq!(detail(30.0, need), Detail::Glyphs);
    assert_eq!(detail(30.0, need - 1), Detail::Smooth);
    assert_eq!(detail(30.0, 0), Detail::Smooth);
    // the cap holds for the biggest cell: a fixed small total
    assert!(glyph_total(10_000.0) <= 2 * MAX_PER_LEAFLET);
}

#[test]
fn the_shimmer_is_small_and_bounded() {
    for i in 0..50 {
        for k in 0..20 {
            let (a, b) = wobble(i, k as f32 * 0.37);
            assert!(a.abs() <= 1.0 && b.abs() <= 1.0);
        }
    }
    assert_ne!(wobble(0, 1.0), wobble(1, 1.0));
}

#[test]
fn runs_are_closed_without_a_neighbour_and_split_by_one() {
    let runs = membrane_runs(Pt::new(0.0, 0.0), 10.0, 64, |_| false);
    assert_eq!(runs.len(), 1);
    assert!(runs[0][0].dist(*runs[0].last().unwrap()) < 1e-3);
    let other = Pt::new(14.0, 0.0);
    let hidden = |p: Pt| p.dist(other) < 10.0;
    let runs = membrane_runs(Pt::new(0.0, 0.0), 10.0, 64, hidden);
    assert!(runs.iter().all(|run| run.iter().all(|p| !hidden(*p))));
    let kept: usize = runs.iter().map(Vec::len).sum();
    assert!(kept < 65 && kept > 30);
    assert!(membrane_runs(Pt::new(0.0, 0.0), 10.0, 64, |_| true).is_empty());
}

#[test]
fn circles_meet_at_two_points_on_both() {
    let (a, b) = circle_intersection(Pt::new(0.0, 0.0), 10.0, Pt::new(12.0, 0.0), 8.0).unwrap();
    for p in [a, b] {
        assert!((p.len() - 10.0).abs() < 1e-3 && (p.dist(Pt::new(12.0, 0.0)) - 8.0).abs() < 1e-3);
    }
    assert!((a.x - b.x).abs() < 1e-3 && a.y * b.y < 0.0);
    assert!(circle_intersection(Pt::new(0.0, 0.0), 10.0, Pt::new(30.0, 0.0), 8.0).is_none());
    assert!(circle_intersection(Pt::new(0.0, 0.0), 10.0, Pt::new(1.0, 0.0), 2.0).is_none(), "one inside the other");
}

#[test]
fn formation_runs_for_a_fixed_time_then_stops() {
    assert_eq!(formation_progress(0.0), Some(0.0));
    assert!((formation_progress(FORMATION_STEPS / 2.0).unwrap() - 0.5).abs() < 1e-6);
    assert_eq!(formation_progress(FORMATION_STEPS), None);
    assert_eq!(formation_progress(-1.0), None);
}

#[test]
fn the_arc_grows_closes_and_only_then_fills() {
    let mut last = -1.0;
    for k in 0..=100 {
        let t = k as f32 / 100.0;
        let span = arc_span(t);
        assert!(span >= last && span <= TAU + 1e-5);
        last = span;
    }
    assert_eq!(arc_span(0.0), 0.0);
    assert!((arc_span(0.6) - TAU).abs() < 1e-5, "closed by 60%");
    assert_eq!(fill_alpha(0.65), 0.0, "nothing fills an open vesicle");
    assert_eq!(fill_alpha(1.0), 1.0);
    assert_eq!(inner_alpha(0.4), 0.0);
    assert_eq!(inner_alpha(0.75), 1.0);
    assert!(curl(0.0) > 1.5 && (curl(0.7) - 1.0).abs() < 1e-6, "the sheet curls shut");
}

#[test]
fn each_lipid_travels_from_where_it_was_to_its_place_and_turns_outward() {
    let glyphs = bilayer(30.0);
    let outer: Vec<Glyph> = glyphs.into_iter().filter(|g| g.leaflet == Leaflet::Outer).collect();
    let n = outer.len();
    let start = Pt::new(-50.0, 20.0);
    for i in [0, n / 2, n - 1] {
        let at0 = outer_pose(i, n, start, 7, &outer[i], 0.0);
        assert!(at0.pos.dist(start) < 1e-4, "starts where the lipid was");
        let end = outer_pose(i, n, start, 7, &outer[i], 1.0);
        assert!(end.pos.dist(outer[i].head) < 1e-3, "ends on the ring");
        assert!(turn(end.angle, outer[i].angle).abs() < 1e-3, "head faces outward");
        // halfway to its arrival it is between the two
        let t = arrival(i, n) / 2.0;
        let mid = outer_pose(i, n, start, 7, &outer[i], t);
        assert!(mid.pos.dist(start) > 1.0 && mid.pos.dist(outer[i].head * curl(t)) > 1.0);
    }
    assert!(arrival(0, n) < arrival(n - 1, n), "the ring fills in order");
    assert!(arrival(n - 1, n) <= 0.6 + 1e-6);
}

#[test]
fn the_same_lipid_always_starts_facing_the_same_way() {
    let g = &bilayer(30.0)[0];
    let a = outer_pose(0, 20, Pt::new(5.0, 5.0), 3, g, 0.0);
    let b = outer_pose(0, 20, Pt::new(5.0, 5.0), 3, g, 0.0);
    assert_eq!(a, b);
    assert_ne!(hash01(1, 2), hash01(1, 3));
    assert!((0.0..1.0).contains(&hash01(9, 9)));
}

#[test]
fn a_dividing_cell_elongates_pinches_and_splits() {
    let (m, d) = (Pt::new(0.0, 0.0), Pt::new(50.0, 0.0));
    let (r1, r2) = (25.0, 24.0);
    assert_eq!(pinch_progress(PINCH_STEPS), None);
    let (a0, b0) = pinch_centres(m, d, 0.0);
    assert!(a0.dist(b0) < 1e-4, "starts as one cell");
    let (a1, b1) = pinch_centres(m, d, 1.0);
    assert!(a1.dist(m) < 1e-4 && b1.dist(d) < 1e-4);
    let mut last_len = 0.0;
    let mut last_neck = f32::INFINITY;
    for k in 1..=20 {
        let (a, b) = pinch_centres(m, d, k as f32 / 20.0);
        let len = pinch_length(a, r1, b, r2);
        let neck = neck_width(a, r1, b, r2);
        assert!(len >= last_len, "the cell keeps elongating");
        assert!(neck <= last_neck + 1e-4, "the furrow only deepens");
        last_len = len;
        last_neck = neck;
    }
    assert_eq!(last_neck, 0.0, "it ends split in two");
    assert!(neck_width(a0, r1, b0, r2) > 2.0 * r2 - 1.0, "no waist at first");
}

#[test]
fn the_cup_reaches_round_closes_and_the_guest_becomes_a_mitochondrion() {
    let guest = Pt::new(30.0, 5.0);
    let (slot, mito_r) = (Pt::new(-10.0, 12.0), 6.0);
    assert!(cup(ENGULF_STEPS, guest, 14.0, slot, mito_r).is_none());
    let first = cup(0.0, guest, 14.0, slot, mito_r).unwrap();
    assert!(!first.closed && first.guest.dist(guest) < 1e-4 && first.guest_radius == 14.0);
    let mut last_span = 0.0;
    let mut closed_seen = false;
    for k in 0..ENGULF_STEPS as u32 {
        let c = cup(k as f32, guest, 14.0, slot, mito_r).unwrap();
        assert!(c.span >= last_span - 1e-5);
        last_span = c.span;
        if c.closed {
            closed_seen = true;
            assert!((c.span - TAU).abs() < 1e-4, "closed means a full ring");
        } else {
            assert!(c.shrunk == 0.0, "the guest shrinks only after the cup closes");
        }
    }
    assert!(closed_seen);
    let end = cup(ENGULF_STEPS - 0.01, guest, 14.0, slot, mito_r).unwrap();
    assert!(end.guest.dist(slot) < 0.2 && (end.guest_radius - mito_r).abs() < 0.1 && end.shrunk > 0.99);
}

#[test]
fn contents_fit_inside_the_membrane() {
    for r in [25.0, 30.0, 40.0, 120.0] {
        let s = bilayer_spec(r);
        assert!(nucleus_radius(r) < s.inner_edge, "nucleus fits");
        for i in 0..3 {
            let (p, len) = mito_slot(i, r);
            assert!(p.len() + len < s.inner_edge + 1.0, "mitochondrion {i} fits at radius {r}");
        }
    }
}
