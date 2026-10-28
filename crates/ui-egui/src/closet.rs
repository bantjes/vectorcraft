//! The Closet and the Basement (Window › Closet): tools, panels and menu items put out of sight
//! without deleting anything. The Closet holds what is used now and then, one click from coming
//! back; the Basement, a tab further away, holds what will probably never be used. A tool put away
//! leaves the toolbar, the All Tools drawer and its key; a panel leaves the Window menu and the
//! dock; a menu item or submenu leaves its menu. Restoring puts each back where it was.
//!
//! Everything is named by a key: `tool:<id>`, `panel:<id>` or `menu:<Menu › Submenu › Item>` (the
//! English labels; a label its menu repeats gets the heading above it in brackets, as the Effect
//! menu's raster `Stylize [Raster Effects]`). [`SHIPPED`] is the sort this build starts with;
//! `UiState::closet` keeps only what was moved since, so it is saved with the preferences.

use std::collections::BTreeMap;

use egui::Ui;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vectorcraft_tools::TOOL_GROUPS;

use crate::menus::Item;
use crate::state::UiState;
use crate::theme::Tokens;
use crate::{VectorcraftApp, icons, widgets};

/// The Closet's own panel id (it can't be put away).
pub const ID: &str = "closet";

/// Where something lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shelf {
    InUse,
    Closet,
    Basement,
}

impl Shelf {
    const ALL: [Self; 3] = [Self::InUse, Self::Closet, Self::Basement];

    fn name(self) -> &'static str {
        match self {
            Self::InUse => "inUse",
            Self::Closet => "closet",
            Self::Basement => "basement",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|shelf| shelf.name() == s)
    }
}

/// What was moved since [`SHIPPED`] (saved with the preferences).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Closet {
    pub moved: BTreeMap<String, Shelf>,
}

use Shelf::{Basement, Closet as InCloset};

/// The sort this build starts with (Emma's lists of 27 and 28 October 2026).
pub const SHIPPED: &[(&str, Shelf)] = &[
    // Panels.
    ("panel:symbols", InCloset),
    ("panel:libraries", InCloset),
    ("panel:artboards", InCloset),
    ("panel:actions", InCloset),
    ("panel:brushes", Basement),
    ("panel:appearance", Basement),
    ("panel:graphicStyles", Basement),
    ("panel:variables", Basement),
    // Shapes, drawing and cutting.
    ("tool:arc", InCloset),
    ("tool:pencil", InCloset),
    ("tool:smooth", InCloset),
    ("tool:pathEraser", InCloset),
    ("tool:eraser", InCloset),
    ("tool:knife", InCloset),
    ("tool:mirrorCut", InCloset),
    ("tool:lineCut", InCloset),
    ("tool:rectCut", InCloset),
    ("tool:paintbrush", InCloset),
    ("tool:blobBrush", InCloset),
    ("tool:curvature", InCloset),
    // Navigation.
    ("tool:zoom", InCloset),
    ("tool:printTiling", InCloset),
    // Modify: all but Rotate, Reflect, Scale and Blend.
    ("tool:width", Basement),
    ("tool:warp", Basement),
    ("tool:twirl", Basement),
    ("tool:pucker", Basement),
    ("tool:bloat", Basement),
    ("tool:scallop", Basement),
    ("tool:crystallize", Basement),
    ("tool:wrinkle", Basement),
    ("tool:shear", Basement),
    ("tool:reshape", Basement),
    ("tool:freeTransform", Basement),
    ("tool:puppetWarp", Basement),
    ("tool:shapeBuilder", Basement),
    ("tool:livePaintBucket", Basement),
    ("tool:livePaintSelection", Basement),
    ("tool:dimension", Basement),
    ("tool:mesh", Basement),
    // The Photoshop-style raster effects.
    ("menu:Effect › Effect Gallery…", InCloset),
    ("menu:Effect › Artistic", InCloset),
    ("menu:Effect › Blur", InCloset),
    ("menu:Effect › Brush Strokes", InCloset),
    ("menu:Effect › Distort", InCloset),
    ("menu:Effect › Pixelate", InCloset),
    ("menu:Effect › Sharpen", InCloset),
    ("menu:Effect › Sketch", InCloset),
    ("menu:Effect › Stylize [Raster Effects]", InCloset),
    ("menu:Effect › Texture", InCloset),
    ("menu:Effect › Video", InCloset),
];

/// Where [`SHIPPED`] puts `key` (in use when it doesn't list it).
fn shipped(key: &str) -> Shelf {
    #[cfg(test)]
    if !SORTED.get() {
        return Shelf::InUse;
    }
    SHIPPED.iter().find(|(k, _)| *k == key).map_or(Shelf::InUse, |(_, s)| *s)
}

