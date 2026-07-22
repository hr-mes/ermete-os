mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    // Initialize Tokio runtime to provide context for tokio::process inside glib futures
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    let app = Application::builder()
        .application_id("it.ermete.Store")
        .build();

    app.connect_activate(|app| {
        ui::build_ui(app);
    });

    app.run();
}
