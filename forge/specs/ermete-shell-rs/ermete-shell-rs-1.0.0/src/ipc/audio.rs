use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{IpcBackend, AudioBus, AudioEvent,  BedrockAudioProxy};

pub enum AudioCommand {
    ToggleMute(oneshot::Sender<zbus::Result<bool>>),
    ToggleSourceMute(oneshot::Sender<zbus::Result<bool>>),
    SetVolume(f64, oneshot::Sender<zbus::Result<()>>),
    SetSourceVolume(f64, oneshot::Sender<zbus::Result<()>>),
}

pub struct AudioActor {
    backend: IpcBackend,
    cached_volume: f64,
    event_bus: AudioBus,
    receiver: mpsc::Receiver<AudioCommand>,
}

impl AudioActor {
    pub fn spawn(backend: IpcBackend, event_bus: AudioBus) -> mpsc::Sender<AudioCommand> {
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
            IpcBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    let current = proxy.muted().await.map_err(|e| zbus::Error::Failure(e.to_string()))?;
                    let new_st = !current;
                    proxy.set_muted(new_st).await?;
                    new_st
                } else {
                    true
                }
            }
            IpcBackend::Disconnected => return Err(zbus::Error::Failure("Audio service offline".into())),
        };
        self.event_bus.emit(AudioEvent::MuteToggled(new_state));
        Ok(new_state)
    }

    async fn handle_toggle_source_mute(&mut self) -> zbus::Result<bool> {
        match &self.backend {
            IpcBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    let current = proxy.source_muted().await.map_err(|e| zbus::Error::Failure(e.to_string()))?;
                    let new_state = !current;
                    proxy.set_source_muted(new_state).await?;
                    return Ok(new_state);
                }
                Err(zbus::Error::Failure("Audio service offline".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("Audio service offline".into())),
        }
    }

    async fn handle_set_volume(&mut self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    proxy.set_volume(volume).await?;
                    self.cached_volume = volume;
                }
            }
            IpcBackend::Disconnected => {}
        }
        self.event_bus.emit(AudioEvent::VolumeChanged(volume));
        Ok(())
    }

    async fn handle_set_source_volume(&mut self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    proxy.set_source_volume(volume).await?;
                }
                Ok(())
            }
            IpcBackend::Disconnected => Ok(()),
        }
    }

}

#[derive(Clone, Debug)]
pub struct AudioController {
    sender: mpsc::Sender<AudioCommand>,
    cached_volume: Arc<Mutex<f64>>,
}

impl AudioController {
    pub fn new(backend: IpcBackend, event_bus: AudioBus) -> Self {
        let sender = AudioActor::spawn(backend, event_bus);
        Self {
            sender,
            cached_volume: Arc::new(Mutex::new(0.5)),
        }
    }


    pub async fn toggle_mute(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::ToggleMute(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }

    pub async fn toggle_source_mute(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::ToggleSourceMute(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }

    pub async fn set_volume(&self, volume: f64) -> zbus::Result<()> {
        let mut c = self.cached_volume.lock().await;
        {
            *c = volume;
        }
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::SetVolume(volume, tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }

    pub async fn set_source_volume(&self, volume: f64) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(AudioCommand::SetSourceVolume(volume, tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }

    pub fn get_cached_volume(&self) -> f64 {
        *self.cached_volume.blocking_lock()
    }
}

impl crate::ipc::system_proxies::ControllerBackend for AudioController {
    fn name(&self) -> &'static str {
        "audio"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_audio_controller() -> AudioController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<AudioController>("audio") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_audio_bus();
        AudioController::new(IpcBackend::Disconnected, bus)
    }
}



