use crate::backend::DrmKmsBackend;
use crate::ipc::protocol::{AiLayoutCommand, CompositorStatus, IpcResponse};
use crate::tiling::TilingEngine;
use tracing::info;

pub struct CompositorState {
    pub tiling_engine: TilingEngine,
    pub drm_backend: DrmKmsBackend,
    #[allow(dead_code)]
    pub is_running: bool,
}

impl CompositorState {
    pub fn new(drm_backend: DrmKmsBackend) -> Self {
        Self {
            tiling_engine: TilingEngine::new(),
            drm_backend,
            is_running: true,
        }
    }

    pub fn status(&self) -> CompositorStatus {
        let (inner, outer) = self.tiling_engine.gaps();
        CompositorStatus {
            active_mode: self.tiling_engine.mode(),
            window_count: self.tiling_engine.window_count(),
            active_workspace: self.tiling_engine.active_workspace(),
            drm_kms_active: !self.drm_backend.is_headless(),
            windows: self.tiling_engine.windows(),
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
                self.tiling_engine.set_mode(mode);
                IpcResponse::success(
                    format!("Layout mode set to {}", mode),
                    Some(self.status()),
                )
            }
            AiLayoutCommand::SetGaps { inner, outer } => {
                self.tiling_engine.set_gaps(inner, outer);
                IpcResponse::success(
                    format!("Gaps updated: inner={}, outer={}", inner, outer),
                    Some(self.status()),
                )
            }
            AiLayoutCommand::FocusWindow { window_id } => {
                if self.tiling_engine.focus_window(window_id) {
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
                self.tiling_engine.apply_ai_placements(window_placements);
                IpcResponse::success(
                    "Applied AI-driven window placements",
                    Some(self.status()),
                )
            }
        }
    }
}
