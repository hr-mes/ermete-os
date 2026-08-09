use crate::backend::DrmKmsBackend;
use crate::ipc::protocol::WindowPlacement;
use crate::tiling::{
    OverviewWorkspaceCard, ScreenGeometry, SnapEngine, SpatialOverview, TilingEngine,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Corner docking positions for Picture-in-Picture surfaces matching ermete_pip_v1 protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum PipCorner {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
    Floating = 4,
}

impl PipCorner {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            _ => Self::Floating,
        }
    }
}

/// Bitmask flags for Picture-in-Picture surface behavior.
pub struct PipFlag;
impl PipFlag {
    pub const ALWAYS_ON_TOP: u32 = 1;
    pub const STICKY_WORKSPACES: u32 = 2;
    pub const SNAP_TO_CORNERS: u32 = 4;
    pub const ENABLE_KAWASE_BLUR: u32 = 8;
}

/// State and geometry of a Picture-in-Picture surface enclave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipSurface {
    pub surface_id: u64,
    pub title: String,
    pub app_id: String,
    pub corner: PipCorner,
    pub margin_x: u32,
    pub margin_y: u32,
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: Option<(u32, u32)>,
    pub flags: u32,
    pub opacity: u8,
    pub workspace: u32,
    pub geometry: WindowPlacement,
}

/// Layer manager handling Picture-in-Picture always-on-top and sticky workspace overlays.
pub struct PipLayerManager {
    pip_surfaces: HashMap<u64, PipSurface>,
}

impl Default for PipLayerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PipLayerManager {
    pub fn new() -> Self {
        Self {
            pip_surfaces: HashMap::new(),
        }
    }

    /// Calculates absolute output coordinates for a PiP surface based on corner dock, margins, and screen bounds.
    pub fn calculate_pip_geometry(
        corner: PipCorner,
        margin_x: u32,
        margin_y: u32,
        width: u32,
        mut height: u32,
        aspect_ratio: Option<(u32, u32)>,
        screen: &ScreenGeometry,
        workspace: u32,
        surface_id: u64,
    ) -> WindowPlacement {
        // Enforce aspect ratio constraint if specified
        if let Some((num, den)) = aspect_ratio {
            if num > 0 && den > 0 {
                height = ((width as f64 * den as f64) / num as f64) as u32;
            }
        }

        let screen_w = screen.width;
        let screen_h = screen.height;
        let mx = margin_x as i32;
        let my = margin_y as i32;
        let w = width.min(screen_w) as i32;
        let h = height.min(screen_h) as i32;

        let (x, y) = match corner {
            PipCorner::TopLeft => (mx, my),
            PipCorner::TopRight => (screen_w as i32 - w - mx, my),
            PipCorner::BottomLeft => (mx, screen_h as i32 - h - my),
            PipCorner::BottomRight => (screen_w as i32 - w - mx, screen_h as i32 - h - my),
            PipCorner::Floating => (mx.max(0), my.max(0)),
        };

        WindowPlacement {
            window_id: surface_id,
            x,
            y,
            width: w as u32,
            height: h as u32,
            workspace,
        }
    }

    /// Registers or updates a Picture-in-Picture surface.
    pub fn register_pip_surface(
        &mut self,
        surface_id: u64,
        title: String,
        app_id: String,
        corner: PipCorner,
        margin_x: u32,
        margin_y: u32,
        width: u32,
        height: u32,
        aspect_ratio: Option<(u32, u32)>,
        flags: u32,
        opacity: u8,
        screen: &ScreenGeometry,
        workspace: u32,
    ) -> WindowPlacement {
        let placement = Self::calculate_pip_geometry(
            corner,
            margin_x,
            margin_y,
            width,
            height,
            aspect_ratio,
            screen,
            workspace,
            surface_id,
        );

        let pip = PipSurface {
            surface_id,
            title,
            app_id,
            corner,
            margin_x,
            margin_y,
            width: placement.width,
            height: placement.height,
            aspect_ratio,
            flags,
            opacity,
            workspace,
            geometry: placement.clone(),
        };

        info!(
            "Registered PiP surface {} ('{}') docked to {:?} (always_on_top: {}, sticky: {})",
            surface_id,
            pip.title,
            corner,
            (flags & PipFlag::ALWAYS_ON_TOP) != 0,
            (flags & PipFlag::STICKY_WORKSPACES) != 0
        );

        self.pip_surfaces.insert(surface_id, pip);
        placement
    }

