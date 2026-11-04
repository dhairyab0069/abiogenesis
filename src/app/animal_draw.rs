//! Drawing for the animals screen: the shoreline scene and one animal at a time.
//! Everything here only turns the numbers from `crate::animalviz` into shapes.

use crate::animals::{Frame, Organ, Traits, Zone};
use crate::animalviz::{
    self as viz, BRAIN_AT, Backbone, EYE_AT, Placement, Posture, Pt, Shore, backbone, limb_chain,
    limb_shape, posture, size, stand_height, visibility,
};
use eframe::egui::{self, Align2, Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke, vec2};

const SKY_TOP: Color32 = Color32::from_rgb(28, 44, 78);
const SKY_LOW: Color32 = Color32::from_rgb(120, 156, 186);
const WATER_TOP: Color32 = Color32::from_rgb(44, 120, 150);
const WATER_DEEP: Color32 = Color32::from_rgb(10, 36, 66);
const SAND: Color32 = Color32::from_rgb(170, 150, 108);
const MUD: Color32 = Color32::from_rgb(112, 96, 70);
const GRASS: Color32 = Color32::from_rgb(86, 120, 70);
const SOIL: Color32 = Color32::from_rgb(92, 78, 56);

const GUT: Color32 = Color32::from_rgb(236, 168, 62);
const NERVE: Color32 = Color32::from_rgb(255, 224, 96);
const MUSCLE: Color32 = Color32::from_rgb(196, 72, 72);
const BONE: Color32 = Color32::from_rgb(238, 236, 220);
const GILL: Color32 = Color32::from_rgb(244, 110, 120);
const LUNG: Color32 = Color32::from_rgb(246, 156, 176);
const LIMB: Color32 = Color32::from_rgb(86, 138, 142);

/// Which way an animal is drawn and how it moves.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pose {
    /// +1 facing right (inland), -1 facing left.
    pub facing: f32,
    /// Motion phase in radians (it advances with time).
    pub phase: f32,
    pub posture: Posture,
    /// Pixel height of the ground under it (for walking, propped and stranded animals).
    pub ground_y: Option<f32>,
}

/// Where organs ended up on screen, for labels.
#[derive(Default)]
pub(super) struct Anchors {
    pub points: Vec<(Organ, Pos2)>,
}

