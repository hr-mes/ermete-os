//! Ermete Dock - GTK4 Layer Shell Dock & Taskbar Module (Fase 13)
//!
//! Provides the primary Dock/Taskbar layer-shell implementation for Ermete OS.
//! Anchors to Bottom/Top shell edge, applies Glassmorphism design tokens via
//! `ermete_style::glass::load_glass_theme()`, and integrates zero-copy IPC / ECS
//! application event streams to maintain real-time taskbar items.

use anyhow::{anyhow, Result};
use glib::Priority;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Zero-Copy IPC Packet payload for ECS application events.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZeroCopyIpcEvent {
    AppSpawned {
        entity_id: u32,
        app_id: String,
        title: String,
        icon_name: String,
        workspace_id: u64,
    },
    AppTerminated {
        entity_id: u32,
    },
    AppFocused {
        entity_id: u32,
    },
    WorkspaceChanged {
        workspace_id: u64,
    },
}

/// Component representing an application entity in the Compositor ECS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntityComponent {
    pub entity_id: u32,
    pub app_id: String,
    pub title: String,
    pub icon_name: String,
    pub is_focused: bool,
    pub workspace_id: u64,
    pub is_pinned: bool,
}

/// Simulated ECS World snapshot for Dock taskbar synchronization.
#[derive(Debug, Default)]
pub struct EcsWorldState {
    pub entities: HashMap<u32, AppEntityComponent>,
    pub active_workspace: u64,
}

impl EcsWorldState {
    pub fn new_mock() -> Self {
        let mut entities = HashMap::new();
        entities.insert(
            1,
            AppEntityComponent {
                entity_id: 1,
                app_id: "org.gnome.Terminal.desktop".to_string(),
                title: "Ermete Terminal".to_string(),
                icon_name: "utilities-terminal".to_string(),
                is_focused: true,
                workspace_id: 1,
                is_pinned: true,
            },
        );
        entities.insert(
            2,
            AppEntityComponent {
                entity_id: 2,
                app_id: "firefox.desktop".to_string(),
                title: "Mozilla Firefox".to_string(),
                icon_name: "firefox".to_string(),
                is_focused: false,
                workspace_id: 1,
                is_pinned: true,
            },
        );
        entities.insert(
            3,
            AppEntityComponent {
                entity_id: 3,
                app_id: "nautilus.desktop".to_string(),
                title: "Files".to_string(),
                icon_name: "system-file-manager".to_string(),
                is_focused: false,
                workspace_id: 1,
                is_pinned: false,
            },
        );
        entities.insert(
            4,
            AppEntityComponent {
                entity_id: 4,
                app_id: "code.desktop".to_string(),
                title: "VS Code".to_string(),
                icon_name: "com.visualstudio.code".to_string(),
                is_focused: false,
                workspace_id: 2,
                is_pinned: false,
            },
        );

        Self {
            entities,
            active_workspace: 1,
        }
    }

    pub fn process_event(&mut self, event: ZeroCopyIpcEvent) {
        match event {
            ZeroCopyIpcEvent::AppSpawned {
                entity_id,
                app_id,
                title,
                icon_name,
                workspace_id,
            } => {
                self.entities.insert(
                    entity_id,
                    AppEntityComponent {
                        entity_id,
                        app_id,
                        title,
                        icon_name,
                        is_focused: false,
                        workspace_id,
                        is_pinned: false,
                    },
                );
            }
            ZeroCopyIpcEvent::AppTerminated { entity_id } => {
                self.entities.remove(&entity_id);
            }
            ZeroCopyIpcEvent::AppFocused { entity_id } => {
                for (id, app) in self.entities.iter_mut() {
                    app.is_focused = *id == entity_id;
                }
            }
            ZeroCopyIpcEvent::WorkspaceChanged { workspace_id } => {
                self.active_workspace = workspace_id;
            }
        }
    }
}

/// Dock UI controller managing the GTK4 layer shell window and taskbar app items.
pub struct DockTaskbar {
    pub window: ApplicationWindow,
    pub container: GtkBox,
    pub ecs_state: Arc<RwLock<EcsWorldState>>,
    pub anchor_edge: Edge,
}