#[cfg(test)]
thread_local! {
    /// Tests start with everything in use, so the app's own tests see every tool, panel and menu
    /// item; the Closet's tests turn the starting sort on.
    static SORTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn use_starting_sort() {
    SORTED.set(true);
}

/// Where `key` lives now.
pub fn shelf(ui: &UiState, key: &str) -> Shelf {
    ui.closet.moved.get(key).copied().unwrap_or_else(|| shipped(key))
}

/// Is `key` put away (in the Closet or the Basement)?
pub fn hidden(ui: &UiState, key: &str) -> bool {
    shelf(ui, key) != Shelf::InUse
}

pub fn tool_hidden(ui: &UiState, id: &str) -> bool {
    hidden(ui, &format!("tool:{id}"))
}

pub fn panel_hidden(ui: &UiState, id: &str) -> bool {
    id != ID && hidden(ui, &format!("panel:{id}"))
}

/// The toolbar's slots without the tools put away; a category whose first slot is left empty
/// names the next one.
pub(crate) fn visible_slots(ui: &UiState, slots: Vec<(Option<&'static str>, Vec<&'static str>)>) -> Vec<(Option<&'static str>, Vec<&'static str>)> {
    let mut out = Vec::new();
    let mut pending = None;
    for (cat, tools) in slots {
        let tools: Vec<&'static str> = tools.into_iter().filter(|t| !tool_hidden(ui, t)).collect();
        if cat.is_some() {
            pending = cat;
        }
        if !tools.is_empty() {
            out.push((pending.take(), tools));
        }
    }
    out
}

// ---------- menus ----------

/// The key segment of each item of a menu (none for separators and headings): its label, and for a
/// label met before in the same menu, the heading above it in brackets (or its count).
fn segments(items: &[Item]) -> Vec<Option<String>> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut header: Option<&str> = None;
    items
        .iter()
        .map(|item| {
            let label = match item {
                Item::Cmd(l, ..) | Item::Todo(l, _) | Item::Sub(l, _) => *l,
                Item::Header(h) => {
                    header = Some(h);
                    return None;
                }
                Item::Sep => return None,
            };
            let n = seen.entry(label).or_default();
            *n += 1;
            Some(match (*n, header) {
                (1, _) => label.to_string(),
                (_, Some(h)) => format!("{label} [{h}]"),
                (n, None) => format!("{label} [{n}]"),
            })
        })
        .collect()
}

/// The panel a Window menu item shows, if it is one.
fn panel_of(item: &Item) -> Option<&str> {
    match item {
        Item::Cmd(_, "window.panel", p) => p.get("panel").and_then(Value::as_str),
        _ => None,
    }
}

/// `tree` without what is put away.
pub fn prune_tree(ui: &UiState, tree: Vec<(&'static str, Vec<Item>)>) -> Vec<(&'static str, Vec<Item>)> {
    tree.into_iter().map(|(title, items)| (title, prune(ui, title, items))).collect()
}

fn prune(ui: &UiState, path: &str, items: Vec<Item>) -> Vec<Item> {
    let segs = segments(&items);
    let mut out = Vec::with_capacity(items.len());
    for (item, seg) in items.into_iter().zip(segs) {
        if panel_of(&item).is_some_and(|p| panel_hidden(ui, p)) {
            continue;
        }
        let Some(seg) = seg else {
            out.push(item);
            continue;
        };
        let key = format!("{path} › {seg}");
        if hidden(ui, &format!("menu:{key}")) {
            continue;
        }
        match item {
            Item::Sub(label, sub) => {
                let sub = prune(ui, &key, sub);
                if sub.iter().any(|i| !matches!(i, Item::Sep | Item::Header(_))) {
                    out.push(Item::Sub(label, sub));
                }
            }
            item => out.push(item),
        }
    }
    tidy(out)
}

/// No separator first, last or twice running, and no heading over nothing.
fn tidy(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    for item in items {
        if matches!(item, Item::Sep | Item::Header(_)) {
            // A heading left with nothing under it goes.
            if matches!(out.last(), Some(Item::Header(_))) {
                out.pop();
            }
            if matches!(item, Item::Sep) && matches!(out.last(), None | Some(Item::Sep)) {
                continue;
            }
        }
        out.push(item);
    }
    while matches!(out.last(), Some(Item::Sep | Item::Header(_))) {
        out.pop();
    }
    out
}

// ---------- what can be put away ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Tool,
    Panel,
    Menu,
}

impl Kind {
    fn heading(self) -> &'static str {
        match self {
            Self::Tool => "Tools",
            Self::Panel => "Panels",
            Self::Menu => "Menus and Effects",
        }
    }
}

