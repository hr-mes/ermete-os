use zbus::proxy;
use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{ControllerBackend, SystemEventBus, SystemEvent, MockState};

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait MprisPlayer {
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn play_pause(&self) -> zbus::Result<()>;
    fn play(&self) -> zbus::Result<()>;
    fn pause(&self) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;
}

#[derive(Clone, Debug)]
pub struct MprisController {
    backend: ControllerBackend,
    cached_mpris: Arc<Mutex<Option<crate::core::mpris::MprisState>>>,
    event_bus: SystemEventBus,
}

impl MprisController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self {
            backend,
            cached_mpris: Arc::new(Mutex::new(None)),
            event_bus,
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            cached_mpris: Arc::new(Mutex::new(None)),
            event_bus,
        }
    }

    pub async fn player_command(&self, cmd: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(dbus) = zbus::fdo::DBusProxy::new(session).await {
                    if let Ok(names) = dbus.list_names().await {
                        for name in names {
                            if name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                if let Ok(player) = MprisPlayerProxy::builder(session)
                                    .destination(name.as_str())?
                                    .path("/org/mpris/MediaPlayer2")?
                                    .build().await
                                {
                                    match cmd {
                                        "play-pause" => { let _ = player.play_pause().await; }
                                        "next" => { let _ = player.next().await; }
                                        "previous" => { let _ = player.previous().await; }
                                        "play" => { let _ = player.play().await; }
                                        "pause" => { let _ = player.pause().await; }
                                        _ => {}
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = self.refresh_mpris().await;
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command = Some(cmd.to_string());
                let _ = self.refresh_mpris().await;
            }
        }
        self.event_bus.emit(SystemEvent::MprisUpdated(self.get_cached_mpris_state()));
        Ok(())
    }

    pub fn get_cached_mpris_state(&self) -> Option<crate::core::mpris::MprisState> {
        self.cached_mpris.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_last_player_command(&self) -> Option<String> {
        match &self.backend {
            ControllerBackend::Dbus { .. } => None,
            ControllerBackend::Mock(state) => state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command.clone(),
        }
    }

    pub async fn refresh_mpris(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(dbus_proxy) = zbus::fdo::DBusProxy::new(session).await {
                    if let Ok(names) = dbus_proxy.list_names().await {
                        for name in names {
                            if name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                if let Ok(props_proxy) = zbus::fdo::PropertiesProxy::builder(session)
                                    .destination(name.as_str())?
                                    .path("/org/mpris/MediaPlayer2")?
                                    .build().await
                                {
                                    let iface: zbus::names::InterfaceName = "org.mpris.MediaPlayer2.Player".try_into().unwrap_or_else(|_| "org.mpris.MediaPlayer2.Player".try_into().unwrap());
                                    let status = props_proxy.get(iface.clone(), "PlaybackStatus").await
                                        .ok()
                                        .and_then(|v| match &*v {
                                            zbus::zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                                            _ => None,
                                        })
                                        .unwrap_or_else(|| "Stopped".to_string());
                                    let title = props_proxy.get(iface.clone(), "Metadata").await
                                        .ok()
                                        .and_then(|v| {
                                            if let zbus::zvariant::Value::Dict(dict) = &*v {
                                                if let Ok(Some(val)) = dict.get(&zbus::zvariant::Value::from("xesam:title")) {
                                                    if let zbus::zvariant::Value::Str(s) = val {
                                                        return Some(s.as_str().to_string());
                                                    }
                                                }
                                            }
                                            None
                                        }).unwrap_or_else(|| "Sconosciuto".to_string());
                                    let artist = props_proxy.get(iface.clone(), "Metadata").await
                                        .ok()
                                        .and_then(|v| {
                                            if let zbus::zvariant::Value::Dict(dict) = &*v {
                                                if let Ok(Some(val)) = dict.get(&zbus::zvariant::Value::from("xesam:artist")) {
                                                    if let zbus::zvariant::Value::Array(arr) = val {
                                                        if let Ok(Some(first)) = arr.get(0) {
                                                            if let zbus::zvariant::Value::Str(s) = first {
                                                                return Some(s.as_str().to_string());
                                                            }
                                                        }
                                                    } else if let zbus::zvariant::Value::Str(s) = val {
                                                        return Some(s.as_str().to_string());
                                                    }
                                                }
                                            }
                                            None
                                        }).unwrap_or_else(|| "-".to_string());
                                    let new_state = crate::core::mpris::MprisState {
                                        title,
                                        artist,
                                        status,
                                    };
                                    if let Ok(mut lock) = self.cached_mpris.lock() {
                                        *lock = Some(new_state);
                                    }
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                if let Ok(mut lock) = self.cached_mpris.lock() {
                    *lock = None;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => {
                if let Ok(mut lock) = self.cached_mpris.lock() {
                    if lock.is_none() {
                        *lock = Some(crate::core::mpris::MprisState {
                            title: "Track Title".to_string(),
                            artist: "Artist".to_string(),
                            status: "Playing".to_string(),
                        });
                    }
                }
                Ok(())
            }
        }
    }
}
