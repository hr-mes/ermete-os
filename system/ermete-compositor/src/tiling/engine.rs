use crate::animation::AnimationEngine;
use crate::ipc::protocol::{LayoutMode, WindowInfo, WindowPlacement};
use std::collections::HashMap;
use tracing::{info, debug};

#[derive(Debug, Clone)]
pub struct ScreenGeometry {
    pub width: u32,
    pub height: u32,
}

impl Default for ScreenGeometry {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

pub struct TilingEngine {
    mode: LayoutMode,
    screen: ScreenGeometry,
    inner_gap: u32,
    outer_gap: u32,
    active_workspace: u32,
    windows: HashMap<u64, WindowInfo>,
    window_order: Vec<u64>,
    focused_window_id: Option<u64>,
    pub animation_engine: AnimationEngine,
}

impl Default for TilingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl TilingEngine {
    pub fn new() -> Self {
        Self {
            mode: LayoutMode::AiDriven,
            screen: ScreenGeometry::default(),
            inner_gap: 8,
            outer_gap: 12,
            active_workspace: 1,
            windows: HashMap::new(),
            window_order: Vec::new(),
            focused_window_id: None,
            animation_engine: AnimationEngine::default(),
        }
    }

    pub fn set_mode(&mut self, mode: LayoutMode) {
        info!("Tiling engine layout mode changed to: {}", mode);
        self.mode = mode;
        self.recalculate_layout();
    }

    pub fn mode(&self) -> LayoutMode {
        self.mode
    }

    pub fn set_gaps(&mut self, inner: u32, outer: u32) {
        self.inner_gap = inner;
        self.outer_gap = outer;
        self.recalculate_layout();
    }

    pub fn gaps(&self) -> (u32, u32) {
        (self.inner_gap, self.outer_gap)
    }

    pub fn active_workspace(&self) -> u32 {
        self.active_workspace
    }

    pub fn add_window(&mut self, id: u64, title: String, app_id: String) {
        let placement = WindowPlacement {
            window_id: id,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            workspace: self.active_workspace,
        };
        let info = WindowInfo {
            id,
            title,
            app_id,
            geometry: placement,
            is_focused: false,
        };
        if !self.windows.contains_key(&id) {
            self.window_order.push(id);
        }
        self.windows.insert(id, info);
        if self.focused_window_id.is_none() {
            self.focused_window_id = Some(id);
        }
        self.update_focus();
        self.recalculate_layout();
    }

    pub fn remove_window(&mut self, id: u64) {
        self.windows.remove(&id);
        self.window_order.retain(|&win_id| win_id != id);
        self.animation_engine.remove_window(id);
        if self.focused_window_id == Some(id) {
            self.focused_window_id = self.window_order.last().copied();
        }
        self.update_focus();
        self.recalculate_layout();
    }


    pub fn focus_window(&mut self, id: u64) -> bool {
        if self.windows.contains_key(&id) {
            self.focused_window_id = Some(id);
            self.update_focus();
            true
        } else {
            false
        }
    }

    pub fn apply_ai_placements(&mut self, placements: Vec<WindowPlacement>) {
        info!("Applying {} AI-generated window placements", placements.len());
        for placement in placements {
            if let Some(win) = self.windows.get_mut(&placement.window_id) {
                win.geometry = placement;
            }
        }
        self.sync_animation_targets();
    }

    pub fn sync_animation_targets(&mut self) {
        for win in self.windows.values() {
            self.animation_engine.update_window_target(win.geometry.clone());
        }
    }

    pub fn tick_animation(&mut self, dt: f64) -> bool {
        self.animation_engine.tick(dt)
    }

    pub fn windows(&self) -> Vec<WindowInfo> {
        self.window_order
            .iter()
            .filter_map(|id| {
                self.windows.get(id).map(|win| {
                    let mut animated_win = win.clone();
                    animated_win.geometry = self.animation_engine.current_placement(&win.geometry);
                    animated_win
                })
            })
            .collect()
    }

    pub fn target_windows(&self) -> Vec<WindowInfo> {
        self.window_order
            .iter()
            .filter_map(|id| self.windows.get(id).cloned())
            .collect()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    fn update_focus(&mut self) {
        for (id, win) in self.windows.iter_mut() {
            win.is_focused = Some(*id) == self.focused_window_id;
        }
    }

    pub fn recalculate_layout(&mut self) {
        let count = self.window_order.len();
        if count == 0 {
            return;
        }

        debug!("Recalculating layout for {} windows in mode {:?}", count, self.mode);

        match self.mode {
            LayoutMode::MasterStack => self.layout_master_stack(),
            LayoutMode::Grid => self.layout_grid(),
            LayoutMode::Spiral => self.layout_spiral(),
            LayoutMode::AiDriven | LayoutMode::Floating => self.layout_ai_dynamic(),
        }

        self.sync_animation_targets();
    }

    fn layout_master_stack(&mut self) {
        let count = self.window_order.len();
        let outer = self.outer_gap as i32;
        let inner = self.inner_gap as i32;
        let avail_w = self.screen.width as i32 - 2 * outer;
        let avail_h = self.screen.height as i32 - 2 * outer;

        if count == 1 {
            if let Some(win) = self.windows.get_mut(&self.window_order[0]) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer,
                    y: outer,
                    width: avail_w as u32,
                    height: avail_h as u32,
                    workspace: self.active_workspace,
                };
            }
            return;
        }

        let master_w = (avail_w - inner) / 2;
        let stack_w = avail_w - master_w - inner;

