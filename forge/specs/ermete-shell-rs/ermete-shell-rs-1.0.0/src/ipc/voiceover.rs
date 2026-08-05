pub fn speak_text(text: String) {
    glib::MainContext::default().spawn_local(async move {
        if let Ok(connection) = zbus::Connection::session().await {
            let _ = connection.call_method(
                Some("os.ermete.VoiceOver"),
                "/os/ermete/VoiceOver",
                Some("os.ermete.VoiceOver"),
                "Speak",
                &(text,)
            ).await;
        }
    });
}

pub fn attach_voiceover_hover<W: gtk4::prelude::IsA<gtk4::Widget>>(widget: &W, text: &str) {
    let ctrl = gtk4::EventControllerMotion::new();
    let text_clone = text.to_string();
    ctrl.connect_enter(move |_, _, _| {
        speak_text(text_clone.clone());
    });
    gtk4::prelude::WidgetExt::add_controller(widget, ctrl);
}