/// One thing that can be put away.
#[derive(Clone, Debug)]
pub struct Thing {
    pub key: String,
    pub kind: Kind,
    /// Its name in the UI language (a menu item's whole path).
    pub label: String,
    pub icon: Option<&'static str>,
}

/// Every tool, panel and menu item, in that order. Menu items a menu repeats as a list (recent
/// files, saved views) aren't listed, nor the Window menu's panels (listed as panels).
pub fn things() -> Vec<Thing> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for t in TOOL_GROUPS.iter().flat_map(|g| g.iter()) {
        if seen.insert(t.id) {
            out.push(Thing { key: format!("tool:{}", t.id), kind: Kind::Tool, label: tl!(t.label).to_string(), icon: Some(icons::tool_icon(t.icon)) });
        }
    }
    for (id, label) in crate::state::all_panels() {
        if id != ID {
            let icon = crate::state::rail_panel(id).map(|p| p.2);
            out.push(Thing { key: format!("panel:{id}"), kind: Kind::Panel, label: tl!(label).to_string(), icon });
        }
    }
    for (title, items) in crate::menus::menu_tree() {
        menu_things(title, &tl!(title).to_string(), &items, &mut out);
    }
    out
}

fn menu_things(path: &str, shown: &str, items: &[Item], out: &mut Vec<Thing>) {
    let segs = segments(items);
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for item in items {
        if let Item::Cmd(l, ..) | Item::Todo(l, _) = item {
            *count.entry(*l).or_default() += 1;
        }
    }
    for (item, seg) in items.iter().zip(segs) {
        let Some(seg) = seg else { continue };
        if panel_of(item).is_some() {
            continue;
        }
        let key = format!("{path} › {seg}");
        match item {
            Item::Cmd(l, ..) | Item::Todo(l, _) => {
                // A list (Open Recent Files' slots) isn't put away one row at a time.
                if count.get(l).copied().unwrap_or(0) <= 2 {
                    out.push(Thing { key: format!("menu:{key}"), kind: Kind::Menu, label: format!("{shown} › {}", tl!(l)), icon: None });
                }
            }
            Item::Sub(l, sub) => {
                let shown = format!("{shown} › {}", tl!(l));
                out.push(Thing { key: format!("menu:{key}"), kind: Kind::Menu, label: shown.clone(), icon: None });
                menu_things(&key, &shown, sub, out);
            }
            Item::Sep | Item::Header(_) => {}
        }
    }
}

// ---------- commands ----------

/// `closet.set {key, shelf}`, `closet.reset {all?}` and `closet.list {shelf?}`; `None` for other ids.
pub(crate) fn command(app: &mut VectorcraftApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    Some(match id {
        "closet.set" => (|| {
            let key = p.get("key").and_then(Value::as_str).ok_or("key is required: tool:<id>, panel:<id> or menu:<path>")?;
            let shelf = p.get("shelf").and_then(Value::as_str).and_then(Shelf::parse).ok_or("shelf must be inUse, closet or basement")?;
            if key == format!("panel:{ID}") || key == "menu:Window › Closet" {
                return Err("the Closet can't be put away".into());
            }
            if !things().iter().any(|t| t.key == key) {
                return Err(format!("nothing called `{key}` can be put away"));
            }
            set(app, key, shelf);
            Ok(json!({ "key": key, "shelf": shelf.name() }))
        })(),
        "closet.reset" => (|| {
            let all = crate::menus::opt_bool(p, "all")?.unwrap_or(false);
            app.ui.closet.moved.clear();
            if all {
                for (key, _) in SHIPPED {
                    app.ui.closet.moved.insert(key.to_string(), Shelf::InUse);
                }
            }
            leave_hidden_tool(app);
            Ok(list(app, None))
        })(),
        "closet.list" => (|| {
            let shelf = match p.get("shelf").and_then(Value::as_str) {
                None => None,
                Some(s) => Some(Shelf::parse(s).ok_or("shelf must be inUse, closet or basement")?),
            };
            Ok(list(app, shelf))
        })(),
        _ => return None,
    })
}

/// Move `key` to `shelf`, remembering only what differs from [`SHIPPED`].
fn set(app: &mut VectorcraftApp, key: &str, to: Shelf) {
    if to == shipped(key) {
        app.ui.closet.moved.remove(key);
    } else {
        app.ui.closet.moved.insert(key.to_string(), to);
    }
    leave_hidden_tool(app);
}

/// The active tool put away gives way to the Selection tool.
fn leave_hidden_tool(app: &mut VectorcraftApp) {
    if tool_hidden(&app.ui, app.session.tool_id()) {
        app.select_tool("selection");
    }
}

