use crate::ipc::protocol::WindowPlacement;
use crate::tiling::engine::ScreenGeometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Designated Snap Zones according to Ermete OS ext_ermete_snap_v1 protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum SnapZone {
    None = 0,
    LeftHalf = 1,
    RightHalf = 2,
    TopHalf = 3,
    BottomHalf = 4,
    TopLeftQuadrant = 5,
    TopRightQuadrant = 6,
    BottomLeftQuadrant = 7,
    BottomRightQuadrant = 8,
    CenterStage = 9,
    CustomRegion = 10,
}

impl SnapZone {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::LeftHalf,
            2 => Self::RightHalf,
            3 => Self::TopHalf,
            4 => Self::BottomHalf,
            5 => Self::TopLeftQuadrant,
            6 => Self::TopRightQuadrant,
            7 => Self::BottomLeftQuadrant,
            8 => Self::BottomRightQuadrant,
            9 => Self::CenterStage,
            10 => Self::CustomRegion,
            _ => Self::None,
        }
    }
}

/// Flags modifying snap behavior.
pub struct SnapFlag;
impl SnapFlag {
    pub const ANIMATE: u32 = 1;
    pub const AUTO_REFLOW: u32 = 2;
    pub const STICKY: u32 = 4;
}

/// Custom bounding rectangle for SnapZone::CustomRegion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Active snap state of a surface/window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapState {
    pub window_id: u64,
    pub zone: SnapZone,
    pub flags: u32,
    pub custom_region: Option<CustomRegion>,
    pub snapped_geometry: WindowPlacement,
}

/// Engine managing window snap zones and layout grid placement calculation.
pub struct SnapEngine {
    snapped_windows: HashMap<u64, SnapState>,
}

impl Default for SnapEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapEngine {
    pub fn new() -> Self {
        Self {
            snapped_windows: HashMap::new(),
        }
    }

    /// Computes window geometry for a given SnapZone based on screen geometry and inner/outer gaps.
    pub fn calculate_zone_geometry(
        &self,
        zone: SnapZone,
        custom: Option<CustomRegion>,
        screen: &ScreenGeometry,
        gaps: (u32, u32),
        workspace: u32,
        window_id: u64,
    ) -> WindowPlacement {
        let (inner, outer) = (gaps.0 as i32, gaps.1 as i32);
        let screen_w = screen.width as i32;
        let screen_h = screen.height as i32;

        let avail_w = (screen_w - 2 * outer).max(100);
        let avail_h = (screen_h - 2 * outer).max(100);

        let half_w = ((avail_w - inner) / 2).max(50);
        let half_h = ((avail_h - inner) / 2).max(50);

        let (x, y, w, h) = match zone {
            SnapZone::None => (outer, outer, avail_w, avail_h),
            SnapZone::LeftHalf => (outer, outer, half_w, avail_h),
            SnapZone::RightHalf => (outer + half_w + inner, outer, avail_w - half_w - inner, avail_h),
            SnapZone::TopHalf => (outer, outer, avail_w, half_h),
            SnapZone::BottomHalf => (outer, outer + half_h + inner, avail_w, avail_h - half_h - inner),
            SnapZone::TopLeftQuadrant => (outer, outer, half_w, half_h),
            SnapZone::TopRightQuadrant => (outer + half_w + inner, outer, avail_w - half_w - inner, half_h),
            SnapZone::BottomLeftQuadrant => (outer, outer + half_h + inner, half_w, avail_h - half_h - inner),
            SnapZone::BottomRightQuadrant => (
                outer + half_w + inner,
                outer + half_h + inner,
                avail_w - half_w - inner,
                avail_h - half_h - inner,
            ),
            SnapZone::CenterStage => {
                let stage_w = (avail_w * 2 / 3).max(100);
                let stage_h = (avail_h * 2 / 3).max(100);
                let stage_x = outer + (avail_w - stage_w) / 2;
                let stage_y = outer + (avail_h - stage_h) / 2;
                (stage_x, stage_y, stage_w, stage_h)
            }
            SnapZone::CustomRegion => {
                if let Some(c) = custom {
                    (c.x, c.y, c.width as i32, c.height as i32)
                } else {
                    (outer, outer, avail_w, avail_h)
                }
            }
        };

        WindowPlacement {
            window_id,
            x,
            y,
            width: w as u32,
            height: h as u32,
            workspace,
        }
    }