fn alpha(c: Color32, a: f32) -> Color32 {
    c.gamma_multiply(a.clamp(0.0, 1.0))
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// A strip of quads between two polylines of equal length (robust for shapes that bend).
fn strip(painter: &egui::Painter, a: &[Pos2], b: &[Pos2], color: Color32) {
    let mut mesh = Mesh::default();
    for k in 0..a.len().min(b.len()) {
        mesh.colored_vertex(a[k], color);
        mesh.colored_vertex(b[k], color);
        if k > 0 {
            let i = (2 * k) as u32;
            mesh.add_triangle(i - 2, i - 1, i);
            mesh.add_triangle(i - 1, i, i + 1);
        }
    }
    painter.add(Shape::mesh(mesh));
}

/// Draws one animal. `centre` is the body's centre (for a swimmer; walkers are placed on `pose.ground_y`),
/// `length` its length in pixels. Returns where each visible organ is, for labels.
pub(super) fn draw_animal(
    painter: &egui::Painter,
    centre: Pos2,
    length: f32,
    t: &Traits,
    pose: Pose,
    time: f32,
) -> Anchors {
    let mut anchors = Anchors::default();
    if !t.bilateral {
        draw_radial(painter, centre, length, t, pose, time, &mut anchors);
        return anchors;
    }
    let l = length;
    let phase = if pose.posture == Posture::Swim {
        pose.phase + time * 4.0
    } else {
        pose.phase + time * 1.1
    };
    let limbs = t.get(Organ::Limbs);
    let shape = limb_shape(t.get(Organ::Fins), limbs);
    let lift = match pose.posture {
        Posture::Stand => stand_height(shape.as_ref(), limbs, t.get(Organ::Skeleton)).max(0.12),
        _ => 0.0,
    };
    let centre = match (pose.posture, pose.ground_y) {
        (Posture::Swim, _) | (_, None) => centre,
        (_, Some(g)) => Pos2::new(centre.x, g - (lift + viz::half_height(t, 0.4) * 0.95) * l),
    };
    // local point to screen
    let at = |p: Pt| Pos2::new(centre.x + p.x * l * pose.facing, centre.y + p.y * l);
    let body_phase = if pose.posture == Posture::Swim {
        phase
    } else {
        0.0
    };
    let walking = matches!(pose.posture, Posture::Stand | Posture::Prop);

    // paired fins or limbs: far side first
    let paired = |painter: &egui::Painter, far: bool| -> Vec<(Organ, Pos2)> {
        let mut found = Vec::new();
        let Some(shape) = shape else { return found };
        let shade = if far { 0.55 } else { 1.0 };
        for (s, scale, offset) in [(0.68, 1.0, 0.0), (0.28, 0.85, std::f32::consts::PI)] {
            // hind appendages are drawn as limbs only once the shape is limb-like or the fins are big
            if s < 0.5 && shape.blend < 0.2 && t.get(Organ::Fins) < 0.5 {
                continue;
            }
            let vis = visibility(t.get(Organ::Fins).max(limbs));
            let mut sh = shape;
            sh.length *= scale;
            let side = if far { std::f32::consts::PI * 0.5 } else { 0.0 };
            let attach = viz::body_point(t, s, 0.95, body_phase);
            let chain = limb_chain(
                &sh,
                attach,
                phase * if walking { 0.7 } else { 1.0 } + offset + side,
                walking,
            );
            let pts: Vec<Pos2> = chain.iter().map(|p| at(*p)).collect();
            let color = alpha(mix(LIMB, Color32::from_rgb(70, 110, 120), 0.4 * shade), vis);
            if sh.web > 0.05 {
                // the webbing: a flap behind the bones
                let fringe = at(Pt::new(
                    attach.x - sh.length * 0.9,
                    attach.y + sh.length * 0.15,
                ));
                painter.add(Shape::convex_polygon(
                    vec![pts[0], pts[2], fringe],
                    alpha(color, 0.55 * sh.web),
                    Stroke::NONE,
                ));
            }
            let w = (2.0 + 4.0 * sh.blend) * (l / 70.0).max(0.5);
            painter.add(Shape::line(
                vec![pts[0], pts[1], pts[2]],
                Stroke::new(w, color),
            ));
            if sh.toes {
                let dir = (pts[2] - pts[1]).normalized();
                for k in -1..=1 {
                    let d = egui::Vec2::angled(dir.angle() + k as f32 * 0.5) * (0.025 * l);
                    painter.line_segment([pts[2], pts[2] + d], Stroke::new(w * 0.6, color));
                }
            }
            if !far && s > 0.5 {
                found.push((Organ::Limbs, pts[2]));
                found.push((Organ::Fins, pts[1]));
            }
        }
        found
    };
    // far-side limbs are only worth drawing for walkers
    if walking {
        paired(painter, true);
    }
    // tail and back fins
    let fin_vis = visibility(t.get(Organ::Fins));
    if fin_vis > 0.0 {
        let tf = viz::tail_fin(t, body_phase);
        let pts: Vec<Pos2> = tf.iter().map(|p| at(*p)).collect();
        painter.add(Shape::convex_polygon(
            pts.clone(),
            alpha(LIMB, 0.7 * fin_vis),
            Stroke::new(1.0, alpha(LIMB, fin_vis)),
        ));
        let df = viz::dorsal_fin(t, body_phase);
        painter.add(Shape::convex_polygon(
            df.iter().map(|p| at(*p)).collect(),
            alpha(LIMB, 0.6 * fin_vis),
            Stroke::new(1.0, alpha(LIMB, fin_vis)),
        ));
        anchors.points.push((Organ::Fins, pts[1]));
    }
    // body
    let skin = t.get(Organ::Skin);
    let base = mix(
        Color32::from_rgb(92, 150, 170),
        Color32::from_rgb(158, 142, 98),
        skin * 0.8,
    );
    let n = 22;
    let back: Vec<Pos2> = (0..=n)
        .map(|k| at(viz::body_point(t, k as f32 / n as f32, -1.0, body_phase)))
        .collect();
    let belly: Vec<Pos2> = (0..=n)
        .map(|k| at(viz::body_point(t, k as f32 / n as f32, 1.0, body_phase)))
        .collect();
    strip(painter, &back, &belly, alpha(base, 0.92));
    let edge = mix(base, Color32::BLACK, 0.45);
    let edge_w = 1.0 + 2.2 * skin;
    painter.add(Shape::line(back.clone(), Stroke::new(edge_w, edge)));
    painter.add(Shape::line(belly.clone(), Stroke::new(edge_w, edge)));
    if skin >= 0.3 {
        // a scaly, sealed skin: short ticks along the back
        for k in (2..n - 1).step_by(2) {
            let a = back[k];
            painter.line_segment(
                [a, a + vec2(0.0, 0.03 * l)],
                Stroke::new(1.0, alpha(edge, skin)),
            );
        }
        anchors.points.push((Organ::Skin, back[n / 3]));
    }

    // organs, inside the body
    let mv = visibility(t.get(Organ::Muscle));
    if mv > 0.0 {
        for blk in viz::muscle_blocks(t, body_phase) {
            painter.add(Shape::line(
                blk.iter().map(|p| at(*p)).collect(),
                Stroke::new(1.6, alpha(MUSCLE, 0.55 * mv)),
            ));
        }
        anchors
            .points
            .push((Organ::Muscle, at(viz::body_point(t, 0.4, 0.45, body_phase))));
    }
    let sv = visibility(t.get(Organ::Skeleton));
    if sv > 0.0 {
        let (line, ticks) = viz::backbone_line(t, body_phase);
        let bone = backbone(t.get(Organ::Skeleton)) == Backbone::Bone;
        let screen: Vec<Pos2> = line.iter().map(|p| at(*p)).collect();
        painter.add(Shape::line(
            screen.clone(),
            Stroke::new(
                if bone { 2.6 } else { 1.8 },
                alpha(BONE, sv * if bone { 1.0 } else { 0.75 }),
            ),
        ));
        for p in ticks {
            let a = at(p);
            painter.line_segment(
                [a - vec2(0.0, 0.024 * l), a + vec2(0.0, 0.024 * l)],
                Stroke::new(1.6, alpha(BONE, sv)),
            );
        }
        if bone {
            // a skull
            let head = at(Pt::new(0.4, -0.2));
            painter.circle_filled(head, 0.05 * l, alpha(BONE, 0.9 * sv));
        }
        anchors.points.push((Organ::Skeleton, screen[5]));
    }
    let lv = visibility(t.get(Organ::Lungs));
    if lv > 0.0 {
        let (c, r) = viz::lung(t, body_phase);
        painter.add(Shape::ellipse_filled(
            at(c),
            vec2(r.x * l, r.y * l),
            alpha(LUNG, 0.85 * lv),
        ));
        anchors.points.push((Organ::Lungs, at(c)));
    }
    let gv = visibility(t.get(Organ::Gut));
    if gv > 0.0 {
        let line: Vec<Pos2> = viz::gut_line(t, body_phase)
            .iter()
            .map(|p| at(*p))
            .collect();
        painter.add(Shape::line(
            line.clone(),
            Stroke::new(3.0 * (l / 70.0).max(0.6), alpha(GUT, gv)),
        ));
        painter.circle_filled(line[0], 0.012 * l + 1.5, alpha(Color32::BLACK, 0.8 * gv));
        if line.len() > 5 {
            anchors.points.push((Organ::Gut, line[5]));
        }
    }
    let wv = visibility(t.get(Organ::Gills));
    if wv > 0.0 {
        let slits = viz::gill_slits(t, body_phase);
        for s in &slits {
            painter.line_segment([at(s[0]), at(s[1])], Stroke::new(1.8, alpha(GILL, wv)));
        }
        anchors.points.push((Organ::Gills, at(slits[0][1])));
    }
    let nv = visibility(t.get(Organ::Nerve));
    if nv > 0.0 {
        let line: Vec<Pos2> = viz::nerve_line(t, body_phase)
            .iter()
            .map(|p| at(*p))
            .collect();
        painter.add(Shape::line(
            line.clone(),
            Stroke::new(1.5, alpha(NERVE, nv)),
        ));
        if t.get(Organ::Nerve) >= BRAIN_AT {
            let (c, r) = viz::brain(t, body_phase);
            painter.circle_filled(at(c), r * l, alpha(NERVE, nv));
        }
        anchors.points.push((Organ::Nerve, line[4]));
    }
    let ev = visibility(t.get(Organ::Eyes));
    if ev > 0.0 {
        let (c, r) = viz::eye(t, body_phase);
        let c = at(c);
        let r = (r * l).max(1.5);
        if t.get(Organ::Eyes) >= EYE_AT {
            painter.circle_filled(c, r, alpha(Color32::WHITE, ev));
            painter.circle_filled(
                c + vec2(0.25 * r * pose.facing, 0.0),
                r * 0.55,
                alpha(Color32::BLACK, ev),
            );
        } else {
            // a light-sensitive patch
            painter.circle_filled(c, r, alpha(Color32::from_rgb(240, 236, 180), 0.8 * ev));
        }
        anchors.points.push((Organ::Eyes, c));
    }
    // jaws or a plain mouth
    let bite = 0.5 + 0.5 * (time * 2.2 + pose.phase).sin();
    let jaws = viz::jaws(t, bite, body_phase);
    let (up, low): (Vec<Pos2>, Vec<Pos2>) = (
        jaws.upper.iter().map(|p| at(*p)).collect(),
        jaws.lower.iter().map(|p| at(*p)).collect(),
    );
    let jv = visibility(t.get(Organ::Jaws));
    if jv > 0.0 && t.get(Organ::Jaws) >= viz::JAWS_OPEN_AT {
        let jaw = alpha(mix(base, BONE, 0.5), 0.95);
        painter.add(Shape::convex_polygon(
            up.clone(),
            jaw,
            Stroke::new(1.0, edge),
        ));
        painter.add(Shape::convex_polygon(
            low.clone(),
            jaw,
            Stroke::new(1.0, edge),
        ));
        if jaws.teeth {
            for k in 0..3 {
                let f = 0.3 + 0.25 * k as f32;
                let a = up[1].lerp(up[2], f);
                painter.add(Shape::convex_polygon(
                    vec![
                        a,
                        a + vec2(1.6 * pose.facing, 0.0),
                        a + vec2(0.8 * pose.facing, 0.03 * l),
                    ],
                    Color32::WHITE,
                    Stroke::NONE,
                ));
            }
        }
        anchors.points.push((Organ::Jaws, low[2]));
    } else {
        painter.line_segment([up[0], up[1]], Stroke::new(1.2, alpha(edge, 0.8)));
    }
    anchors.points.extend(paired(painter, false));
    anchors
}

fn draw_radial(
    painter: &egui::Painter,
    centre: Pos2,
    length: f32,
    t: &Traits,
    pose: Pose,
    time: f32,
    anchors: &mut Anchors,
) {
    let l = length;
    let phase = pose.phase + time * 1.6;
    let centre = match (pose.posture, pose.ground_y) {
        (Posture::Swim, _) | (_, None) => centre,
        (_, Some(g)) => Pos2::new(centre.x, g - 0.3 * l),
    };
    let r = viz::radial_radius(t, phase) * l;
    let skin = t.get(Organ::Skin);
    let base = mix(
        Color32::from_rgb(150, 120, 190),
        Color32::from_rgb(170, 140, 110),
        skin * 0.8,
    );
    for k in 0..viz::ARMS {
        let arm: Vec<Pos2> = viz::arm(t, k, phase, 6)
            .iter()
            .map(|p| centre + vec2(p.x * l, p.y * l))
            .collect();
        painter.add(Shape::line(arm.clone(), Stroke::new(2.2, alpha(base, 0.8))));
        painter.circle_filled(*arm.last().unwrap(), 1.8, alpha(base, 0.9));
    }
    painter.circle_filled(centre, r, alpha(base, 0.9));
    painter.circle_stroke(
        centre,
        r,
        Stroke::new(1.0 + 2.2 * skin, mix(base, Color32::BLACK, 0.45)),
    );
    // a blind gut: a pouch with one opening
    let gv = visibility(t.get(Organ::Gut));
    let pouch = r * (0.35 + 0.8 * t.get(Organ::Gut));
    painter.circle_filled(centre, pouch, alpha(GUT, 0.8 * gv.max(0.4)));
    painter.circle_filled(
        centre,
        pouch * 0.35,
        alpha(Color32::BLACK, 0.8 * gv.max(0.4)),
    );
    anchors
        .points
        .push((Organ::Gut, centre + vec2(pouch * 0.6, 0.0)));
    let nv = visibility(t.get(Organ::Nerve));
    if nv > 0.0 {
        // a nerve net: a ring with spokes
        painter.circle_stroke(centre, r * 0.78, Stroke::new(1.2, alpha(NERVE, nv)));
        for k in 0..viz::ARMS {
            let d = egui::Vec2::angled(k as f32 / viz::ARMS as f32 * std::f32::consts::TAU);
            painter.line_segment(
                [centre + d * pouch, centre + d * r * 0.78],
                Stroke::new(1.0, alpha(NERVE, nv * 0.8)),
            );
        }
        anchors
            .points
            .push((Organ::Nerve, centre + vec2(0.0, -r * 0.78)));
    }
    let ev = visibility(t.get(Organ::Eyes));
    if ev > 0.0 {
        for k in 0..3 {
            let d = egui::Vec2::angled(k as f32 * 2.09 + 0.6) * r * 0.9;
            painter.circle_filled(
                centre + d,
                1.8 + 2.0 * t.get(Organ::Eyes),
                alpha(Color32::from_rgb(240, 236, 180), ev),
            );
        }
        anchors
            .points
            .push((Organ::Eyes, centre + egui::Vec2::angled(0.6) * r * 0.9));
    }
    let sv = visibility(t.get(Organ::Skeleton));
    if sv > 0.0 {
        painter.line_segment(
            [centre - vec2(r * 0.5, 0.0), centre + vec2(r * 0.5, 0.0)],
            Stroke::new(1.5, alpha(BONE, sv * 0.7)),
        );
        anchors
            .points
            .push((Organ::Skeleton, centre - vec2(r * 0.4, 0.0)));
    }
    let wv = visibility(t.get(Organ::Gills));
    if wv > 0.0 {
        for k in 0..3 {
            let d = egui::Vec2::angled(1.0 + k as f32 * 2.1) * r * 0.8;
            painter.circle_stroke(
                centre + d,
                2.0 + 2.0 * size(t.get(Organ::Gills)),
                Stroke::new(1.3, alpha(GILL, wv)),
            );
        }
        anchors
            .points
            .push((Organ::Gills, centre + egui::Vec2::angled(1.0) * r * 0.8));
    }
}

/// The big picture of the lineage's leading animal, with a label on every organ that has appeared.
pub(super) fn draw_portrait(
    painter: &egui::Painter,
    rect: Rect,
    t: &Traits,
    zone: Zone,
    time: f32,
) {
    painter.rect_filled(
        rect,
        6.0,
        if zone.is_water() {
            Color32::from_rgb(14, 40, 66)
        } else {
            Color32::from_rgb(52, 76, 92)
        },
    );
    let p = posture(t, zone);
    let length = (rect.width() * 0.5).min(rect.height() * 1.1);
    let ground = (!zone.is_water()).then_some(rect.bottom() - 26.0);
    if let Some(g) = ground {
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(rect.left(), g), rect.right_bottom()),
            0.0,
            SOIL,
        );
    }
    let centre = Pos2::new(rect.center().x, rect.center().y + 4.0);
    let pose = Pose {
        facing: 1.0,
        phase: 0.0,
        posture: if zone.is_water() { Posture::Swim } else { p },
        ground_y: ground,
    };
    let anchors = draw_animal(painter, centre, length, t, pose, time);
    // labels: organs sitting high go along the top, the rest along the bottom
    let mut seen: Vec<Organ> = Vec::new();
    let mut top: Vec<(Organ, Pos2)> = Vec::new();
    let mut bottom: Vec<(Organ, Pos2)> = Vec::new();
    for (o, pos) in anchors.points {
        if seen.contains(&o) {
            continue;
        }
        seen.push(o);
        if pos.y < centre.y {
            top.push((o, pos))
        } else {
            bottom.push((o, pos))
        }
    }
    for (row, y_edge, align) in [
        (top, rect.top() + 4.0, Align2::CENTER_TOP),
        (bottom, rect.bottom() - 4.0, Align2::CENTER_BOTTOM),
    ] {
        let mut row = row;
        row.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
        let n = row.len().max(1) as f32;
        for (i, (o, pos)) in row.into_iter().enumerate() {
            let x = rect.left() + rect.width() * (i as f32 + 0.5) / n;
            let stagger = if i % 2 == 0 { 0.0 } else { 12.0 };
            let y = if align == Align2::CENTER_TOP {
                y_edge + stagger
            } else {
                y_edge - stagger
            };
            let end = Pos2::new(
                x,
                if align == Align2::CENTER_TOP {
                    y + 11.0
                } else {
                    y - 11.0
                },
            );
            painter.line_segment([pos, end], Stroke::new(0.7, Color32::from_white_alpha(90)));
            painter.text(
                Pos2::new(x, y),
                align,
                o.label(),
                FontId::proportional(10.5),
                Color32::from_white_alpha(225),
            );
        }
    }
}

