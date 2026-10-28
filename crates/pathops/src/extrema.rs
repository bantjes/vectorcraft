//! Object → Path → Nodes at Extrema (FontLab's "nodes at extremes"): an anchor at every point
//! where a curve is furthest left, right, up or down, then the old in-between smooth anchors
//! taken out where two curves join into one without the shape moving.

use kurbo::{CubicBez, ParamCurve, ParamCurveExtrema, ParamCurveNearest, Point};
use vectorcraft_geom::{AnchorKind, PathData, SubPath};

use crate::fit::{end_tangent, fit_single, sample, start_tangent};

/// How far (document points) the shape may move when an old anchor is taken out: far below
/// anything visible at any zoom a drawing is made at.
pub const EXTREMA_TOLERANCE: f64 = 0.02;

/// Anchors closer than this to a segment's end count as being on it (no anchor is added there).
const NEAR_END: f64 = 1e-3;

/// `path` with an anchor at every horizontal and vertical extreme of its curves, and the smooth
/// anchors that aren't at one removed where the shape stays within [`EXTREMA_TOLERANCE`].
pub fn nodes_at_extrema(path: &PathData) -> PathData {
    let mut out = path.clone();
    for sp in &mut out.subpaths {
        add_extrema(sp);
        remove_between(sp);
    }
    out
}

/// The parameters (strictly inside, ascending) where `c` is at a horizontal or vertical extreme.
fn extrema_of(c: &CubicBez) -> Vec<f64> {
    let mut ts: Vec<f64> = c.extrema().into_iter().filter(|t| *t > 0.0 && *t < 1.0).collect();
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    ts.retain(|t| c.eval(*t).distance(c.p0) > NEAR_END && c.eval(*t).distance(c.p3) > NEAR_END);
    ts
}

fn add_extrema(sp: &mut SubPath) {
    for seg in (0..sp.segment_count()).rev() {
        if sp.segment_is_line(seg) {
            continue;
        }
        // Latest first: splitting at t keeps [0, t] as `seg`, so an earlier u lies at u / t of it.
        let mut end = 1.0;
        for t in extrema_of(&sp.segment(seg)).into_iter().rev() {
            sp.insert_anchor(seg, t / end);
            end = t;
        }
    }
}

/// True when anchor `i`'s curve runs level or upright through it (an extreme, to be kept).
fn at_extreme(sp: &SubPath, i: usize) -> bool {
    let a = &sp.anchors[i];
    let d = if a.has_out() { a.h_out - a.p } else { a.p - a.h_in };
    let len = d.hypot();
    len > 0.0 && (d.x.abs() / len < 1e-6 || d.y.abs() / len < 1e-6)
}

/// One cubic standing for `a` then `b` (which meet smoothly at `a.p3`), its handles along theirs,
/// when it stays within [`EXTREMA_TOLERANCE`] of both.
fn merged(a: &CubicBez, b: &CubicBez) -> Option<CubicBez> {
    let mut pts = vec![];
    sample(a, 24, &mut pts, true);
    sample(b, 24, &mut pts, false);
    let (c, _, _) = fit_single(&pts, start_tangent(a), end_tangent(b));
    let close = |from: &CubicBez, to: &[&CubicBez]| {
        (0..=24).all(|k| {
            let p: Point = from.eval(k as f64 / 24.0);
            to.iter().map(|s| s.nearest(p, 1e-6).distance_sq.sqrt()).fold(f64::INFINITY, f64::min) <= EXTREMA_TOLERANCE
        })
    };
    (close(&c, &[a, b]) && close(a, &[&c]) && close(b, &[&c])).then_some(c)
}

fn remove_between(sp: &mut SubPath) {
    while let Some(i) = (0..sp.anchors.len()).find(|&i| removable(sp, i).is_some()) {
        let c = removable(sp, i).expect("found removable");
        let n = sp.anchors.len();
        let (prev, next) = ((i + n - 1) % n, (i + 1) % n);
        sp.anchors[prev].h_out = c.p1;
        sp.anchors[next].h_in = c.p2;
        sp.anchors.remove(i);
    }
}