    /// Removes a PiP surface enclave.
    pub fn remove_pip_surface(&mut self, surface_id: u64) -> Option<PipSurface> {
        if let Some(pip) = self.pip_surfaces.remove(&surface_id) {
            info!("Removed PiP surface {}", surface_id);
            Some(pip)
        } else {
            None
        }
    }

    /// Returns visible PiP surfaces for the specified workspace (including sticky surfaces).
    pub fn get_visible_pip_surfaces(&self, active_workspace: u32) -> Vec<PipSurface> {
        self.pip_surfaces
            .values()
            .filter(|pip| {
                let is_sticky = (pip.flags & PipFlag::STICKY_WORKSPACES) != 0;
                is_sticky || pip.workspace == active_workspace
            })
            .cloned()
            .collect()
    }

    pub fn get_pip_surface(&self, surface_id: u64) -> Option<&PipSurface> {
        self.pip_surfaces.get(&surface_id)
    }

    pub fn all_pip_surfaces(&self) -> Vec<&PipSurface> {
        self.pip_surfaces.values().collect()
    }
}

/// Render layout payload returned by DesktopState for composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopRenderLayout {
    pub active_workspace: u32,
    pub tiled_windows: Vec<WindowPlacement>,
    pub overview_active: bool,
    pub overview_workspace_cards: Vec<OverviewWorkspaceCard>,
    pub pip_overlays: Vec<PipSurface>,
}

use crate::backend::shm::{CompositorShmState, ShmBufferLimits};
use crate::input_routing::InputRouter;
use crate::screencopy::ScreencopyManager;

/// Main Smithay Desktop State managing tiled layout engine, snap engine, spatial overview, PiP layer, screencopy manager, and input router.
pub struct DesktopState {
    pub drm_backend: DrmKmsBackend,
    pub shm_state: CompositorShmState,
    pub tiling_engine: TilingEngine,
    pub snap_engine: SnapEngine,
    pub overview: SpatialOverview,
    pub pip_manager: PipLayerManager,
    pub screencopy_manager: ScreencopyManager,
    pub input_router: InputRouter,
    pub screen: ScreenGeometry,
}

impl DesktopState {
    pub fn new(drm_backend: DrmKmsBackend) -> Self {
        Self {
            drm_backend,
            shm_state: CompositorShmState::with_default_limits(),
            tiling_engine: TilingEngine::new(),
            snap_engine: SnapEngine::new(),
            overview: SpatialOverview::new(),
            pip_manager: PipLayerManager::new(),
            screencopy_manager: ScreencopyManager::new(),
            input_router: InputRouter::new(),
            screen: ScreenGeometry::default(),
        }
    }


    pub fn set_screen_geometry(&mut self, width: u32, height: u32) {
        self.screen = ScreenGeometry { width, height };
    }

    /// Returns complete layered layout for current workspace rendering.
    pub fn compute_render_layout(&self) -> DesktopRenderLayout {
        let active_ws = self.tiling_engine.active_workspace();
        let windows = self.tiling_engine.windows();

        let tiled_windows: Vec<WindowPlacement> = windows
            .iter()
            .map(|w| {
                if let Some(snapped) = self.snap_engine.get_snap_state(w.id) {
                    snapped.snapped_geometry.clone()
                } else {
                    w.geometry.clone()
                }
            })
            .collect();

        let pip_overlays = self.pip_manager.get_visible_pip_surfaces(active_ws);

        DesktopRenderLayout {
            active_workspace: active_ws,
            tiled_windows,
            overview_active: self.overview.is_active(),
            overview_workspace_cards: self.overview.get_workspace_cards().to_vec(),
            pip_overlays,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pip_layer_manager_docking_and_sticky_layer() {
        let mut manager = PipLayerManager::new();
        let screen = ScreenGeometry { width: 1920, height: 1080 };

        // Test bottom-right corner dock
        let p1 = manager.register_pip_surface(
            1,
            "Video Player".into(),
            "mpv".into(),
            PipCorner::BottomRight,
            16,
            16,
            400,
            225,
            Some((16, 9)),
            PipFlag::ALWAYS_ON_TOP | PipFlag::STICKY_WORKSPACES,
            255,
            &screen,
            1,
        );

        assert_eq!(p1.width, 400);
        assert_eq!(p1.height, 225);
        assert_eq!(p1.x, 1920 - 400 - 16);
        assert_eq!(p1.y, 1080 - 225 - 16);

        // Verify sticky layer visible on workspace 2
        let visible_ws2 = manager.get_visible_pip_surfaces(2);
        assert_eq!(visible_ws2.len(), 1);
        assert_eq!(visible_ws2[0].surface_id, 1);
    }
}
