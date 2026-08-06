use gtk4::prelude::*;
use gtk4::ApplicationWindow;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

thread_local! {
    pub static ACTIVE_POPUP: std::cell::RefCell<Option<(String, glib::WeakRef<ApplicationWindow>)>> = const { std::cell::RefCell::new(None) };
}


pub fn setup_popup_autoclose(pop: &ApplicationWindow, tag: &str) {
    let mut to_close = None;
    ACTIVE_POPUP.with(|p| {
        if let Some((_, old_weak)) = p.borrow().as_ref() {
            if let Some(old_win) = old_weak.upgrade() {
                if old_win != *pop && old_win.is_visible() {
                    to_close = Some(old_win);
                }
            }
        }
        *p.borrow_mut() = Some((tag.to_string(), pop.downgrade()));
    });

    if let Some(win) = to_close {
        win.close();
    }

    pop.set_keyboard_mode(KeyboardMode::OnDemand);
    pop.set_namespace(tag);

    if let Some(app) = pop.application() {
        let bg_win = ApplicationWindow::builder()
            .application(&app)
            .css_classes(["bg-overlay-window"])
            .build();
            
        bg_win.init_layer_shell();
        bg_win.set_namespace("bg-overlay");
        bg_win.set_layer(Layer::Top);
        bg_win.set_anchor(Edge::Top, true);
        bg_win.set_anchor(Edge::Bottom, true);
        bg_win.set_anchor(Edge::Left, true);
        bg_win.set_anchor(Edge::Right, true);
        bg_win.set_exclusive_zone(-1);
        bg_win.set_keyboard_mode(KeyboardMode::None);
        
        let empty_box = gtk4::Box::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        bg_win.set_child(Some(&empty_box));
        
        let click = gtk4::GestureClick::new();
        click.set_button(0); // Tutti i bottoni
        let pop_close_clone = pop.clone();
        click.connect_pressed(move |_, _, _, _| {
            pop_close_clone.close();
        });
        empty_box.add_controller(click);
        
        let bg_clone = bg_win.clone();
        pop.connect_close_request(move |win| {
            bg_clone.close();
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
        
        bg_win.present();
    } else {
        pop.connect_close_request(move |win| {
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
    }

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);
}
