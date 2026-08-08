pub mod network;
pub mod audio;
pub mod display;
pub mod ebpf;
pub mod module_item;
pub mod panel;

pub use network::*;
pub use audio::*;
pub use display::*;
pub use ebpf::*;
pub use module_item::*;
pub use panel::*;

use gtk4::Application;
use relm4::{Component, ComponentController};

thread_local! {
    static CONTROL_CENTER_CTRL: std::cell::RefCell<Option<relm4::component::Connector<panel::ControlCenterPanel>>> = const { std::cell::RefCell::new(None) };
}

pub fn show_control_center_panel(app: &Application) {
    use crate::ui::topbar::toggle_or_open_popup;
    toggle_or_open_popup("control-center", || {
        CONTROL_CENTER_CTRL.with(|c| {
            let mut cell = c.borrow_mut();
            if let Some(ctrl) = cell.as_ref() {
                let _ = ctrl.sender().send(panel::CcPanelInput::ToggleVisible);
            } else {
                let ctrl = panel::ControlCenterPanel::builder().launch(app.clone());
                *cell = Some(ctrl);
            }
        });
    });
}
