use clap::Parser;
use gtk4::prelude::*;
use gtk4::{gio, Application};

// Dummy modules for now
mod ui;
mod greeter;
mod core;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    topbar: bool,
    #[arg(long)]
    greeter: bool,
    #[arg(long)]
    lock: bool,
    #[arg(long)]
    spotlight: bool,
    #[arg(long)]
    launcher: bool,
    #[arg(long)]
    dock: bool,
    #[arg(long)]
    control_center: bool,
    #[arg(long)]
    media_player: bool,
    #[arg(long)]
    sys_monitor: bool,
    #[arg(long)]
    calendar: bool,
    #[arg(long)]
    powermenu: bool,
    #[arg(long)]
    gatekeeper_prompt: Option<String>,
    #[arg(long)]
    privacy_prompt: Option<String>,
    #[arg(long)]
    overview: bool,
    #[arg(long)]
    store: bool,
}

const APP_ID: &str = "os.ermete.Shell";

fn main() -> glib::ExitCode {
    // SAFETY: Called at the very start of main(), before user Rust threads are created.
    // Uses C-level setenv to avoid UB concerns with Rust's set_var in multithreaded contexts.
    // Workaround for Vulkan swapchain resizing panic (VK_ERROR_OUT_OF_DATE_KHR) on Wayland.
    unsafe {
        libc::setenv(
            b"GSK_RENDERER\0".as_ptr() as *const libc::c_char,
            b"ngl\0".as_ptr() as *const libc::c_char,
            1,
        );
    }

    let args = Args::parse();
    crate::core::system_proxies::init_system_controller();

    if let Some(req_info) = args.privacy_prompt {
        let app = Application::builder()
            .application_id("os.ermete.PrivacyPrompt")
            .build();
        let req_clone = req_info.clone();
        app.connect_activate(move |app| {
            crate::ui::privacy_prompt::build_ui(app, &req_clone);
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    if let Some(app_path) = args.gatekeeper_prompt {
        let app = Application::builder()
            .application_id("os.ermete.GatekeeperPrompt")
            .build();
        let path_clone = app_path.clone();
        app.connect_activate(move |app| {
            crate::ui::gatekeeper_prompt::build_ui(app, &path_clone);
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    if args.overview {
        let app = Application::builder()
            .application_id("os.ermete.MissionControl")
            .build();
        app.connect_activate(|app| {
            crate::ui::mission_control::build_ui(app);
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    if args.store {
        let app = Application::builder()
            .application_id("os.ermete.StoreUI")
            .build();
        app.connect_activate(|app| {
            crate::ui::store::show_store_modal(app);
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    // If greeter or lock mode is requested explicitly, run standalone authentication app
    if args.greeter || args.lock {
        let is_lock = args.lock;
        let app_id = if is_lock { "os.ermete.Lockscreen" } else { "os.ermete.Greeter" };
        let app = Application::builder()
            .application_id(app_id)
            .build();
        app.connect_activate(move |app| {
            greeter::build_ui(app, is_lock);
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    if args.dock {
        let app = Application::builder()
            .application_id("os.ermete.Dock")
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();
        app.connect_activate(|app| {
            static ACTIVATED_DOCK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !ACTIVATED_DOCK.swap(true, std::sync::atomic::Ordering::SeqCst) {
                ui::dock::build_ui(app);
            } else {
                ui::dock::toggle_dock_visibility();
            }
        });
        app.connect_command_line(|app, _cmdline| {
            app.activate();
            0
        });
        return app.run();
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_activate(move |app| {
        static ACTIVATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !ACTIVATED.swap(true, std::sync::atomic::Ordering::SeqCst) {
            use relm4::Component;
            let ctrl = ui::topbar_relm4::TopbarModel::builder()
                .launch(app.clone());
            
            // Salviamo il controller globalmente per non far spegnere la runtime
            thread_local! {
                static TOPBAR_CTRL: std::cell::RefCell<Option<Box<dyn std::any::Any>>> = std::cell::RefCell::new(None);
            }
            TOPBAR_CTRL.with(|c| *c.borrow_mut() = Some(Box::new(ctrl)));
                
            crate::ui::osd::spawn_osd(app);
            crate::ui::desktop_widgets::build_desktop_widgets(app);
        }
    });

    app.connect_command_line(move |app, cmdline| {
        app.activate();
        let args = cmdline.arguments();
        for arg in args.iter().skip(1) {
            let s = arg.to_string_lossy();
            let clean = s.trim_start_matches("--");
            crate::ui::topbar::handle_command(app, clean);
        }
        0
    });

    // Pass original CLI args to run so GTK forwards them to primary instance
    app.run()
}
