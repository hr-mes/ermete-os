use zbus::{interface, Connection};
use tracing::{info, error};
use std::collections::HashMap;
use tokio::process::Command;

pub struct ErmetePortal;

pub struct ScreenCastPortal;
pub struct CameraPortal;
pub struct LocationPortal;
pub struct MicrophonePortal;
pub struct FileChooserPortal;

impl ErmetePortal {
    async fn request_permission(resource: &str, app_id: &str) -> bool {
        info!("Prompting user for {} permission for app: {}", resource, app_id);
        
        let status = Command::new("ermete-shell-rs")
            .arg("--privacy-prompt")
            .arg(format!("{}:{}", resource, app_id))
            .status();

        match status.await {
            Ok(exit_status) => {
                let granted = exit_status.success();
                if granted {
                    info!("Permission GRANTED for {}.", resource);
                    let res = resource.to_string();
                    tokio::spawn(async move {
                        if let Ok(conn) = Connection::session().await {
                            let _ = conn.call_method(
                                Some("os.ermete.Shell"),
                                "/os/ermete/Shell",
                                Some("os.ermete.Shell"),
                                "SetPrivacyIndicator",
                                &(res, true),
                            ).await;
                        }
                    });
                } else {
                    info!("Permission DENIED for {}.", resource);
                }
                granted
            }
            Err(e) => {
                error!("Failed to launch privacy prompt: {}", e);
                false
            }
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.ScreenCast")]
impl ScreenCastPortal {
    #[zbus(name = "CreateSession")]
    async fn create_session(&self, _handle: String, _session_handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if ErmetePortal::request_permission("ScreenCast", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Camera")]
impl CameraPortal {
    async fn access_camera(&self, _handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if ErmetePortal::request_permission("Camera", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Location")]
impl LocationPortal {
    #[zbus(name = "CreateSession")]
    async fn create_session(&self, _handle: String, _session_handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if ErmetePortal::request_permission("Location", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Microphone")]
impl MicrophonePortal {
    async fn access_microphone(&self, _handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if ErmetePortal::request_permission("Microphone", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserPortal {
    #[zbus(name = "OpenFile")]
    async fn open_file(&self, _handle: String, app_id: String, _parent_window: String, _title: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<(u32, HashMap<String, zbus::zvariant::Value<'_>>), zbus::fdo::Error> {
        if ErmetePortal::request_permission("FileChooser", &app_id).await {
            Ok((0, HashMap::new()))
        } else {
            Ok((2, HashMap::new()))
        }
    }

    #[zbus(name = "SaveFile")]
    async fn save_file(&self, _handle: String, app_id: String, _parent_window: String, _title: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<(u32, HashMap<String, zbus::zvariant::Value<'_>>), zbus::fdo::Error> {
        if ErmetePortal::request_permission("FileChooser", &app_id).await {
            Ok((0, HashMap::new()))
        } else {
            Ok((2, HashMap::new()))
        }
    }

    #[zbus(name = "SaveFiles")]
    async fn save_files(&self, _handle: String, app_id: String, _parent_window: String, _title: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<(u32, HashMap<String, zbus::zvariant::Value<'_>>), zbus::fdo::Error> {
        if ErmetePortal::request_permission("FileChooser", &app_id).await {
            Ok((0, HashMap::new()))
        } else {
            Ok((2, HashMap::new()))
        }
    }
}
