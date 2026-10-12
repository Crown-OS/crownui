#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod run;

#[cfg(all(feature = "wayland", feature = "impeller"))]
pub use run::run;
pub use runtime::{AppConfig, RunError};
pub use widgets::kit;

/// Everything an app author needs, in one import.
pub mod prelude {
    pub use geometry::{
        BLACK, Color, CornerRadii, CornerShape, Point, Rect, Size, TRANSPARENT, Vector, WHITE, rgb,
        rgba,
    };
    pub use layouting::GridTrack;
    pub use motion::{Curve, Spring};
    pub use reactive::{Memo, Runtime, Signal, Store};
    pub use runtime::AppConfig;
    pub use semantics::Role;
    pub use view::{
        AnyView, ColorScheme, Cx, Element, Event, IntoProp, Listeners, PointerEvents, Semantic,
        Shadow, Styled, View, ViewSeq, bind, component, error_boundary, for_each, keyed, show,
    };
    pub use widgets::{
        button, canvas, cover_image, divider, hstack, icon, image, lazy_vstack, popover,
        scroll_view, spacer, svg, text, text_field, toggle, vstack, zstack,
    };
    pub use windowing::{
        Anchor, Capabilities, CursorIcon, DragIcon, DragOutcome, DragSource, KeyboardInteractivity,
        Layer, LayerExtent, LayerMargin, LayerOptions, Material, MaterialBlur, MaterialShadow,
    };
}

/// Extension points for component libraries and alternative backends.
pub mod ext {
    pub use geometry::Transform;
    pub use graphics::{
        Brush, CreateRenderer, DrawCommand, DrawList, GradientStop, GraphicsBackend, ImageId,
        Paint, Path, Renderer, Stroke, StrokeCap, StrokeJoin, TextLayout, TextStyle, TextSystem,
    };
    pub use input::{Key, Keybind, KeyboardEvent, Modifiers, NamedKey, PointerEvent, WheelEvent};
    pub use layouting::LayoutEngine;
    pub use runtime::{InlineApp, Runner};
    pub use view::{
        AppIconRequest, AppTool, ClickEvent, DragEvent, DragLeaveEvent, DragOverEvent, DropEvent,
        FocusEvent, InputEvent, Kind, Listener, ListenerKind, Modifier, NodeId, Prop, StyleProp,
        ToolInfo, Ui, Variant, VariantStyle,
    };
    pub use windowing::{AppIconPlacement, ApplicationHandler, Platform, PlatformSurface};
}