/// The one curve that replaces the two either side of anchor `i`, when `i` can go: a smooth
/// anchor inside the path (not an open path's end; a closed path keeps three), not at an extreme.
fn removable(sp: &SubPath, i: usize) -> Option<CubicBez> {
    let n = sp.anchors.len();
    let a = &sp.anchors[i];
    let inner = if sp.closed { n > 3 } else { i > 0 && i + 1 < n };
    if !inner || a.kind != AnchorKind::Smooth || !a.has_in() || !a.has_out() || at_extreme(sp, i) {
        return None;
    }
    merged(&sp.segment((i + n - 1) % n), &sp.segment(i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_geom::Anchor;

    fn smooth(p: (f64, f64), h_in: (f64, f64), h_out: (f64, f64)) -> Anchor {
        Anchor { p: p.into(), h_in: h_in.into(), h_out: h_out.into(), kind: AnchorKind::Smooth }
    }

    /// Largest distance from points of `a` to `b`'s outline, both ways.
    fn apart(a: &PathData, b: &PathData) -> f64 {
        let segs = |p: &PathData| -> Vec<CubicBez> { p.subpaths.iter().flat_map(|s| (0..s.segment_count()).map(|i| s.segment(i)).collect::<Vec<_>>()).collect() };
        let (sa, sb) = (segs(a), segs(b));
        let one = |x: &[CubicBez], y: &[CubicBez]| {
            x.iter()
                .flat_map(|c| (0..=32).map(move |k| c.eval(k as f64 / 32.0)))
                .map(|p| y.iter().map(|s| s.nearest(p, 1e-9).distance_sq.sqrt()).fold(f64::INFINITY, f64::min))
                .fold(0.0, f64::max)
        };
        one(&sa, &sb).max(one(&sb, &sa))
    }

    /// A circle-like shape drawn with its anchors at 45° (none at the extremes).
    fn tilted_circle() -> PathData {
        let k = 0.5523 * 50.0;
        let r = 50.0 / 2f64.sqrt();
        let pts = [(r, r), (-r, r), (-r, -r), (r, -r)];
        let anchors = pts
            .iter()
            .map(|&(x, y)| {
                // Tangent at angle θ runs along (-sin θ, cos θ).
                let (tx, ty) = (-y / 50.0, x / 50.0);
                smooth((x, y), (x - tx * k, y - ty * k), (x + tx * k, y + ty * k))
            })
            .collect();
        PathData::single(SubPath::new(anchors, true))
    }

    #[test]
    fn a_tilted_circle_gets_its_four_extremes_and_keeps_its_shape() {
        let before = tilted_circle();
        let after = nodes_at_extrema(&before);
        let sp = &after.subpaths[0];
        for (x, y) in [(50.0, 0.0), (0.0, 50.0), (-50.0, 0.0), (0.0, -50.0)] {
            assert!(sp.anchors.iter().any(|a| a.p.distance(Point::new(x, y)) < 0.5), "an anchor near ({x}, {y}): {:?}", sp.anchors);
        }
        assert!(apart(&before, &after) < EXTREMA_TOLERANCE * 2.0, "{}", apart(&before, &after));
        assert_eq!(sp.anchors.len(), 4, "the 45° anchors are gone: {:?}", sp.anchors);
    }

    #[test]
    fn a_shape_already_at_its_extremes_is_left_alone() {
        let k = 0.5523 * 50.0;
        let circle = PathData::single(SubPath::new(
            vec![
                smooth((50.0, 0.0), (50.0, -k), (50.0, k)),
                smooth((0.0, 50.0), (k, 50.0), (-k, 50.0)),
                smooth((-50.0, 0.0), (-50.0, k), (-50.0, -k)),
                smooth((0.0, -50.0), (-k, -50.0), (k, -50.0)),
            ],
            true,
        ));
        assert_eq!(nodes_at_extrema(&circle), circle);
    }

    #[test]
    fn straight_lines_and_corners_stay() {
        let square = PathData::single(SubPath::polyline(&[(0.0, 0.0).into(), (10.0, 0.0).into(), (10.0, 10.0).into(), (0.0, 10.0).into()], true));
        assert_eq!(nodes_at_extrema(&square), square);
    }

    #[test]
    fn an_s_curve_keeps_its_middle_anchor_when_taking_it_out_would_move_the_shape() {
        // Two arches joined smoothly at a slant: one cubic can't follow both.
        let s = PathData::single(SubPath::new(
            vec![
                Anchor { p: (0.0, 0.0).into(), h_in: (0.0, 0.0).into(), h_out: (0.0, 40.0).into(), kind: AnchorKind::Corner },
                smooth((20.0, 20.0), (10.0, 10.0), (30.0, 30.0)),
                Anchor { p: (40.0, 40.0).into(), h_in: (40.0, 0.0).into(), h_out: (40.0, 40.0).into(), kind: AnchorKind::Corner },
            ],
            false,
        ));
        let after = nodes_at_extrema(&s);
        assert!(apart(&s, &after) < EXTREMA_TOLERANCE * 2.0, "{}", apart(&s, &after));
        assert!(after.subpaths[0].anchors.iter().any(|a| a.p.distance(Point::new(20.0, 20.0)) < 1e-9), "{:?}", after.subpaths[0].anchors);
    }
}
