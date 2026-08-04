pub mod dock_config;
pub mod dock_data;
pub mod dock_watcher;
pub mod controller;
pub mod ui;

pub use dock_config::*;
pub use dock_data::*;
pub use dock_watcher::*;
pub use ui::{build_ui, toggle_dock_visibility};
