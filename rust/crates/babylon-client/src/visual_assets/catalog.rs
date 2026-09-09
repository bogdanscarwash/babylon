//! Typed identifiers and manifest-ordered metadata for visual assets.

/// A stable identifier for one embedded visual asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualAssetId {
    /// The Babylon title mark.
    TitleMark,
    /// The sixteen-cell interface icon atlas.
    InterfaceAtlas,
    /// The six-cell map-marker atlas.
    MarkerAtlas,
    /// The four-cell projection-provenance atlas.
    ProvenanceAtlas,
    /// The four-cell frame atlas.
    FrameAtlas,
    /// The three-cell surface-texture atlas.
    SurfaceAtlas,
    /// The Red Apparatus hero illustration.
    HeroRedApparatus,
    /// The Empire Anatomized hero illustration.
    HeroEmpireAnatomized,
    /// The Bunker Oracle concept illustration.
    ConceptBunkerOracle,
    /// The Living Map concept illustration.
    ConceptLivingMap,
    /// The Carceral Circuit concept illustration.
    ConceptCarceralCircuit,
    /// The Metabolic Rift concept illustration.
    ConceptMetabolicRift,
    /// The counties-story banner.
    BannerCounties,
    /// The carceral-story banner.
    BannerCarceral,
    /// The topology-story banner.
    BannerTopology,
    /// The collapse-story banner.
    BannerCollapse,
}

/// Immutable metadata for one entry in the visual-asset catalog.
pub struct VisualAssetDescriptor {
    /// The typed identifier used to retrieve this image.
    pub id: VisualAssetId,
    /// The player-facing name used in visual asset views.
    pub label: &'static str,
    /// The source image width in pixels.
    pub width: u32,
    /// The source image height in pixels.
    pub height: u32,
    /// The number of atlas columns.
    pub columns: u32,
    /// The number of atlas rows.
    pub rows: u32,
}

/// The fixed, manifest-ordered catalog of all embedded visual images.
pub const VISUAL_ASSET_CATALOG: [VisualAssetDescriptor; 16] = [
    VisualAssetDescriptor {
        id: VisualAssetId::TitleMark,
        label: "Title mark",
        width: 768,
        height: 192,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::InterfaceAtlas,
        label: "Interface atlas",
        width: 512,
        height: 512,
        columns: 4,
        rows: 4,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::MarkerAtlas,
        label: "Marker atlas",
        width: 384,
        height: 256,
        columns: 3,
        rows: 2,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::ProvenanceAtlas,
        label: "Provenance atlas",
        width: 256,
        height: 256,
        columns: 2,
        rows: 2,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::FrameAtlas,
        label: "Frame atlas",
        width: 256,
        height: 64,
        columns: 4,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::SurfaceAtlas,
        label: "Surface atlas",
        width: 384,
        height: 128,
        columns: 3,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::HeroRedApparatus,
        label: "Red Apparatus",
        width: 1536,
        height: 864,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::HeroEmpireAnatomized,
        label: "Empire Anatomized",
        width: 1536,
        height: 864,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::ConceptBunkerOracle,
        label: "Bunker Oracle",
        width: 1024,
        height: 1024,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::ConceptLivingMap,
        label: "Living Map",
        width: 1024,
        height: 1024,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::ConceptCarceralCircuit,
        label: "Carceral Circuit",
        width: 1024,
        height: 1024,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::ConceptMetabolicRift,
        label: "Metabolic Rift",
        width: 1024,
        height: 1024,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::BannerCounties,
        label: "Counties banner",
        width: 1536,
        height: 384,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::BannerCarceral,
        label: "Carceral banner",
        width: 1536,
        height: 384,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::BannerTopology,
        label: "Topology banner",
        width: 1536,
        height: 384,
        columns: 1,
        rows: 1,
    },
    VisualAssetDescriptor {
        id: VisualAssetId::BannerCollapse,
        label: "Collapse banner",
        width: 1536,
        height: 384,
        columns: 1,
        rows: 1,
    },
];

/// A cell in the sixteen-icon interface atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum InterfaceIcon {
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// Step.
    Step,
    /// Speed.
    Speed,
    /// Lens.
    Lens,
    /// Map.
    Map,
    /// Story.
    Story,
    /// Beat.
    Beat,
    /// Roster.
    Roster,
    /// Material.
    Material,
    /// Topology.
    Topology,
    /// Flow.
    Flow,
    /// Pin.
    Pin,
    /// Inspect.
    Inspect,
    /// Warning.
    Warning,
    /// Close.
    Close,
}

impl InterfaceIcon {
    /// The number of cells in the interface atlas.
    pub const COUNT: usize = 16;

    /// Returns this icon's atlas index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// A cell in the six-marker map atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum MarkerIcon {
    /// Hover state.
    Hover,
    /// Selection state.
    Selection,
    /// Pinned state.
    Pin,
    /// Event state.
    Event,
    /// Origin state.
    Origin,
    /// Target state.
    Target,
}

impl MarkerIcon {
    /// The number of cells in the marker atlas.
    pub const COUNT: usize = 6;

    /// Returns this marker's atlas index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// A cell in the four-state provenance atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ProvenanceIcon {
    /// Material provenance.
    Material,
    /// Absent provenance.
    Absent,
    /// Not-computed provenance.
    NotComputed,
    /// Redacted provenance.
    Redacted,
}

impl ProvenanceIcon {
    /// The number of cells in the provenance atlas.
    pub const COUNT: usize = 4;

    /// Returns this provenance icon's atlas index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// A cell in the four-frame atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum FrameKind {
    /// Neutral frame.
    Neutral,
    /// Selected frame.
    Selected,
    /// Critical frame.
    Critical,
    /// Absent frame.
    Absent,
}

impl FrameKind {
    /// The number of cells in the frame atlas.
    pub const COUNT: usize = 4;

    /// Returns this frame's atlas index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// A cell in the three-surface atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum SurfaceKind {
    /// Concrete surface.
    Concrete,
    /// Hatch surface.
    Hatch,
    /// Paper surface.
    Paper,
}

impl SurfaceKind {
    /// The number of cells in the surface atlas.
    pub const COUNT: usize = 3;

    /// Returns this surface's atlas index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}