    /// Snaps a window into a designated snap zone and registers its state.
    #[allow(clippy::too_many_arguments)]
    pub fn snap_window(
        &mut self,
        window_id: u64,
        zone: SnapZone,
        flags: u32,
        custom_region: Option<CustomRegion>,
        screen: &ScreenGeometry,
        gaps: (u32, u32),
        workspace: u32,
    ) -> WindowPlacement {
        let placement = self.calculate_zone_geometry(zone, custom_region, screen, gaps, workspace, window_id);
        let state = SnapState {
            window_id,
            zone,
            flags,
            custom_region,
            snapped_geometry: placement.clone(),
        };

        info!(
            "Window {} snapped to zone {:?} (flags: {:#x}, geometry: {}x{} at {},{})",
            window_id, zone, flags, placement.width, placement.height, placement.x, placement.y
        );

        self.snapped_windows.insert(window_id, state);
        placement
    }

    /// Releases a window from its snap zone.
    pub fn unsnap_window(&mut self, window_id: u64) -> Option<SnapState> {
        if let Some(state) = self.snapped_windows.remove(&window_id) {
            info!("Window {} unsnapped from zone {:?}", window_id, state.zone);
            Some(state)
        } else {
            None
        }
    }

    /// Returns active snap state for a given window ID.
    pub fn get_snap_state(&self, window_id: u64) -> Option<&SnapState> {
        self.snapped_windows.get(&window_id)
    }

    /// Checks if a window is currently snapped.
    pub fn is_snapped(&self, window_id: u64) -> bool {
        self.snapped_windows.contains_key(&window_id)
    }

    /// Detects proposed snap zone when dragging window cursor near screen edges or corners.
    pub fn detect_snap_zone_from_cursor(
        &self,
        cursor_x: i32,
        cursor_y: i32,
        screen: &ScreenGeometry,
        margin: i32,
    ) -> SnapZone {
        let screen_w = screen.width as i32;
        let screen_h = screen.height as i32;

        let near_left = cursor_x <= margin;
        let near_right = cursor_x >= screen_w - margin;
        let near_top = cursor_y <= margin;
        let near_bottom = cursor_y >= screen_h - margin;

        if near_top && near_left {
            SnapZone::TopLeftQuadrant
        } else if near_top && near_right {
            SnapZone::TopRightQuadrant
        } else if near_bottom && near_left {
            SnapZone::BottomLeftQuadrant
        } else if near_bottom && near_right {
            SnapZone::BottomRightQuadrant
        } else if near_left {
            SnapZone::LeftHalf
        } else if near_right {
            SnapZone::RightHalf
        } else if near_top {
            SnapZone::TopHalf
        } else if near_bottom {
            SnapZone::BottomHalf
        } else {
            SnapZone::None
        }
    }

    /// Returns all currently snapped window states.
    pub fn all_snapped_windows(&self) -> Vec<&SnapState> {
        self.snapped_windows.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_engine_zones_and_cursor_detection() {
        let mut engine = SnapEngine::new();
        let screen = ScreenGeometry { width: 1920, height: 1080 };
        let gaps = (10, 20);

        // Test snapping left half
        let placement = engine.snap_window(1, SnapZone::LeftHalf, SnapFlag::ANIMATE, None, &screen, gaps, 1);
        assert_eq!(placement.x, 20);
        assert_eq!(placement.y, 20);
        assert!(placement.width > 0);
        assert!(placement.height > 0);
        assert!(engine.is_snapped(1));

        // Test cursor snap zone detection
        assert_eq!(engine.detect_snap_zone_from_cursor(5, 5, &screen, 20), SnapZone::TopLeftQuadrant);
        assert_eq!(engine.detect_snap_zone_from_cursor(1915, 5, &screen, 20), SnapZone::TopRightQuadrant);
        assert_eq!(engine.detect_snap_zone_from_cursor(5, 500, &screen, 20), SnapZone::LeftHalf);
        assert_eq!(engine.detect_snap_zone_from_cursor(500, 500, &screen, 20), SnapZone::None);

        // Test custom region snap
        let custom = CustomRegion { x: 100, y: 100, width: 400, height: 300 };
        let custom_placement = engine.snap_window(2, SnapZone::CustomRegion, 0, Some(custom), &screen, gaps, 1);
        assert_eq!(custom_placement.x, 100);
        assert_eq!(custom_placement.y, 100);
        assert_eq!(custom_placement.width, 400);
        assert_eq!(custom_placement.height, 300);

        // Test unsnap
        let unsnapped = engine.unsnap_window(1);
        assert!(unsnapped.is_some());
        assert!(!engine.is_snapped(1));
    }
}
