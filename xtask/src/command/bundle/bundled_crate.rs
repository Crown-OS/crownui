/// Whether a bundled crate shows up in the published documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Exposure {
    Documented,
    Hidden,
}

/// A crate compiled into the published `crownui` as the module `crownui::<module>`.
#[derive(Debug)]
pub(crate) struct BundledCrate {
    pub(crate) module: &'static str,
    /// Relative to the workspace root.
    pub(crate) directory: &'static str,
    /// The `crownui` feature the module is compiled under.
    pub(crate) feature: Option<&'static str>,
    pub(crate) exposure: Exposure,
    /// Text replacements adapting the crate's own features and file paths to the bundle.
    pub(crate) replacements: &'static [(&'static str, &'static str)],
}

const fn documented(module: &'static str, directory: &'static str) -> BundledCrate {
    BundledCrate {
        module,
        directory,
        feature: None,
        exposure: Exposure::Documented,
        replacements: &[],
    }
}

const fn gated(module: &'static str, feature: &'static str) -> BundledCrate {
    BundledCrate {
        feature: Some(feature),
        ..documented(module, module)
    }
}

/// Every library the `crownui` facade reaches, including the CrownOS crates outside this workspace.
pub(crate) const BUNDLED_CRATES: &[BundledCrate] = &[
    documented("geometry", "geometry"),
    documented("input", "input"),
    documented("reactive", "reactive"),
    documented("motion", "motion"),
    documented("inspect", "inspect"),
    documented("crownconfig", "crownconfig"),
    documented("windowing", "windowing"),
    documented("graphics", "graphics"),
    documented("layouting", "layouting"),
    documented("semantics", "semantics"),
    documented("gesture", "gesture"),
    gated("impeller", "impeller"),
    documented("taffy_layout", "taffy-layout"),
    gated("accessibility", "accessibility"),
    gated("mcp", "mcp"),
    documented("view", "view"),
    documented("runtime", "runtime"),
    documented("widgets", "widgets"),
    documented("crowntest", "crowntest"),
    BundledCrate {
        exposure: Exposure::Hidden,
        replacements: &[("feature = \"xilem\"", "any()")],
        ..documented("crownos_config", "../crownos-config")
    },
    BundledCrate {
        feature: Some("wayland"),
        exposure: Exposure::Hidden,
        replacements: &[
            ("feature = \"client\"", "feature = \"wayland\""),
            ("feature = \"server\"", "any()"),
            ("\"./protocols/", "\"./crownos_protocols/protocols/"),
        ],
        ..documented("crownos_protocols", "../crownos-protocols")
    },
];
