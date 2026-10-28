use serde_json::json;

use super::*;
use crate::menus::menu_tree;

fn app() -> VectorcraftApp {
    use_starting_sort();
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
    app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    app
}

/// The labels of a menu's items, as they show.
fn labels(items: &[Item]) -> Vec<&'static str> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Cmd(l, ..) | Item::Todo(l, _) | Item::Sub(l, _) | Item::Header(l) => Some(*l),
            Item::Sep => None,
        })
        .collect()
}

fn menu_items(app: &VectorcraftApp, title: &str) -> Vec<Item> {
    prune_tree(&app.ui, menu_tree()).into_iter().find(|(t, _)| *t == title).unwrap().1
}

fn toolbar(app: &VectorcraftApp) -> Vec<&'static str> {
    crate::toolbar::slots(app).into_iter().flat_map(|(_, s)| s).collect()
}

#[test]
fn every_key_the_build_starts_with_names_something_that_can_be_put_away() {
    let keys: Vec<String> = things().into_iter().map(|t| t.key).collect();
    for (key, _) in SHIPPED {
        assert!(keys.iter().any(|k| k == key), "{key}");
    }
}

#[test]
fn the_starting_sort_puts_tools_away_and_moves_pen_and_measure() {
    let app = app();
    let shown = toolbar(&app);
    for gone in ["pencil", "arc", "eraser", "knife", "paintbrush", "curvature", "zoom", "printTiling", "warp", "shear", "mesh", "polygon", "star", "flare"] {
        assert!(!shown.contains(&gone), "{gone} shows");
    }
    for kept in ["selection", "pen", "rectangle", "shaper", "scissors", "join", "rotate", "scale", "reflect", "blend", "type", "measure", "hand"] {
        assert!(shown.contains(&kept), "{kept} is missing");
    }
    let slots = crate::toolbar::slots(&app);
    // Pen right under the selection tools; Measure, with Dimension, among the navigation tools.
    assert_eq!(slots[2].1[0], "pen");
    let navigate = slots.iter().position(|(c, _)| *c == Some("Navigate")).unwrap();
    assert!(slots[navigate..].iter().take_while(|(c, _)| c.is_none() || *c == Some("Navigate")).any(|(_, s)| s.contains(&"measure") && s.contains(&"dimension")));
    // Pencil's category heading moves to the next slot that shows.
    assert!(slots.iter().any(|(c, s)| *c == Some("Draw") && s[0] == "scissors"));
}

#[test]
fn a_slot_never_shows_a_tool_put_away_since_it_was_last_used() {
    let mut app = app();
    app.ui.slot_tool.insert("gradient".into(), "mesh".into());
    let slots = crate::toolbar::slots(&app);
    let (_, color) = slots.iter().find(|(_, s)| s.contains(&"gradient")).unwrap();
    assert_eq!(crate::toolbar::remembered(&app, color), "gradient");
    app.ui.slot_tool.insert("gradient".into(), "gradient".into());
    assert_eq!(crate::toolbar::remembered(&app, color), "gradient");
}

#[test]
fn a_tool_put_away_comes_back_where_it_was() {
    let mut app = app();
    app.run("closet.set", json!({"key": "tool:pencil", "shelf": "inUse"})).unwrap();
    assert!(toolbar(&app).contains(&"pencil"));
    app.run("tool.select", json!({"tool": "pencil"})).unwrap();
    app.run("closet.set", json!({"key": "tool:pencil", "shelf": "basement"})).unwrap();
    assert!(!toolbar(&app).contains(&"pencil"));
    assert_eq!(app.session.tool_id(), "selection", "the active tool put away gives way");
    assert_eq!(shelf(&app.ui, "tool:pencil"), Shelf::Basement);
    // Back to where the build starts it: nothing is remembered.
    app.run("closet.set", json!({"key": "tool:pencil", "shelf": "closet"})).unwrap();
    assert!(app.ui.closet.moved.is_empty());
}

#[test]
fn the_raster_effects_leave_the_effect_menu_without_their_heading() {
    let mut app = app();
    let effect = labels(&menu_items(&app, "Effect"));
    for gone in ["Raster Effects", "Effect Gallery…", "Artistic", "Blur", "Sketch", "Texture"] {
        assert!(!effect.contains(&gone), "{gone}: {effect:?}");
    }
    assert_eq!(effect.iter().filter(|l| **l == "Stylize").count(), 1, "the vector Stylize stays");
    assert!(effect.contains(&"Distort & Transform"));
    app.run("closet.set", json!({"key": "menu:Effect › Blur", "shelf": "inUse"})).unwrap();
    let effect = labels(&menu_items(&app, "Effect"));
    assert!(effect.contains(&"Raster Effects") && effect.contains(&"Blur"), "{effect:?}");
}

#[test]
fn a_panel_put_away_leaves_the_window_menu_and_can_not_be_opened() {
    let mut app = app();
    assert!(!labels(&menu_items(&app, "Window")).contains(&"Brushes"));
    assert!(labels(&menu_items(&app, "Window")).contains(&"Closet"));
    let e = app.run("window.panel", json!({"panel": "brushes"})).unwrap_err();
    assert!(e.contains("Closet"), "{e}");
    app.run("closet.set", json!({"key": "panel:brushes", "shelf": "inUse"})).unwrap();
    assert!(labels(&menu_items(&app, "Window")).contains(&"Brushes"));
    app.run("window.panel", json!({"panel": "brushes"})).unwrap();
    // Putting it away again closes it.
    app.run("closet.set", json!({"key": "panel:layers", "shelf": "closet"})).unwrap();
    assert!(panel_hidden(&app.ui, "layers"));
}

#[test]
fn the_closet_itself_and_unknown_things_can_not_be_put_away() {
    let mut app = app();
    assert!(app.run("closet.set", json!({"key": "panel:closet", "shelf": "basement"})).is_err());
    assert!(app.run("closet.set", json!({"key": "tool:nosuch", "shelf": "closet"})).is_err());
    assert!(app.run("closet.set", json!({"key": "tool:pen", "shelf": "attic"})).is_err());
}

#[test]
fn bringing_everything_back_empties_both_shelves_and_reset_restores_the_sort() {
    let mut app = app();
    let all = app.run("closet.reset", json!({"all": true})).unwrap();
    assert_eq!(all, json!({"closet": [], "basement": []}));
    assert!(toolbar(&app).contains(&"pencil") && labels(&menu_items(&app, "Effect")).contains(&"Blur"));
    let back = app.run("closet.reset", json!({})).unwrap();
    assert!(back["closet"].as_array().unwrap().contains(&json!("tool:pencil")));
    assert!(back["basement"].as_array().unwrap().contains(&json!("panel:brushes")));
}

#[test]
fn every_menu_key_lists_once() {
    let mut keys: Vec<String> = things().into_iter().map(|t| t.key).collect();
    let n = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), n);
    assert!(keys.iter().any(|k| k == "menu:Effect › Stylize [Raster Effects]"));
}

#[test]
fn the_panel_draws_each_tab_headless() {
    let mut app = app();
    for tab in 0..TABS.len() {
        let ctx = egui::Context::default();
        crate::panels::set_pstate(&ctx, "closet-tab", tab);
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                show(&mut app, ui);
                menu(&mut app, ui);
            });
            out.textures_delta.clear();
        }
    }
}