        // Master window
        if let Some(win) = self.windows.get_mut(&self.window_order[0]) {
            win.geometry = WindowPlacement {
                window_id: win.id,
                x: outer,
                y: outer,
                width: master_w as u32,
                height: avail_h as u32,
                workspace: self.active_workspace,
            };
        }

        // Stack windows
        let stack_count = count - 1;
        let stack_h = (avail_h - (stack_count as i32 - 1) * inner) / stack_count as i32;
        for (i, &id) in self.window_order.iter().skip(1).enumerate() {
            if let Some(win) = self.windows.get_mut(&id) {
                let y = outer + i as i32 * (stack_h + inner);
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer + master_w + inner,
                    y,
                    width: stack_w as u32,
                    height: stack_h as u32,
                    workspace: self.active_workspace,
                };
            }
        }
    }

    fn layout_grid(&mut self) {
        let count = self.window_order.len();
        let cols = (count as f64).sqrt().ceil() as usize;
        let rows = (count as f64 / cols as f64).ceil() as usize;

        let outer = self.outer_gap as i32;
        let inner = self.inner_gap as i32;
        let avail_w = self.screen.width as i32 - 2 * outer;
        let avail_h = self.screen.height as i32 - 2 * outer;

        let cell_w = (avail_w - (cols as i32 - 1) * inner) / cols as i32;
        let cell_h = (avail_h - (rows as i32 - 1) * inner) / rows as i32;

        for (idx, &id) in self.window_order.iter().enumerate() {
            let r = idx / cols;
            let c = idx % cols;

            if let Some(win) = self.windows.get_mut(&id) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer + c as i32 * (cell_w + inner),
                    y: outer + r as i32 * (cell_h + inner),
                    width: cell_w as u32,
                    height: cell_h as u32,
                    workspace: self.active_workspace,
                };
            }
        }
    }

    fn layout_spiral(&mut self) {
        let outer = self.outer_gap as i32;
        let inner = self.inner_gap as i32;
        let mut x = outer;
        let mut y = outer;
        let mut w = self.screen.width as i32 - 2 * outer;
        let mut h = self.screen.height as i32 - 2 * outer;

        let count = self.window_order.len();
        for (idx, &id) in self.window_order.iter().enumerate() {
            let is_last = idx == count - 1;
            let (win_w, win_h) = if is_last {
                (w, h)
            } else if idx % 2 == 0 {
                let half = (w - inner) / 2;
                w = w - half - inner;
                (half, h)
            } else {
                let half = (h - inner) / 2;
                h = h - half - inner;
                (w, half)
            };

            if let Some(win) = self.windows.get_mut(&id) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x,
                    y,
                    width: win_w.max(100) as u32,
                    height: win_h.max(100) as u32,
                    workspace: self.active_workspace,
                };
            }

            if idx % 2 == 0 && !is_last {
                x += win_w + inner;
            } else if !is_last {
                y += win_h + inner;
            }
        }
    }

    fn layout_ai_dynamic(&mut self) {
        // AI dynamic split algorithm: split screen along golden ratio or adaptive binary tree
        let count = self.window_order.len();
        if count == 0 {
            return;
        }

        let outer = self.outer_gap as i32;
        let inner = self.inner_gap as i32;
        let avail_w = self.screen.width as i32 - 2 * outer;
        let avail_h = self.screen.height as i32 - 2 * outer;

        if count == 1 {
            if let Some(win) = self.windows.get_mut(&self.window_order[0]) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer,
                    y: outer,
                    width: avail_w as u32,
                    height: avail_h as u32,
                    workspace: self.active_workspace,
                };
            }
            return;
        }

        // Binary partition according to window count
        let left_count = count.div_ceil(2);
        let right_count = count - left_count;

        let left_w = (avail_w - inner) * left_count as i32 / count as i32;
        let right_w = avail_w - left_w - inner;

        // Left column
        let left_h = (avail_h - (left_count as i32 - 1) * inner) / left_count as i32;
        for (i, &id) in self.window_order.iter().take(left_count).enumerate() {
            if let Some(win) = self.windows.get_mut(&id) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer,
                    y: outer + i as i32 * (left_h + inner),
                    width: left_w as u32,
                    height: left_h as u32,
                    workspace: self.active_workspace,
                };
            }
        }

        // Right column
        let right_h = (avail_h - (right_count as i32 - 1) * inner) / right_count as i32;
        for (i, &id) in self.window_order.iter().skip(left_count).enumerate() {
            if let Some(win) = self.windows.get_mut(&id) {
                win.geometry = WindowPlacement {
                    window_id: win.id,
                    x: outer + left_w + inner,
                    y: outer + i as i32 * (right_h + inner),
                    width: right_w as u32,
                    height: right_h as u32,
                    workspace: self.active_workspace,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling_engine_window_lifecycle_and_modes() {
        let mut engine = TilingEngine::new();
        assert_eq!(engine.window_count(), 0);

        engine.add_window(1, "Terminal".into(), "foot".into());
        engine.add_window(2, "Browser".into(), "firefox".into());
        assert_eq!(engine.window_count(), 2);

        // Test MasterStack layout calculation
        engine.set_mode(LayoutMode::MasterStack);
        let windows = engine.windows();
        assert_eq!(windows.len(), 2);
        assert!(windows[0].geometry.width > 0);
        assert!(windows[1].geometry.width > 0);

        // Test Grid layout calculation
        engine.set_mode(LayoutMode::Grid);
        let windows_grid = engine.windows();
        assert_eq!(windows_grid.len(), 2);

        // Remove window
        engine.remove_window(1);
        assert_eq!(engine.window_count(), 1);
    }
}