/// The keys on `shelf` (every key put away, by shelf, without one).
fn list(app: &VectorcraftApp, shelf: Option<Shelf>) -> Value {
    let things = things();
    let on = |s: Shelf| things.iter().filter(|t| self::shelf(&app.ui, &t.key) == s).map(|t| t.key.clone()).collect::<Vec<_>>();
    match shelf {
        Some(s) => json!(on(s)),
        None => json!({ "closet": on(Shelf::Closet), "basement": on(Shelf::Basement) }),
    }
}

// ---------- the panel ----------

/// The panel's tabs.
const TABS: [&str; 3] = ["Closet", "Basement", "Everything"];

pub fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let tab: usize = crate::panels::pstate(ui.ctx(), "closet-tab");
    if let Some(i) = widgets::tab_bar(ui, &TABS, tab) {
        crate::panels::set_pstate(ui.ctx(), "closet-tab", i);
    }
    let tab = TABS.get(tab).copied().unwrap_or("Closet");
    let (icon, blurb) = match tab {
        "Closet" => ("dc-closet", "Put away, one click from coming back."),
        "Basement" => ("dc-basement", "Things you'll probably never need. Nothing here is deleted."),
        _ => ("dc-closet", "Everything that can be put away. Pick where each one lives."),
    };
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
        icons::paint(ui, icon, r, t.text_dim);
        ui.label(egui::RichText::new(tl!(blurb)).size(11.5).color(t.text_dim));
    });
    ui.add_space(4.0);
    let query = widgets::search_field(ui, egui::Id::unique("closet-search"), "Search");
    let query = query.trim().to_lowercase();
    ui.add_space(4.0);
    let mut moves: Vec<(String, Shelf)> = Vec::new();
    let wanted = |s: Shelf| match tab {
        "Closet" => s == Shelf::Closet,
        "Basement" => s == Shelf::Basement,
        _ => true,
    };
    let things: Vec<Thing> = things()
        .into_iter()
        .filter(|th| wanted(shelf(&app.ui, &th.key)))
        .filter(|th| query.is_empty() || th.label.to_lowercase().contains(&query))
        .collect();
    egui::ScrollArea::vertical().auto_shrink([false, true]).max_height(ui.available_height().max(120.0) - 30.0).show(ui, |ui| {
        if things.is_empty() {
            ui.add_space(8.0);
            let empty = match tab {
                "Closet" => "The Closet is empty.",
                "Basement" => "The Basement is empty.",
                _ => "Nothing matches.",
            };
            ui.label(egui::RichText::new(tl!(empty)).color(t.text_dim));
        }
        let mut kind = None;
        for th in &things {
            if kind != Some(th.kind) {
                kind = Some(th.kind);
                ui.add_space(6.0);
                widgets::section_header(ui, tl!(th.kind.heading()));
            }
            let now = shelf(&app.ui, &th.key);
            ui.horizontal(|ui| {
                ui.set_min_height(26.0);
                if let Some(icon) = th.icon {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                    icons::paint(ui, icon, r, t.text);
                }
                ui.add(egui::Label::new(egui::RichText::new(&th.label).color(t.text)).truncate());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right to left: the far shelf last.
                    let buttons: &[(&str, Shelf)] = match now {
                        Shelf::Closet => &[("To Basement", Shelf::Basement), ("Restore", Shelf::InUse)],
                        Shelf::Basement => &[("To Closet", Shelf::Closet), ("Restore", Shelf::InUse)],
                        Shelf::InUse => &[("To Basement", Shelf::Basement), ("To Closet", Shelf::Closet)],
                    };
                    for (label, to) in buttons {
                        if widgets::flat_button_fit(ui, label, 64.0).clicked() {
                            moves.push((th.key.clone(), *to));
                        }
                    }
                });
            });
        }
    });
    for (key, to) in moves {
        if let Err(e) = app.run("closet.set", json!({ "key": key, "shelf": to.name() })) {
            app.ui.status = e;
        }
    }
}

pub fn menu(app: &mut VectorcraftApp, ui: &mut Ui) {
    if widgets::menu_item(ui, tl!("Restore My Starting Sort"), true, false) {
        app.run("closet.reset", json!({})).ok();
    }
    if widgets::menu_item(ui, tl!("Bring Everything Back"), true, false) {
        app.run("closet.reset", json!({ "all": true })).ok();
    }
}

/// Opens the Closet panel.
pub fn open(app: &mut VectorcraftApp) {
    if let Err(e) = app.run("window.panel", json!({ "panel": ID })) {
        app.ui.status = e;
    }
}

#[cfg(test)]
mod tests;
