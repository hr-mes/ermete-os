use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::cell::RefCell;
use std::rc::Rc;
use zbus::interface;

/// Represents the geometric morphing state of the Morphic Pill (Dynamic Island)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PillState {
    Compact,
    Expanded,
    Interactive,
}

/// Dynamic LiveActivity payload received via ZBus or internal system events
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiveActivityPayload {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub progress: Option<f64>,
    pub state: PillState,
    pub category: String,
}

impl Default for LiveActivityPayload {
    fn default() -> Self {
        Self {
            id: "system-ai".to_string(),
            title: "Ermete AI".to_string(),
            subtitle: "Agent Swarm Active".to_string(),
            icon: "✨".to_string(),
            progress: Some(0.42),
            state: PillState::Compact,
            category: "ai".to_string(),
        }
    }
}

/// Damped Harmonic Oscillator Spring Physics Solver
#[derive(Debug, Clone)]
pub struct Spring {
    pub current: f64,
    pub target: f64,
    pub velocity: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Spring {
    pub fn new(initial: f64, stiffness: f64, damping: f64) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
            stiffness,
            damping,
            mass: 1.0,
        }
    }

    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn update(&mut self, dt: f64) -> bool {
        let spring_force = -self.stiffness * (self.current - self.target);
        let damping_force = -self.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.mass;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;

        let distance = (self.current - self.target).abs();
        if distance < 0.05 && self.velocity.abs() < 0.05 {
            self.current = self.target;
            self.velocity = 0.0;
            false
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct SpringState {
    pub width_spring: Spring,
    pub height_spring: Spring,
}

impl SpringState {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width_spring: Spring::new(width, 240.0, 22.0),
            height_spring: Spring::new(height, 240.0, 22.0),
        }
    }

    pub fn set_targets_for_state(&mut self, state: PillState) {
        let (w, h) = match state {
            PillState::Compact => (120.0, 28.0),
            PillState::Expanded => (270.0, 44.0),
            PillState::Interactive => (350.0, 86.0),
        };
        self.width_spring.set_target(w);
        self.height_spring.set_target(h);
    }
}

pub struct MorphicPillModel {
    pub state: PillState,
    pub payload: LiveActivityPayload,
    pub is_hovered: bool,
    pub spring_state: Rc<RefCell<SpringState>>,
}

#[derive(Debug)]
pub enum MorphicPillInput {
    UpdateActivity(LiveActivityPayload),
    SetState(PillState),
    ToggleState,
    HoverChanged(bool),
    DismissActivity,
    ActionButtonClicked(String),
}

/// ZBus Interface for `os.ermete.Shell.LiveActivity`
pub struct LiveActivityZbusServer {
    sender: relm4::ComponentSender<MorphicPillModel>,
}

#[interface(name = "os.ermete.Shell.LiveActivity")]
impl LiveActivityZbusServer {
    pub async fn update_activity(
        &self,
        id: String,
        state_str: String,
        title: String,
        subtitle: String,
        icon: String,
        progress: f64,
    ) {
        let state = match state_str.to_lowercase().as_str() {
            "expanded" => PillState::Expanded,
            "interactive" => PillState::Interactive,
            _ => PillState::Compact,
        };
        let payload = LiveActivityPayload {
            id,
            title,
            subtitle,
            icon,
            progress: if progress >= 0.0 { Some(progress) } else { None },
            state,
            category: "zbus".to_string(),
        };
        let _ = self.sender.input(MorphicPillInput::UpdateActivity(payload));
    }

    pub async fn set_state(&self, state_str: String) {
        let state = match state_str.to_lowercase().as_str() {
            "expanded" => PillState::Expanded,
            "interactive" => PillState::Interactive,
            _ => PillState::Compact,
        };
        let _ = self.sender.input(MorphicPillInput::SetState(state));
    }

    pub async fn dismiss(&self, _id: String) {
        let _ = self.sender.input(MorphicPillInput::DismissActivity);
    }
}

pub fn spawn_zbus_listener(sender: relm4::ComponentSender<MorphicPillModel>) {
    glib::MainContext::default().spawn_local(async move {
        let server = LiveActivityZbusServer { sender };

        let builder = match zbus::connection::Builder::session() {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to get session bus for LiveActivity");
                return;
            }
        };

        let builder = match builder.name("os.ermete.Shell.LiveActivity") {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to request LiveActivity DBus name");
                return;
            }
        };

        let builder = match builder.serve_at("/os/ermete/Shell/LiveActivity", server) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to serve LiveActivity DBus interface");
                return;
            }
        };

        if let Ok(_conn) = builder.build().await {
            tracing::info!("Registered os.ermete.Shell.LiveActivity ZBus daemon cleanly");
        }
    });
}

#[relm4::component(pub)]
impl SimpleComponent for MorphicPillModel {
    type Input = MorphicPillInput;
    type Output = ();
    type Init = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "morphic-pill-container",
            set_valign: gtk::Align::Center,
            set_halign: gtk::Align::Center,

            add_controller = gtk::GestureClick {
                connect_pressed[sender] => move |_, _, _, _| {
                    sender.input(MorphicPillInput::ToggleState);
                },
            },

            add_controller = gtk::EventControllerMotion {
                connect_enter[sender] => move |_, _, _| {
                    sender.input(MorphicPillInput::HoverChanged(true));
                },
                connect_leave[sender] => move |_| {
                    sender.input(MorphicPillInput::HoverChanged(false));
                },
            },

