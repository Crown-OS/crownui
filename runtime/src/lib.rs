#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod batch;
mod bridges;
mod config;
mod frame;
mod frame_clock;
mod frame_pacing;
mod frame_stats;
mod inline;
mod render;
mod render_inbox;
mod render_thread;
mod resize_band;
mod runner;
mod window_runner;

#[cfg(feature = "accessibility")]
mod accessibility_bridge;
#[cfg(feature = "tokio")]
mod executor;
#[cfg(feature = "mcp")]
mod mcp_bridge;

pub use batch::RenderBatch;
pub use config::AppConfig;
pub use frame_pacing::FramePacing;
pub use inline::{InlineApp, InlineError};
pub use render::{RenderStats, RenderTree};
pub use runner::{RunError, Runner};