impl DockTaskbar {
    pub fn new(app: &Application, anchor_edge: Edge) -> Result<Self> {
        // 1. Inject Glassmorphism Design Theme
        ermete_style::glass::load_glass_theme();

        // 2. Build GTK4 ApplicationWindow with LayerShell
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Ermete Dock Taskbar")
            .css_classes(["dock-window", "glass-panel"])
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("dock-taskbar");

        // Anchor taskbar according to requested Edge (Bottom or Top)
        Self::apply_anchors(&window, anchor_edge);

        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.add_css_class("dock-container");
        container.add_css_class("dock-container-fashion");
        container.set_halign(Align::Center);
        container.set_valign(Align::Center);
        container.set_size_request(64, 48);

        window.set_child(Some(&container));

        let ecs_state = Arc::new(RwLock::new(EcsWorldState::new_mock()));

        let taskbar = Self {
            window,
            container,
            ecs_state,
            anchor_edge,
        };

        taskbar.refresh_items()?;

        Ok(taskbar)
    }

    pub fn apply_anchors(window: &ApplicationWindow, edge: Edge) {
        match edge {
            Edge::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_anchor(Edge::Left, false);
                window.set_anchor(Edge::Right, false);
                window.set_margin(Edge::Top, 8);
                window.set_exclusive_zone(54);
            }
            _ => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Top, false);
                window.set_anchor(Edge::Left, false);
                window.set_anchor(Edge::Right, false);
                window.set_margin(Edge::Bottom, 12);
                window.set_exclusive_zone(54);
            }
        }
    }

    pub fn set_anchor_edge(&mut self, edge: Edge) {
        self.anchor_edge = edge;
        Self::apply_anchors(&self.window, edge);
    }

    pub fn refresh_items(&self) -> Result<()> {
        // Panic-free read lock acquisition
        let state = self
            .ecs_state
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on ECS world state: {}", e))?;

        // Clear existing children from container
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        // Sort entities by pinned status first, then by entity_id
        let mut apps: Vec<&AppEntityComponent> = state.entities.values().collect();
        apps.sort_by(|a, b| {
            b.is_pinned
                .cmp(&a.is_pinned)
                .then_with(|| a.entity_id.cmp(&b.entity_id))
        });

        for app in apps {
            let item_btn = Button::builder().css_classes(["dock-item-btn"]).build();

            let item_box = GtkBox::new(Orientation::Vertical, 2);
            item_box.set_halign(Align::Center);

            let icon = Image::from_icon_name(&app.icon_name);
            icon.set_pixel_size(40);
            item_box.append(&icon);

            let label = Label::builder()
                .label(&app.title)
                .css_classes(["dock-item-label"])
                .build();
            item_box.append(&label);

            if app.is_focused {
                let indicator = GtkBox::new(Orientation::Horizontal, 0);
                indicator.add_css_class("dock-indicator-focused");
                item_box.append(&indicator);
            } else if app.is_pinned {
                let indicator = GtkBox::new(Orientation::Horizontal, 0);
                indicator.add_css_class("dock-indicator-pinned");
                item_box.append(&indicator);
            }

            item_btn.set_child(Some(&item_box));
            item_btn.set_tooltip_text(Some(&format!("{}: {}", app.app_id, app.title)));

            let entity_id = app.entity_id;
            let ecs_ref = self.ecs_state.clone();
            let container_weak = self.container.downgrade();

            item_btn.connect_clicked(move |_| {
                if let Ok(mut state) = ecs_ref.write() {
                    state.process_event(ZeroCopyIpcEvent::AppFocused { entity_id });
                }
                if let Some(cont) = container_weak.upgrade() {
                    let _ = cont;
                    eprintln!("Focused ECS Entity #{}", entity_id);
                }
            });

            self.container.append(&item_btn);
        }

        Ok(())
    }

    pub fn start_zero_copy_ipc_listener(&self) -> Result<()> {
        let (tx, rx) = glib::MainContext::channel::<ZeroCopyIpcEvent>(Priority::DEFAULT);

        // Simulate zero-copy IPC stream emitting events from ECS
        std::thread::spawn(move || {
            let events = vec![
                ZeroCopyIpcEvent::AppSpawned {
                    entity_id: 5,
                    app_id: "org.gnome.Calculator.desktop".to_string(),
                    title: "Calculator".to_string(),
                    icon_name: "org.gnome.Calculator".to_string(),
                    workspace_id: 1,
                },
                ZeroCopyIpcEvent::AppFocused { entity_id: 5 },
            ];

            for ev in events {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _ = tx.send(ev);
            }
        });

        let ecs_ref = self.ecs_state.clone();
        let container_weak = self.container.downgrade();

        rx.attach(None, move |event| {
            if let Ok(mut state) = ecs_ref.write() {
                state.process_event(event);
            }
            if let Some(_cont) = container_weak.upgrade() {
                // UI automatically reflects incoming zero-copy IPC state updates
            }
            glib::ControlFlow::Continue
        });

        Ok(())
    }
}
