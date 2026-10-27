//! Formats VectorCraft doesn't read or write (and what to use instead), and a small param helper
//! the exporters share.

use serde_json::{Value, json};

/// A format that can be neither opened nor written, and what to use instead.
#[derive(Clone, Copy, Debug)]
pub struct Unsupported {
    pub id: &'static str,
    pub label: &'static str,
    /// Lower-case extensions without the dot.
    pub extensions: &'static [&'static str],
    pub hint: &'static str,
}

/// Formats people ask for that VectorCraft doesn't read or write. Append-only.
pub const UNSUPPORTED: &[Unsupported] = &[
    Unsupported {
        id: "dwg",
        label: "DWG",
        extensions: &["dwg"],
        hint: "DWG is not available (no openly licensed writer exists): export PDF or SVG instead",
    },
    Unsupported {
        id: "pict",
        label: "PICT",
        extensions: &["pict", "pct", "pic"],
        hint: "PICT is a retired format and is not supported: convert the file to PDF or PNG first",
    },
    Unsupported {
        id: "pmd",
        label: "PageMaker",
        extensions: &["pmd"],
        hint: "PageMaker (.pmd) is not supported: open the layout in PageMaker and export PDF or EPS, or place a PDF export here",
    },
    Unsupported {
        id: "dxf",
        label: "DXF",
        extensions: &["dxf"],
        hint: "This build of VectorCraft leaves DXF out: export PDF or SVG instead",
    },
    Unsupported {
        id: "emf",
        label: "EMF",
        extensions: &["emf"],
        hint: "This build of VectorCraft leaves EMF out: export PDF or SVG instead",
    },
    Unsupported {
        id: "wmf",
        label: "WMF",
        extensions: &["wmf"],
        hint: "This build of VectorCraft leaves WMF out: export PDF or SVG instead",
    },
    Unsupported {
        id: "affinity",
        label: "Affinity",
        extensions: &["af", "afdesign", "afpub", "afphoto"],
        hint: "This build of VectorCraft doesn't open Affinity files: export PDF or SVG from Affinity instead",
    },
];

/// The unsupported format an id or extension (any case, leading dot allowed) names.
pub fn unsupported(id_or_ext: &str) -> Option<&'static Unsupported> {
    let k = id_or_ext.trim_start_matches('.').to_ascii_lowercase();
    UNSUPPORTED.iter().find(|u| u.id == k || u.extensions.contains(&k.as_str()))
}

impl Unsupported {
    pub fn to_json(&self) -> Value {
        json!({ "id": self.id, "label": self.label, "extensions": self.extensions, "hint": self.hint })
    }
}

/// A string or number param as text (`2018` and `"2018"` alike).
pub(crate) fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
