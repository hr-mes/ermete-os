use crate::backend::render::{KawaseBlurConfig, KawaseBlurPipeline};
use crate::backend::DrmKmsBackend;
use crate::dbus_listener::{spawn_dbus_appearance_listener, AppearanceSettings};
use crate::desktop_state::DesktopState;
use crate::ipc::protocol::{AiLayoutCommand, CompositorStatus, IpcResponse};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

pub struct CompositorState {
    pub desktop_state: DesktopState,
    #[allow(dead_code)]
    pub blur_pipeline: KawaseBlurPipeline,
    #[allow(dead_code)]
    pub is_running: bool,
    pub appearance_dirty: Arc<AtomicBool>,
    pub appearance_rx: watch::Receiver<AppearanceSettings>,
    pub current_appearance: AppearanceSettings,
    #[allow(dead_code)]
    dbus_listener_handle: tokio::task::JoinHandle<()>,
}

impl CompositorState {
    pub fn new(drm_backend: DrmKmsBackend) -> Self {
        let appearance_dirty = Arc::new(AtomicBool::new(false));
        let (tx, rx) = watch::channel(AppearanceSettings::default());

        let listener_handle = spawn_dbus_appearance_listener(Arc::clone(&appearance_dirty), tx);

        Self {
            desktop_state: DesktopState::new(drm_backend),
            blur_pipeline: KawaseBlurPipeline::new(KawaseBlurConfig::default()),
            is_running: true,
            appearance_dirty,
            appearance_rx: rx,
            current_appearance: AppearanceSettings::default(),
            dbus_listener_handle: listener_handle,
        }
    }

    pub fn apply_pending_appearance(&mut self) {
        let updated = self.appearance_rx.borrow().clone();
        info!(
            "Applying pre-parsed DBus appearance update without blocking 1000Hz loop: color_scheme={}, accent={}",
            updated.color_scheme, updated.accent_color
        );
        self.current_appearance = updated;
    }

    pub fn tick_animation(&mut self, dt: f64) {
        self.desktop_state.tiling_engine.tick_animation(dt);
    }

    pub fn status(&self) -> CompositorStatus {
        let (inner, outer) = self.desktop_state.tiling_engine.gaps();
        CompositorStatus {
            active_mode: self.desktop_state.tiling_engine.mode(),
            window_count: self.desktop_state.tiling_engine.window_count(),
            active_workspace: self.desktop_state.tiling_engine.active_workspace(),
            drm_kms_active: !self.desktop_state.drm_backend.is_headless(),
            windows: self.desktop_state.tiling_engine.windows(),
            inner_gap: inner,
            outer_gap: outer,
            active_screencopy_frames: self.desktop_state.screencopy_manager.active_frame_count(),
            active_input_grabs: self.desktop_state.input_router.active_grab_count(),
        }
    }

    pub async fn process_command(&mut self, cmd: AiLayoutCommand) -> IpcResponse {
        match cmd {
            AiLayoutCommand::Ping => {
                IpcResponse::success("PONG", Some(self.status()))
            }
            AiLayoutCommand::QueryState => {
                IpcResponse::success("Compositor state queried", Some(self.status()))
            }
            AiLayoutCommand::SetLayoutMode { mode } => {
                self.desktop_state.tiling_engine.set_mode(mode);
                IpcResponse::success(
                    format!("Layout mode set to {}", mode),
                    Some(self.status()),
                )
            }
            AiLayoutCommand::SetGaps { inner, outer } => {
                self.desktop_state.tiling_engine.set_gaps(inner, outer);
                IpcResponse::success(
                    format!("Gaps updated: inner={}, outer={}", inner, outer),
                    Some(self.status()),
                )
            }
            AiLayoutCommand::FocusWindow { window_id } => {
                if self.desktop_state.tiling_engine.focus_window(window_id) {
                    self.desktop_state.input_router.set_focused_surface(Some(window_id));
                    IpcResponse::success(
                        format!("Focused window {}", window_id),
                        Some(self.status()),
                    )
                } else {
                    IpcResponse::error(format!("Window {} not found", window_id))
                }
            }
            AiLayoutCommand::ApplyAiTileMap { window_placements } => {
                info!("Received AI auto-tiling instructions for {} windows", window_placements.len());
                self.desktop_state.tiling_engine.apply_ai_placements(window_placements);
                IpcResponse::success(
                    "Applied AI-driven window placements",
                    Some(self.status()),
                )
            }
            AiLayoutCommand::RequestScreenCapture { app_id, pid, output_id } => {
                match self.desktop_state.screencopy_manager.request_capture_output(&app_id, pid, output_id, true, None).await {
                    Ok(frame_id) => IpcResponse::success(
                        format!("Screen capture request (Frame #{}) AUTHORIZED by Gatekeeper for '{}'", frame_id, app_id),
                        Some(self.status()),
                    ),
                    Err(err) => IpcResponse::error(format!("Screen capture request DENIED: {}", err)),
                }
            }
            AiLayoutCommand::RequestGlobalInputGrab { app_id, pid } => {
                match self.desktop_state.input_router.request_global_input_grab(&app_id, pid).await {
                    Ok(grab_id) => IpcResponse::success(
                        format!("Global input grab #{ } AUTHORIZED by Gatekeeper for '{}'", grab_id, app_id),
                        Some(self.status()),
                    ),
                    Err(err) => IpcResponse::error(format!("Global input grab DENIED: {}", err)),
                }
            }
        }
    }
}