/// The shoreline: sky, tide, the ground, zone labels and every sampled animal. Returns animal boxes
/// (for hover text), each with the animal's index in `frame.samples`.
pub(super) fn draw_scene(
    painter: &egui::Painter,
    rect: Rect,
    shore: &Shore,
    frame: &Frame,
    time: f32,
) -> Vec<(Rect, usize)> {
    let px = |x: f32, y: f32| {
        Pos2::new(
            rect.left() + x * rect.width(),
            rect.top() + y * rect.height(),
        )
    };
    // sky
    let split = shore.water_level(time);
    let bands = 12;
    for k in 0..bands {
        let (a, b) = (
            k as f32 / bands as f32 * split,
            (k + 1) as f32 / bands as f32 * split,
        );
        painter.rect_filled(
            Rect::from_min_max(px(0.0, a), px(1.0, b + 0.002)),
            0.0,
            mix(SKY_TOP, SKY_LOW, k as f32 / bands as f32),
        );
    }
    painter.circle_filled(
        px(0.9, 0.12),
        rect.height() * 0.05,
        Color32::from_rgb(255, 232, 160),
    );
    // water, in depth bands
    let depth_bands = 14;
    for k in 0..depth_bands {
        let (a, b) = (
            split + (1.0 - split) * k as f32 / depth_bands as f32,
            split + (1.0 - split) * (k + 1) as f32 / depth_bands as f32,
        );
        painter.rect_filled(
            Rect::from_min_max(px(0.0, a), px(1.0, b + 0.002)),
            0.0,
            mix(WATER_TOP, WATER_DEEP, k as f32 / depth_bands as f32),
        );
    }
    // the wavy surface
    let waves: Vec<Pos2> = (0..=80)
        .map(|k| {
            let x = k as f32 / 80.0;
            px(x, split + 0.004 * (x * 38.0 + time * 1.6).sin())
        })
        .collect();
    painter.add(Shape::line(
        waves,
        Stroke::new(1.5, Color32::from_white_alpha(120)),
    ));
    // the ground
    let steps = 90;
    for k in 0..steps {
        let (x0, x1) = (k as f32 / steps as f32, (k + 1) as f32 / steps as f32);
        let zone = shore.zone_at((x0 + x1) / 2.0);
        let colour = match zone {
            Zone::OpenSea | Zone::Shallows => SAND,
            Zone::TidalFlats => mix(MUD, SAND, 0.2),
            Zone::Land => SOIL,
        };
        let mut mesh = Mesh::default();
        let tl = px(x0, shore.ground(x0));
        let tr = px(x1, shore.ground(x1));
        let (bl, br) = (
            Pos2::new(tl.x, rect.bottom()),
            Pos2::new(tr.x, rect.bottom()),
        );
        for (p, c) in [
            (tl, colour),
            (tr, colour),
            (br, mix(colour, Color32::BLACK, 0.35)),
            (bl, mix(colour, Color32::BLACK, 0.35)),
        ] {
            mesh.colored_vertex(p, c);
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(Shape::mesh(mesh));
        if zone == Zone::Land {
            painter.line_segment([tl, tr], Stroke::new(3.0, GRASS));
        }
        if zone == Zone::TidalFlats && !shore.flooded((x0 + x1) / 2.0, time) {
            painter.line_segment([tl, tr], Stroke::new(2.0, mix(MUD, Color32::BLACK, 0.2)));
        }
    }
    // zone bands and counts
    let label_y = rect.bottom() - 4.0;
    for z in Zone::ALL {
        let (a, b) = shore.span(z);
        if z != Zone::OpenSea {
            painter.line_segment(
                [px(a, 0.0), px(a, 1.0)],
                Stroke::new(0.7, Color32::from_white_alpha(36)),
            );
        }
        let x = (a + b) / 2.0;
        painter.text(
            Pos2::new(rect.left() + x * rect.width(), label_y),
            Align2::CENTER_BOTTOM,
            format!("{}  ({})", z.label(), frame.counts[z.index()]),
            FontId::proportional(11.5),
            Color32::from_white_alpha(215),
        );
    }
    // animals
    let mut boxes = Vec::new();
    let length = (rect.width() * 0.075).clamp(34.0, 78.0);
    for zone in Zone::ALL {
        let members: Vec<(usize, &crate::animals::Sample)> = frame
            .samples
            .iter()
            .enumerate()
            .filter(|(_, s)| s.zone == zone)
            .collect();
        let n = members.len();
        for (k, (index, sample)) in members.into_iter().enumerate() {
            let post = posture(&sample.traits, zone);
            let Placement { at, facing, phase } = shore.place(zone, k, n, time, post);
            let size = if sample.traits.bilateral {
                length
            } else {
                length * 0.8
            };
            let centre = px(at.x, at.y);
            let ground = px(at.x, shore.ground(at.x)).y;
            let pose = Pose {
                facing,
                phase,
                posture: post,
                ground_y: Some(ground),
            };
            draw_animal(painter, centre, size, &sample.traits, pose, time);
            boxes.push((
                Rect::from_center_size(
                    Pos2::new(
                        centre.x,
                        if post == Posture::Swim {
                            centre.y
                        } else {
                            ground - size * 0.25
                        },
                    ),
                    vec2(size * 1.1, size * 0.7),
                ),
                index,
            ));
        }
    }
    boxes
}
