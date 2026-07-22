use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, ProgressBar, Image, Box as GtkBox, Orientation, Align};
use gtk4_layer_shell::{Layer, LayerShell, Edge};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use crate::core::live_state::get_live_state;

pub fn spawn_osd(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.add_css_class("osd-window");
    
    // Set up Layer Shell
    window.init_layer_shell();
    window.set_namespace("osd");
    window.set_layer(Layer::Overlay);
    window.set_margin(Edge::Bottom, 100);
    // Center at the bottom horizontally
    // By not setting Left/Right anchors, it naturally centers if we don't expand.
    
    window.set_default_size(200, 50);
    window.set_visible(false); // Starts hidden

    let vbox = GtkBox::new(Orientation::Vertical, 10);
    vbox.set_halign(Align::Center);
    vbox.set_valign(Align::Center);
    vbox.set_margin_top(15);
    vbox.set_margin_bottom(15);
    vbox.set_margin_start(15);
    vbox.set_margin_end(15);
    
    let icon = Image::builder()
        .icon_name("audio-volume-high-symbolic")
        .pixel_size(48)
        .build();
    
    let progress = ProgressBar::builder()
        .valign(Align::Center)
        .halign(Align::Fill)
        .hexpand(true)
        .build();
        
    vbox.append(&icon);
    vbox.append(&progress);
    
    window.set_child(Some(&vbox));
    
    let last_state = Rc::new(RefCell::new(get_live_state()));
    let hide_timeout_id = Rc::new(RefCell::new(None::<glib::SourceId>));
    
    let window_rc = window.clone();
    
    glib::timeout_add_local(Duration::from_millis(100), move || {
        let current_state = get_live_state();
        let mut last = last_state.borrow_mut();
        
        let vol_diff = (current_state.volume * 100.0 - last.volume * 100.0).abs();
        let bright_diff = (current_state.brightness - last.brightness).abs();
        
        // The condition "changes by > 1.0"
        let vol_changed = vol_diff > 1.0 || (current_state.volume - last.volume).abs() > 1.0;
        let bright_changed = bright_diff > 1.0;
        
        if vol_changed || bright_changed {
            if vol_changed {
                icon.set_icon_name(Some("audio-volume-high-symbolic"));
                progress.set_fraction(current_state.volume.clamp(0.0, 1.0));
            } else {
                icon.set_icon_name(Some("display-brightness-symbolic"));
                progress.set_fraction((current_state.brightness / 100.0).clamp(0.0, 1.0));
            }
            
            window_rc.set_visible(true);
            
            let win_clone = window_rc.clone();
            let hide_timeout_clone = hide_timeout_id.clone();
            let mut timeout_guard = hide_timeout_id.borrow_mut();
            if let Some(id) = timeout_guard.take() {
                // Ignore errors if the source was already removed (but it shouldn't be since we clear it on fire)
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    id.remove(); // Safely remove to cancel the timeout
                }));
            }
            
            *timeout_guard = Some(glib::timeout_add_local_once(Duration::from_secs(2), move || {
                win_clone.set_visible(false);
                *hide_timeout_clone.borrow_mut() = None;
            }));
            
            *last = current_state;
        }
        
        glib::ControlFlow::Continue
    });
}