            // --- COMPACT VIEW ---
            gtk::Box {
                #[watch]
                set_visible: model.state == PillState::Compact,
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    #[watch]
                    set_label: &model.payload.icon,
                    add_css_class: "morphic-pill-icon",
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.payload.title,
                    add_css_class: "morphic-pill-compact",
                },
            },

            // --- EXPANDED VIEW ---
            gtk::Box {
                #[watch]
                set_visible: model.state == PillState::Expanded,
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    #[watch]
                    set_label: &model.payload.icon,
                    add_css_class: "morphic-pill-icon",
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        #[watch]
                        set_label: &model.payload.title,
                        add_css_class: "morphic-pill-title",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &model.payload.subtitle,
                        add_css_class: "morphic-pill-subtitle",
                        set_halign: gtk::Align::Start,
                    },
                },

                gtk::ProgressBar {
                    #[watch]
                    set_visible: model.payload.progress.is_some(),
                    #[watch]
                    set_fraction: model.payload.progress.unwrap_or(0.0),
                    add_css_class: "morphic-pill-progress",
                    set_valign: gtk::Align::Center,
                    set_width_request: 60,
                },
            },

            // --- INTERACTIVE VIEW ---
            gtk::Box {
                #[watch]
                set_visible: model.state == PillState::Interactive,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 6,
                set_valign: gtk::Align::Center,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Label {
                        #[watch]
                        set_label: &model.payload.icon,
                        add_css_class: "morphic-pill-icon",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,

                        gtk::Label {
                            #[watch]
                            set_label: &model.payload.title,
                            add_css_class: "morphic-pill-title",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Label {
                            #[watch]
                            set_label: &model.payload.subtitle,
                            add_css_class: "morphic-pill-subtitle",
                            set_halign: gtk::Align::Start,
                        },
                    },

                    gtk::Button {
                        set_label: "✕",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::DismissActivity,
                    },
                },

                gtk::ProgressBar {
                    #[watch]
                    set_visible: model.payload.progress.is_some(),
                    #[watch]
                    set_fraction: model.payload.progress.unwrap_or(0.0),
                    add_css_class: "morphic-pill-progress",
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_halign: gtk::Align::End,

                    gtk::Button {
                        set_label: "⏸ Pause",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::ActionButtonClicked("toggle_pause".to_string()),
                    },

                    gtk::Button {
                        set_label: "Collapse",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::SetState(PillState::Compact),
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let initial_payload = LiveActivityPayload::default();
        let initial_state = initial_payload.state;

        let spring_state = Rc::new(RefCell::new(SpringState::new(120.0, 28.0)));
        spring_state.borrow_mut().set_targets_for_state(initial_state);

        let spring_clone = spring_state.clone();
        root.add_tick_callback(move |widget, _clock| {
            let mut s = spring_clone.borrow_mut();
            let dt = 0.016; // 60 FPS frame delta
            let w_active = s.width_spring.update(dt);
            let h_active = s.height_spring.update(dt);

            if w_active || h_active {
                widget.set_size_request(
                    s.width_spring.current as i32,
                    s.height_spring.current as i32,
                );
            }
            glib::ControlFlow::Continue
        });

        let sender_clone = sender.clone();
        spawn_zbus_listener(sender_clone);

        let model = MorphicPillModel {
            state: initial_state,
            payload: initial_payload,
            is_hovered: false,
            spring_state,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            MorphicPillInput::UpdateActivity(new_payload) => {
                self.state = new_payload.state;
                self.payload = new_payload;
                self.spring_state.borrow_mut().set_targets_for_state(self.state);
            }
            MorphicPillInput::SetState(new_state) => {
                self.state = new_state;
                self.spring_state.borrow_mut().set_targets_for_state(new_state);
            }
            MorphicPillInput::ToggleState => {
                let next_state = match self.state {
                    PillState::Compact => PillState::Expanded,
                    PillState::Expanded => PillState::Interactive,
                    PillState::Interactive => PillState::Compact,
                };
                self.state = next_state;
                self.spring_state.borrow_mut().set_targets_for_state(next_state);
            }
            MorphicPillInput::HoverChanged(is_hovered) => {
                self.is_hovered = is_hovered;
                if is_hovered && self.state == PillState::Compact {
                    self.state = PillState::Expanded;
                    self.spring_state.borrow_mut().set_targets_for_state(PillState::Expanded);
                } else if !is_hovered && self.state == PillState::Expanded {
                    self.state = PillState::Compact;
                    self.spring_state.borrow_mut().set_targets_for_state(PillState::Compact);
                }
            }
            MorphicPillInput::DismissActivity => {
                self.payload = LiveActivityPayload::default();
                self.state = PillState::Compact;
                self.spring_state.borrow_mut().set_targets_for_state(PillState::Compact);
            }
            MorphicPillInput::ActionButtonClicked(action) => {
                tracing::info!(action = %action, "MorphicPill action clicked");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_physics_convergence() {
        let mut spring = Spring::new(0.0, 240.0, 22.0);
        spring.set_target(100.0);

        let mut steps = 0;
        while spring.update(0.016) && steps < 500 {
            steps += 1;
        }

        assert_eq!(spring.current, 100.0);
        assert_eq!(spring.velocity, 0.0);
        assert!(steps < 200, "Spring physics took too long to converge: {} steps", steps);
    }

    #[test]
    fn test_pill_state_target_dimensions() {
        let mut s = SpringState::new(120.0, 28.0);
        
        s.set_targets_for_state(PillState::Expanded);
        assert_eq!(s.width_spring.target, 270.0);
        assert_eq!(s.height_spring.target, 44.0);

        s.set_targets_for_state(PillState::Interactive);
        assert_eq!(s.width_spring.target, 350.0);
        assert_eq!(s.height_spring.target, 86.0);

        s.set_targets_for_state(PillState::Compact);
        assert_eq!(s.width_spring.target, 120.0);
        assert_eq!(s.height_spring.target, 28.0);
    }
}
