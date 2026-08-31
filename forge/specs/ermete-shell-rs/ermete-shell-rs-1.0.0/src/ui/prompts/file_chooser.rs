use gtk4::prelude::*;
use gtk4::{Application, FileChooserNative, FileChooserAction, ResponseType};
use std::process;

pub fn build_ui(app: &Application) {
    let dialog = FileChooserNative::new(
        Some("Seleziona File (Ermete OS)"),
        gtk4::Window::NONE,
        FileChooserAction::Open,
        Some("Apri"),
        Some("Annulla"),
    );

    dialog.connect_response(|dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    // Print raw path to stdout for portal.rs to read
                    println!("{}", path.display());
                    process::exit(0);
                }
            }
        }
        process::exit(1);
    });

    dialog.show();
}
