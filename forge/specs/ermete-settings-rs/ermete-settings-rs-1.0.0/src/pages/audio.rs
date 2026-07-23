use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Scale};

#[zbus::dbus_proxy(
    interface = "os.ermete.Bedrock",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock"
)]
trait Bedrock {
    #[dbus_proxy(property, name = "Volume")]
    fn audio_volume(&self) -> zbus::Result<f64>;
    #[dbus_proxy(property, name = "Volume")]
    fn set_audio_volume(&self, value: f64) -> zbus::Result<()>;
}

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Audio e Suoni (Bedrock DBus)")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();
    container.append(&title);

    let scale = Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 1.0, 0.05);
    scale.set_value(0.5); // Initial placeholder, loaded via D-Bus immediately below

    let scale_clone = scale.clone();
    let ctx = gtk4::glib::MainContext::default();
    ctx.spawn_local(async move {
        if let Ok(conn) = crate::get_connection().await {
            if let Ok(proxy) = BedrockProxy::new(&conn).await {
                if let Ok(vol) = proxy.audio_volume().await {
                    scale_clone.set_value(vol);
                }
            }
        }
    });

    scale.connect_value_changed(move |s| {
        let val = s.value();
        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = BedrockProxy::new(&conn).await {
                    let _ = proxy.set_audio_volume(val).await;
                }
            }
        });
    });

    container.append(&scale);
    container
}

