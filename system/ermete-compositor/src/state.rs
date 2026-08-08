use crate::backend::render::{KawaseBlurConfig, KawaseBlurPipeline};
use crate::backend::DrmKmsBackend;
use crate::desktop_state::DesktopState;
use crate::ipc::protocol::{AiLayoutCommand, CompositorStatus, IpcResponse};
use tracing::info;

pub struct CompositorState {
    pub desktop_state: DesktopState,
    #[allow(dead_code)]
    pub blur_pipeline: KawaseBlurPipeline,
    #[allow(dead_code)]
    pub is_running: bool,
}

impl CompositorState {
    pub fn new(drm_backend: DrmKmsBackend) -> Self {
        Self {
            desktop_state: DesktopState::new(drm_backend),
            blur_pipeline: KawaseBlurPipeline::new(KawaseBlurConfig::default()),
            is_running: true,
        }
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
        }
    }

    pub fn process_command(&mut self, cmd: AiLayoutCommand) -> IpcResponse {
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
        }
    }
}
