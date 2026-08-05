use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::ipc::system_proxies::{ControllerBackend, SystemEventBus, SystemEvent, MockState, BedrockAudioProxy};

pub enum AudioCommand {
    ToggleMute(oneshot::Sender<zbus::Result<bool>>),
    ToggleSourceMute(oneshot::Sender<zbus::Result<bool>>),
    SetVolume(f64, oneshot::Sender<zbus::Result<()>>),
    SetSourceVolume(f64, oneshot::Sender<zbus::Result<()>>),
}

pub struct AudioActor {
    backend: ControllerBackend,
    cached_volume: f64,
    event_bus: SystemEventBus,
    receiver: mpsc::Receiver<AudioCommand>,
}

impl AudioActor {
    pub fn spawn(backend: ControllerBackend, event_bus: SystemEventBus) -> mpsc::Sender<AudioCommand> {
        let (tx, rx) = mpsc::channel(32);
        let actor = Self {
            backend,
            cached_volume: 0.5,
            event_bus,
            receiver: rx,
        };
        tokio::spawn(actor.run());
        tx
    }

    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                AudioCommand::ToggleMute(resp) => {
                    let res = self.handle_toggle_mute().await;
                    let _ = resp.send(res);
                }
                AudioCommand::ToggleSourceMute(resp) => {
                    let res = self.handle_toggle_source_mute().await;
                    let _ = resp.send(res);
                }
                AudioCommand::SetVolume(vol, resp) => {
                    let res = self.handle_set_volume(vol).await;
                    let _ = resp.send(res);
                }
                AudioCommand::SetSourceVolume(vol, resp) => {
                    let res = self.handle_set_source_volume(vol).await;
                    let _ = resp.send(res);
                }
            }
        }
    }

    async fn handle_toggle_mute(&mut self) -> zbus::Result<bool> {
        let new_state = match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    let current = proxy.muted().await.unwrap_or(false);
                    let new_st = !current;
                    proxy.set_muted(new_st).await?;
                    new_st
                } else {
                    true
                }
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.mute = !s.mute;
                s.mute
            }
        };
        self.event_bus.emit(SystemEvent::MuteToggled(new_state));
        Ok(new_state)
    }

    async fn handle_toggle_source_mute(&mut self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    let current = proxy.source_muted().await.unwrap_or(false);
                    let new_state = !current;
                    proxy.set_source_muted(new_state).await?;
                    return Ok(new_state);
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.source_mute = !s.source_mute;
                Ok(s.source_mute)
            }
        }
    }

    async fn handle_set_volume(&mut self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    proxy.set_volume(volume).await?;
                    self.cached_volume = volume;
                }
            }
            ControllerBackend::Mock(state) => {
                self.cached_volume = volume;
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.volume = volume;
            }
        }
        self.event_bus.emit(SystemEvent::VolumeChanged(volume));
        Ok(())
    }

    async fn handle_set_source_volume(&mut self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    proxy.set_source_volume(volume).await?;
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.source_volume = volume;
                Ok(())
            }
        }
    }

}

#[derive(Clone, Debug)]
pub struct AudioController {
    sender: mpsc::Sender<AudioCommand>,
    cached_volume: Arc<Mutex<f64>>,
}

impl AudioController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        let sender = AudioActor::spawn(backend, event_bus);
        Self {
            sender,
            cached_volume: Arc::new(Mutex::new(0.5)),
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        let backend = ControllerBackend::Mock(state);
        let sender = AudioActor::spawn(backend, event_bus);
        Self {
            sender,
            cached_volume: Arc::new(Mutex::new(0.5)),
        }
    }

    pub async fn toggle_mute(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::ToggleMute(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(false))
        } else {
            Ok(false)
        }
    }

    pub async fn toggle_source_mute(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::ToggleSourceMute(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(false))
        } else {
            Ok(false)
        }
    }

    pub async fn set_volume(&self, volume: f64) -> zbus::Result<()> {
        if let Ok(mut c) = self.cached_volume.lock() {
            *c = volume;
        }
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::SetVolume(volume, tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn set_source_volume(&self, volume: f64) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::SetSourceVolume(volume, tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub fn get_cached_volume(&self) -> f64 {
        *self.cached_volume.lock().unwrap_or_else(|e| e.into_inner())
    }
}
