use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{ControllerBackend, SystemEventBus, SystemEvent, MockState, BedrockAudioProxy};

#[derive(Clone, Debug)]
pub struct AudioController {
    backend: ControllerBackend,
    cached_volume: Arc<Mutex<f64>>,
    event_bus: SystemEventBus,
}

impl AudioController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self {
            backend,
            cached_volume: Arc::new(Mutex::new(0.5)),
            event_bus,
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            cached_volume: Arc::new(Mutex::new(0.5)),
            event_bus,
        }
    }

    pub async fn toggle_mute(&self) -> zbus::Result<bool> {
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

    pub async fn toggle_source_mute(&self) -> zbus::Result<bool> {
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

    pub async fn set_volume(&self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    proxy.set_volume(volume).await?;
                    if let Ok(mut c) = self.cached_volume.lock() {
                        *c = volume;
                    }
                }
            }
            ControllerBackend::Mock(state) => {
                if let Ok(mut c) = self.cached_volume.lock() {
                    *c = volume;
                }
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.volume = volume;
            }
        }
        self.event_bus.emit(SystemEvent::VolumeChanged(volume));
        Ok(())
    }

    pub async fn set_source_volume(&self, volume: f64) -> zbus::Result<()> {
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

    pub async fn get_volume(&self) -> zbus::Result<f64> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await {
                    if let Ok(vol) = proxy.volume().await {
                        if let Ok(mut c) = self.cached_volume.lock() {
                            *c = vol;
                        }
                        return Ok(vol);
                    }
                }
                Ok(*self.cached_volume.lock().unwrap_or_else(|e| e.into_inner()))
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).volume),
        }
    }

    pub fn get_cached_volume(&self) -> f64 {
        *self.cached_volume.lock().unwrap_or_else(|e| e.into_inner())
    }
}
