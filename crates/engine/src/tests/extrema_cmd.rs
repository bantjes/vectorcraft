//! Object › Path › Nodes at Extrema on a document path.

use serde_json::json;

use crate::*;

#[test]
fn nodes_at_extrema_puts_an_ellipse_rotated_45_degrees_on_its_extremes() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 300})).unwrap();
    s.execute("shape.ellipse", &json!({"x": 100, "y": 100, "width": 100, "height": 60})).unwrap();
    s.execute("object.rotate", &json!({"angle": 30})).unwrap();
    let before = s.doc().unwrap().doc.clone();
    let r = s.execute("object.path.nodesAtExtrema", &json!({})).unwrap();
    let id = NodeId(r["ids"][0].as_u64().unwrap());
    let doc = &s.doc().unwrap().doc;
    let n = doc.node(id).unwrap();
    let b0 = before.node(id).unwrap().geometric_bounds().unwrap();
    let b1 = n.geometric_bounds().unwrap();
    assert!((b0.x0 - b1.x0).abs() < 0.05 && (b0.y1 - b1.y1).abs() < 0.05, "same shape: {b0:?} {b1:?}");
    let path = n.path_data().unwrap();
    for x in [b1.x0, b1.x1] {
        assert!(path.subpaths[0].anchors.iter().any(|a| (a.p.x - x).abs() < 0.01), "an anchor on x = {x}: {path:?}");
    }
    for y in [b1.y0, b1.y1] {
        assert!(path.subpaths[0].anchors.iter().any(|a| (a.p.y - y).abs() < 0.01), "an anchor on y = {y}: {path:?}");
    }
}
