pub mod anim;
pub mod topbar;
pub use ermete_dock::ui as dock;
pub mod control_center;
pub mod notifications;
pub mod osd;
pub mod powermenu;
pub mod spotlight;
pub mod clipboard;
pub mod prompts;
pub mod greeter;
pub mod mission_control;
pub mod desktop_widgets;
pub mod store;

pub use crate::wayland::popup as popup_manager;
pub use prompts::gatekeeper as gatekeeper_prompt;
pub use prompts::privacy as privacy_prompt;
