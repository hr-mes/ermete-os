use zbus::proxy;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{IpcBackend, MprisBus, MprisEvent};
pub use crate::ipc::types::MprisState;

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

pub enum MprisCommand {
    PlayerCommand(String, oneshot::Sender<zbus::Result<()>>),
    GetCachedMprisState(oneshot::Sender<Option<MprisState>>),
}

pub struct MprisActor {
    backend: IpcBackend,
    cached_mpris: Option<MprisState>,
    event_bus: MprisBus,
    receiver: mpsc::Receiver<MprisCommand>,
}

impl MprisActor {
    pub fn spawn(backend: IpcBackend, event_bus: MprisBus) -> mpsc::Sender<MprisCommand> {
        let (tx, rx) = mpsc::channel(32);
        let actor = Self {
            backend,
            cached_mpris: None,
            event_bus,
            receiver: rx,
        };
        tokio::spawn(actor.run());
        tx
    }

    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                MprisCommand::PlayerCommand(c, resp) => {
                    let res = self.handle_player_command(&c).await;
                    let _ = resp.send(res);
                }
                MprisCommand::GetCachedMprisState(resp) => {
                    let _ = resp.send(self.cached_mpris.clone());
                }
            }
        }
    }

    async fn handle_player_command(&mut self, cmd: &str) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { session, .. } => {
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
                let _ = self.handle_refresh_mpris().await;
            }
            IpcBackend::Disconnected => {}
        }
        self.event_bus.emit(MprisEvent::MprisUpdated(self.cached_mpris.clone()));
        Ok(())
    }

    async fn handle_refresh_mpris(&mut self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { session, .. } => {
                if let Ok(dbus_proxy) = zbus::fdo::DBusProxy::new(session).await {
                    if let Ok(names) = dbus_proxy.list_names().await {
                        for name in names {
                            if name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                if let Ok(props_proxy) = zbus::fdo::PropertiesProxy::builder(session)
                                    .destination(name.as_str())?
                                    .path("/org/mpris/MediaPlayer2")?
                                    .build().await
                                {
                                    if let Ok(iface) = zbus::names::InterfaceName::try_from("org.mpris.MediaPlayer2.Player") {
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
                                                    if let Ok(Some(zbus::zvariant::Value::Str(s))) = dict.get(&zbus::zvariant::Value::from("xesam:title")) {
                                                        return Some(s.as_str().to_string());
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
                                                            if let Ok(Some(zbus::zvariant::Value::Str(s))) = arr.get(0) {
                                                                return Some(s.as_str().to_string());
                                                            }
                                                        } else if let zbus::zvariant::Value::Str(s) = val {
                                                            return Some(s.as_str().to_string());
                                                        }
                                                    }
                                                }
                                                None
                                            }).unwrap_or_else(|| "-".to_string());
                                        let new_state = MprisState {
                                            title,
                                            artist,
                                            status,
                                        };
                                        self.cached_mpris = Some(new_state);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                self.cached_mpris = None;
                Ok(())
            }
            IpcBackend::Disconnected => {
                self.cached_mpris = None;
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MprisController {
    sender: mpsc::Sender<MprisCommand>,
    cached_mpris: Arc<Mutex<Option<MprisState>>>,
    last_player_command: Arc<Mutex<Option<String>>>,
}

impl MprisController {
    pub fn new(backend: IpcBackend, event_bus: MprisBus) -> Self {
        let sender = MprisActor::spawn(backend, event_bus);
        Self {
            sender,
            cached_mpris: Arc::new(Mutex::new(None)),
            last_player_command: Arc::new(Mutex::new(None)),
        }
    }


    pub async fn player_command(&self, cmd: &str) -> zbus::Result<()> {
        let mut lock = self.last_player_command.lock().await;
        {
            *lock = Some(cmd.to_string());
        }
        let (tx, rx) = oneshot::channel();
        if self.sender.send(MprisCommand::PlayerCommand(cmd.to_string(), tx)).await.is_ok() {
            let res = rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?;
            let _ = self.refresh_mpris().await;
            res
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }

    pub fn get_cached_mpris_state(&self) -> Option<MprisState> {
        self.cached_mpris.blocking_lock().clone()
    }

    pub async fn refresh_mpris(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(MprisCommand::GetCachedMprisState(tx)).await.is_ok() {
            if let Ok(state) = rx.await {
                let mut lock = self.cached_mpris.lock().await;
        {
                    *lock = state;
                }
            }
        }
        Ok(())
    }
}

impl crate::ipc::system_proxies::ControllerBackend for MprisController {
    fn name(&self) -> &'static str {
        "mpris"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_mpris_controller() -> MprisController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<MprisController>("mpris") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_mpris_bus();
        MprisController::new(IpcBackend::Disconnected, bus)
    }
}



