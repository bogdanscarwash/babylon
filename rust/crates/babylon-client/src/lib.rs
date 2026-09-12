//! Durable campaign observer, geographic presentation, and headless Archive commands.

// Bevy system parameters are passed by value by the SystemParam API.
#![allow(clippy::needless_pass_by_value)]

pub mod app;
pub mod atlas;
pub mod cli;
pub mod decision_surface;
pub mod dossier;
pub mod logging;
pub mod map;
pub mod map_economy_lens;
pub mod observer;
pub(crate) mod observer_controls;
pub(crate) mod observer_focus;
pub mod observer_io;
pub(crate) mod observer_layout;
pub mod observer_ui;
pub(crate) mod observer_warning;
pub mod palette;
pub mod session_log;
pub mod tessellate;
#[cfg(test)]
mod test_support;
pub mod ui;
pub mod visual_assets;

pub mod campaign_browser;
pub mod observer_audio;
pub mod observer_history;
pub mod observer_map3d;
pub mod observer_performance;
pub mod observer_theme;
pub mod production;
pub(crate) mod production_brief;
pub(crate) mod production_freight;
mod production_layout;

mod material_relations;
mod workforce;
