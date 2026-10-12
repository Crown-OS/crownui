//! Declarative views for CrownUI. See `README.md`.
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod animation_frames;
mod app_icons;
mod color_scheme;
mod component;
mod cx;
mod dispatch;
mod drop_target;
mod dynamic;
mod element;
mod event;
mod handler;
mod layout;
mod lazy;
mod listeners;
mod modifier;
mod node;
mod pointer_events;
mod prop;
mod render_op;
mod semantic;
mod semantics_build;
mod style;
mod style_prop;
mod styled;
mod tasks;
mod text_cache;
mod tools;
mod tree;
mod ui;
mod util;
mod view;

pub use app_icons::AppIconRequest;
pub use color_scheme::ColorScheme;
pub use component::{Component, component};
pub use cx::{ClipboardRequest, Cx, ImageUpload};
pub use dynamic::{
    DynamicChildren, ErrorBoundaryKind, ForEachKind, KeyedKind, ShowKind, error_boundary, for_each, keyed, show,
};
pub use element::{Element, Kind, VariantStyle};
pub use event::{
    ClickEvent, DragEvent, DragLeaveEvent, DragOverEvent, DropEvent, Event, FocusEvent, InputEvent, LongPressEvent,
    PinchEvent, ScrollEvent,
};
pub use handler::{Handler, Listener, ListenerKind};
pub use layout::LayoutState;
pub use lazy::{LazyForEachKind, Viewport, lazy_for_each};
pub use listeners::Listeners;
pub use modifier::{Modifier, SemanticProp, Variant, VariantProp};
pub use pointer_events::PointerEvents;
pub use node::{ElementNode, NodeContent, NodeGeometry, NodeId};
pub use prop::{Bind, IntoProp, Prop, bind};
pub use render_op::{NodeKey, RenderContent, RenderOp, UiFrame};
pub use semantic::Semantic;
pub use style::{NodeStyle, Shadow, TextOverrides, Visual};
pub use style_prop::{Corners, Edges, SizeProperty, StyleImpact, StyleProp};
pub use styled::Styled;
pub use tasks::{TaskExecutor, TaskFuture, TaskId};
pub use text_cache::{TextCache, TextMetrics};
pub use tools::{AppTool, ToolInfo};
pub use tree::ElementTree;
pub use ui::{ActionError, Ui};
pub use view::{AnyView, View, ViewSeq};
