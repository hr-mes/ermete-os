use crate::ipc::protocol::{WindowInfo, WindowPlacement};
use crate::tiling::engine::ScreenGeometry;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Spatial Overview direction navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverviewDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Representation of a window rendered in Spatial Overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewWindowCard {
    pub window_id: u64,
    pub title: String,
    pub app_id: String,
    pub original_geometry: WindowPlacement,
    pub overview_geometry: WindowPlacement,
    pub workspace: u32,
    pub is_focused: bool,
}

/// Representation of a workspace card containing window cards in Spatial Overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewWorkspaceCard {
    pub workspace_id: u32,
    pub bounds: WindowPlacement,
    pub windows: Vec<OverviewWindowCard>,
    pub is_active_workspace: bool,
}

/// Controller managing Spatial Overview state, workspace cards, and layout grid calculations.
pub struct SpatialOverview {
    active: bool,
    scale_factor: f32,
    focused_window_id: Option<u64>,
    workspace_cards: Vec<OverviewWorkspaceCard>,
}

impl Default for SpatialOverview {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl SpatialOverview {
    pub fn new() -> Self {
        Self {
            active: false,
            scale_factor: 0.65,
            focused_window_id: None,
            workspace_cards: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale.clamp(0.2, 0.95);
    }

    /// Enters Spatial Overview mode and computes scaled window card positions across workspaces.
    pub fn enter_overview(
        &mut self,
        windows: &[WindowInfo],
        active_workspace: u32,
        total_workspaces: u32,
        screen: &ScreenGeometry,
    ) {
        self.active = true;
        info!(
            "Entering Spatial Overview mode for {} workspaces and {} total windows",
            total_workspaces,
            windows.len()
        );

        let ws_count = total_workspaces.max(1) as usize;
        let cols = (ws_count as f32).sqrt().ceil() as usize;
        let rows = (ws_count as f32 / cols as f32).ceil() as usize;

        let margin = 40i32;
        let avail_w = screen.width as i32 - 2 * margin;
        let avail_h = screen.height as i32 - 2 * margin;

        let ws_w = ((avail_w - (cols as i32 - 1) * 20) / cols as i32).max(100);
        let ws_h = ((avail_h - (rows as i32 - 1) * 20) / rows as i32).max(100);

        let mut cards = Vec::new();
        let scale = (ws_w as f32 / screen.width as f32).min(ws_h as f32 / screen.height as f32) * self.scale_factor;

        for ws_idx in 0..ws_count {
            let ws_id = (ws_idx + 1) as u32;
            let r = ws_idx / cols;
            let c = ws_idx % cols;

            let ws_x = margin + c as i32 * (ws_w + 20);
            let ws_y = margin + r as i32 * (ws_h + 20);

            let ws_bounds = WindowPlacement {
                window_id: 0,
                x: ws_x,
                y: ws_y,
                width: ws_w as u32,
                height: ws_h as u32,
                workspace: ws_id,
            };

            let ws_windows: Vec<OverviewWindowCard> = windows
                .iter()
                .filter(|w| w.geometry.workspace == ws_id)
                .map(|win| {
                    let scaled_x = ws_x + (win.geometry.x as f32 * scale) as i32;
                    let scaled_y = ws_y + (win.geometry.y as f32 * scale) as i32;
                    let scaled_w = (win.geometry.width as f32 * scale).max(20.0) as u32;
                    let scaled_h = (win.geometry.height as f32 * scale).max(20.0) as u32;

                    OverviewWindowCard {
                        window_id: win.id,
                        title: win.title.clone(),
                        app_id: win.app_id.clone(),
                        original_geometry: win.geometry.clone(),
                        overview_geometry: WindowPlacement {
                            window_id: win.id,
                            x: scaled_x,
                            y: scaled_y,
                            width: scaled_w,
                            height: scaled_h,
                            workspace: ws_id,
                        },
                        workspace: ws_id,
                        is_focused: win.is_focused,
                    }
                })
                .collect();

            cards.push(OverviewWorkspaceCard {
                workspace_id: ws_id,
                bounds: ws_bounds,
                windows: ws_windows,
                is_active_workspace: ws_id == active_workspace,
            });
        }

        self.workspace_cards = cards;
        if self.focused_window_id.is_none() {
            self.focused_window_id = windows.iter().find(|w| w.is_focused).map(|w| w.id);
        }
    }

    /// Exits Spatial Overview mode.
    pub fn exit_overview(&mut self) {
        info!("Exiting Spatial Overview mode");
        self.active = false;
        self.workspace_cards.clear();
    }

    /// Selects a window in Spatial Overview mode, exiting overview and returning (window_id, workspace_id).
    pub fn select_window(&mut self, window_id: u64) -> Option<(u64, u32)> {
        for card in &self.workspace_cards {
            for win in &card.windows {
                if win.window_id == window_id {
                    self.focused_window_id = Some(window_id);
                    let target_ws = card.workspace_id;
                    self.exit_overview();
                    return Some((window_id, target_ws));
                }
            }
        }
        None
    }

    /// Navigates window selection grid in Spatial Overview.
    pub fn navigate_grid(&mut self, direction: OverviewDirection) -> Option<u64> {
        let all_windows: Vec<&OverviewWindowCard> = self
            .workspace_cards
            .iter()
            .flat_map(|ws| ws.windows.iter())
            .collect();

        if all_windows.is_empty() {
            return None;
        }

        let current_idx = self
            .focused_window_id
            .and_then(|id| all_windows.iter().position(|w| w.window_id == id))
            .unwrap_or(0);

        let next_idx = match direction {
            OverviewDirection::Right | OverviewDirection::Down => (current_idx + 1) % all_windows.len(),
            OverviewDirection::Left | OverviewDirection::Up => {
                if current_idx == 0 {
                    all_windows.len() - 1
                } else {
                    current_idx - 1
                }
            }
        };

        let target_id = all_windows[next_idx].window_id;
        self.focused_window_id = Some(target_id);

        for ws in &mut self.workspace_cards {
            for win in &mut ws.windows {
                win.is_focused = win.window_id == target_id;
            }
        }

        Some(target_id)
    }

    pub fn get_workspace_cards(&self) -> &[OverviewWorkspaceCard] {
        &self.workspace_cards
    }

    pub fn focused_window_id(&self) -> Option<u64> {
        self.focused_window_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_overview_lifecycle_and_navigation() {
        let mut overview = SpatialOverview::new();
        assert!(!overview.is_active());

        let screen = ScreenGeometry { width: 1920, height: 1080 };
        let windows = vec![
            WindowInfo {
                id: 101,
                title: "Terminal".into(),
                app_id: "foot".into(),
                geometry: WindowPlacement {
                    window_id: 101,
                    x: 20,
                    y: 20,
                    width: 900,
                    height: 1000,
                    workspace: 1,
                },
                is_focused: true,
            },
            WindowInfo {
                id: 102,
                title: "Editor".into(),
                app_id: "code".into(),
                geometry: WindowPlacement {
                    window_id: 102,
                    x: 940,
                    y: 20,
                    width: 900,
                    height: 1000,
                    workspace: 2,
                },
                is_focused: false,
            },
        ];

        overview.enter_overview(&windows, 1, 2, &screen);
        assert!(overview.is_active());
        assert_eq!(overview.get_workspace_cards().len(), 2);

        // Test grid navigation
        let next_id = overview.navigate_grid(OverviewDirection::Right);
        assert_eq!(next_id, Some(102));

        // Test window selection
        let selected = overview.select_window(102);
        assert_eq!(selected, Some((102, 2)));
        assert!(!overview.is_active());
    }
}
