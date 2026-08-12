#![allow(irrefutable_let_patterns)]
use gtk4::cairo;
use gtk4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use zbus::interface;

/// Represents the geometric morphing state of the Morphic Pill (Dynamic Island)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PillState {
    Compact,
    Expanded,
    Interactive,
    OsdVolume,
    OsdMedia,
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
            title: "Ermete OS".to_string(),
            subtitle: "System Active".to_string(),
            icon: "✨".to_string(),
            progress: Some(0.42),
            state: PillState::Compact,
            category: "system".to_string(),
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
        let (w, h, stiffness, damping) = match state {
            PillState::Compact => (120.0, 28.0, 200.0, 24.0),
            PillState::Expanded => (270.0, 44.0, 240.0, 22.0),
            PillState::Interactive => (350.0, 86.0, 240.0, 22.0),
            PillState::OsdVolume => (290.0, 46.0, 320.0, 20.0),
            PillState::OsdMedia => (360.0, 90.0, 260.0, 18.0),
        };
        self.width_spring.stiffness = stiffness;
        self.width_spring.damping = damping;
        self.height_spring.stiffness = stiffness;
        self.height_spring.damping = damping;
        self.width_spring.set_target(w);
        self.height_spring.set_target(h);
    }
}

/// Dynamic FFT Waveform Cairo Drawing Renderer
pub fn draw_fft_waveform(
    _area: &gtk4::DrawingArea,
    cr: &cairo::Context,
    width: i32,
    height: i32,
    tick: u64,
    volume: f64,
    is_playing: bool,
) {
    let num_bars = 8;
    let bar_spacing = 3.0;
    let total_spacing = bar_spacing * (num_bars - 1) as f64;
    let bar_width = ((width as f64 - total_spacing) / num_bars as f64).max(2.0);
    let max_h = height as f64;

    for i in 0..num_bars {
        let x = i as f64 * (bar_width + bar_spacing);
        let phase = tick as f64 * 0.15 + i as f64 * 0.75;
        let amp = if is_playing {
            0.35 + 0.55 * (phase.sin().abs())
        } else if volume > 0.0 {
            0.2 + 0.6 * volume * ((phase * 1.2).cos().abs())
        } else {
            0.15
        };

        let bar_h = (max_h * amp).clamp(4.0, max_h);
        let y = max_h - bar_h;

        let ratio = i as f64 / (num_bars - 1) as f64;
        let r = 0.0 + 0.66 * ratio;
        let g = 0.83 - 0.5 * ratio;
        let b = 1.0;
        let alpha = 0.9;

        cr.set_source_rgba(r, g, b, alpha);

        let corner_radius = (bar_width / 2.0).min(3.0);
        cr.new_sub_path();
        cr.arc(x + corner_radius, y + corner_radius, corner_radius, std::f64::consts::PI, 1.5 * std::f64::consts::PI);
        cr.arc(x + bar_width - corner_radius, y + corner_radius, corner_radius, 1.5 * std::f64::consts::PI, 0.0);
        cr.arc(x + bar_width - corner_radius, y + bar_h - corner_radius, corner_radius, 0.0, 0.5 * std::f64::consts::PI);
        cr.arc(x + corner_radius, y + bar_h - corner_radius, corner_radius, 0.5 * std::f64::consts::PI, std::f64::consts::PI);
        cr.close_path();
        let _ = cr.fill();
    }
}

#[derive(Clone, Default)]
pub struct FftDrawState {
    pub tick: u64,
    pub volume: f64,
    pub is_playing: bool,
}

pub struct MorphicPillModel {
    pub state: PillState,
    pub payload: LiveActivityPayload,
    pub is_hovered: bool,
    pub spring_state: Rc<RefCell<SpringState>>,
    pub volume: f64,
    pub is_muted: bool,
    pub media_title: String,
    pub media_artist: String,
    pub is_playing: bool,
    pub draw_state: Rc<RefCell<FftDrawState>>,
    pub auto_collapse_source: Rc<RefCell<Option<glib::SourceId>>>,
}

#[derive(Debug)]
pub enum MorphicPillInput {
    UpdateActivity(LiveActivityPayload),
    SetState(PillState),
    ToggleState,
    HoverChanged(bool),
    DismissActivity,
    ActionButtonClicked(String),
    VolumeChanged(f64),
    MuteToggled(bool),
    MediaStateChanged { title: String, artist: String, is_playing: bool },
    MediaPlayPause,
    MediaNext,
    MediaPrev,
    AutoCollapse,
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
        let b_id = if id.len() > 256 { id[..256].to_string() } else { id };
        let b_title = if title.len() > 1024 { title[..1024].to_string() } else { title };
        let b_subtitle = if subtitle.len() > 1024 { subtitle[..1024].to_string() } else { subtitle };
        let b_icon = if icon.len() > 256 { icon[..256].to_string() } else { icon };

        let state = match state_str.to_lowercase().as_str() {
            "expanded" => PillState::Expanded,
            "interactive" => PillState::Interactive,
            "volume" => PillState::OsdVolume,
            "media" => PillState::OsdMedia,
            _ => PillState::Compact,
        };
        let payload = LiveActivityPayload {
            id: b_id,
            title: b_title,
            subtitle: b_subtitle,
            icon: b_icon,
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
            "volume" => PillState::OsdVolume,
            "media" => PillState::OsdMedia,
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
            Ok(b) => b.max_queued(1024),
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
                    set_label: if model.is_playing { "🎵" } else { &model.payload.icon },
                    add_css_class: "morphic-pill-icon",
                },

                gtk::Label {
                    #[watch]
                    set_label: if model.is_playing { &model.media_title } else { &model.payload.title },
                    add_css_class: "morphic-pill-compact",
                },

                #[name = "compact_fft_canvas"]
                gtk::DrawingArea {
                    set_content_width: 24,
                    set_content_height: 14,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_visible: model.is_playing || model.volume > 0.0,
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

            // --- OSD VOLUME VIEW ---
            gtk::Box {
                #[watch]
                set_visible: model.state == PillState::OsdVolume,
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    #[watch]
                    set_label: if model.is_muted || model.volume <= 0.001 {
                        "🔇"
                    } else if model.volume < 0.33 {
                        "🔈"
                    } else if model.volume < 0.66 {
                        "🔉"
                    } else {
                        "🔊"
                    },
                    add_css_class: "morphic-pill-icon",
                },

                gtk::Label {
                    set_label: "Volume",
                    add_css_class: "morphic-pill-title",
                    set_valign: gtk::Align::Center,
                },

                gtk::ProgressBar {
                    #[watch]
                    set_fraction: model.volume,
                    add_css_class: "morphic-pill-progress",
                    set_valign: gtk::Align::Center,
                    set_width_request: 80,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("{}%", (model.volume * 100.0).round() as i32),
                    add_css_class: "morphic-pill-subtitle",
                    set_valign: gtk::Align::Center,
                },

                #[name = "volume_fft_canvas"]
                gtk::DrawingArea {
                    set_content_width: 44,
                    set_content_height: 18,
                    set_valign: gtk::Align::Center,
                },
            },

            // --- OSD MEDIA / INTERACTIVE VIEW ---
            gtk::Box {
                #[watch]
                set_visible: model.state == PillState::OsdMedia || model.state == PillState::Interactive,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 6,
                set_valign: gtk::Align::Center,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Label {
                        #[watch]
                        set_label: if model.is_playing { "🎵" } else { &model.payload.icon },
                        add_css_class: "morphic-pill-icon",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,

                        gtk::Label {
                            #[watch]
                            set_label: if !model.media_title.is_empty() { &model.media_title } else { &model.payload.title },
                            add_css_class: "morphic-pill-title",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Label {
                            #[watch]
                            set_label: if !model.media_artist.is_empty() { &model.media_artist } else { &model.payload.subtitle },
                            add_css_class: "morphic-pill-subtitle",
                            set_halign: gtk::Align::Start,
                        },
                    },

                    gtk::Button {
                        set_label: "✕",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::SetState(PillState::Compact),
                    },
                },

                #[name = "media_fft_canvas"]
                gtk::DrawingArea {
                    set_content_width: 220,
                    set_content_height: 22,
                    set_halign: gtk::Align::Center,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_halign: gtk::Align::Center,

                    gtk::Button {
                        set_label: "⏮",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::MediaPrev,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: if model.is_playing { "⏸" } else { "▶" },
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::MediaPlayPause,
                    },

                    gtk::Button {
                        set_label: "⏭",
                        add_css_class: "morphic-pill-btn",
                        connect_clicked => MorphicPillInput::MediaNext,
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

        let draw_state = Rc::new(RefCell::new(FftDrawState {
            tick: 0,
            volume: 0.5,
            is_playing: false,
        }));

        let model = MorphicPillModel {
            state: initial_state,
            payload: initial_payload,
            is_hovered: false,
            spring_state: spring_state.clone(),
            volume: 0.5,
            is_muted: false,
            media_title: "Ermete Audio".to_string(),
            media_artist: "System".to_string(),
            is_playing: false,
            draw_state: draw_state.clone(),
            auto_collapse_source: Rc::new(RefCell::new(None)),
        };

        let widgets = view_output!();

        // Connect FFT DrawingAreas
        let ds_compact = draw_state.clone();
        widgets.compact_fft_canvas.set_draw_func(move |area, cr, w, h| {
            let ds = ds_compact.borrow();
            draw_fft_waveform(area, cr, w, h, ds.tick, ds.volume, ds.is_playing);
        });

        let ds_volume = draw_state.clone();
        widgets.volume_fft_canvas.set_draw_func(move |area, cr, w, h| {
            let ds = ds_volume.borrow();
            draw_fft_waveform(area, cr, w, h, ds.tick, ds.volume, ds.is_playing);
        });

        let ds_media = draw_state.clone();
        widgets.media_fft_canvas.set_draw_func(move |area, cr, w, h| {
            let ds = ds_media.borrow();
            draw_fft_waveform(area, cr, w, h, ds.tick, ds.volume, ds.is_playing);
        });

        let spring_clone = spring_state;
        let ds_tick_clone = draw_state;
        let compact_canvas = widgets.compact_fft_canvas.clone();
        let volume_canvas = widgets.volume_fft_canvas.clone();
        let media_canvas = widgets.media_fft_canvas.clone();

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

            {
                let mut ds = ds_tick_clone.borrow_mut();
                ds.tick = ds.tick.wrapping_add(1);
            }
            compact_canvas.queue_draw();
            volume_canvas.queue_draw();
            media_canvas.queue_draw();

            glib::ControlFlow::Continue
        });

        let sender_clone = sender.clone();
        spawn_zbus_listener(sender_clone);

        // Subscribe to OSD events for Volume
        let sender_osd = sender.clone();
        crate::ui::viewmodel::OsdViewModel::subscribe(move |event| {
            if let crate::ui::viewmodel::OsdEvent::Volume(v) = event {
                sender_osd.input(MorphicPillInput::VolumeChanged(v));
            }
        });

        // Subscribe to MPRIS events for Media state
        let sender_mpris = sender;
        let mut mpris_rx = crate::ipc::system_proxies::get_mpris_bus().subscribe();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(ev) = mpris_rx.recv().await {
                if let crate::ipc::types::MprisEvent::MprisUpdated(opt_state) = ev {
                    if let Some(mstate) = opt_state {
                        let is_playing = mstate.status.to_lowercase().contains("play");
                        sender_mpris.input(MorphicPillInput::MediaStateChanged {
                            title: mstate.title,
                            artist: mstate.artist,
                            is_playing,
                        });
                    }
                }
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
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
                    PillState::Compact => {
                        if self.is_playing {
                            PillState::OsdMedia
                        } else {
                            PillState::Expanded
                        }
                    }
                    PillState::Expanded => PillState::Interactive,
                    PillState::Interactive | PillState::OsdMedia | PillState::OsdVolume => PillState::Compact,
                };
                self.state = next_state;
                self.spring_state.borrow_mut().set_targets_for_state(next_state);
            }
            MorphicPillInput::HoverChanged(is_hovered) => {
                self.is_hovered = is_hovered;
                if is_hovered && self.state == PillState::Compact {
                    let target = if self.is_playing { PillState::OsdMedia } else { PillState::Expanded };
                    self.state = target;
                    self.spring_state.borrow_mut().set_targets_for_state(target);
                } else if !is_hovered && (self.state == PillState::Expanded || self.state == PillState::OsdMedia) {
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
            MorphicPillInput::VolumeChanged(v) => {
                let clamped = v.clamp(0.0, 1.0);
                self.volume = clamped;
                self.draw_state.borrow_mut().volume = clamped;
                self.state = PillState::OsdVolume;
                self.spring_state.borrow_mut().set_targets_for_state(PillState::OsdVolume);

                if let Some(src) = self.auto_collapse_source.borrow_mut().take() {
                    src.remove();
                }

                let sender_clone = sender.clone();
                let src_ref = self.auto_collapse_source.clone();
                *self.auto_collapse_source.borrow_mut() = Some(glib::timeout_add_local_once(
                    Duration::from_millis(2000),
                    move || {
                        sender_clone.input(MorphicPillInput::AutoCollapse);
                        *src_ref.borrow_mut() = None;
                    },
                ));
            }
            MorphicPillInput::MuteToggled(muted) => {
                self.is_muted = muted;
            }
            MorphicPillInput::MediaStateChanged { title, artist, is_playing } => {
                self.media_title = title;
                self.media_artist = artist;
                self.is_playing = is_playing;
                self.draw_state.borrow_mut().is_playing = is_playing;
            }
            MorphicPillInput::MediaPlayPause => {
                self.is_playing = !self.is_playing;
                self.draw_state.borrow_mut().is_playing = self.is_playing;
                glib::MainContext::default().spawn_local(async move {
                    let mpris = crate::ipc::mpris::get_mpris_controller();
                    let _ = mpris.player_command("PlayPause").await;
                });
            }
            MorphicPillInput::MediaNext => {
                glib::MainContext::default().spawn_local(async move {
                    let mpris = crate::ipc::mpris::get_mpris_controller();
                    let _ = mpris.player_command("Next").await;
                });
            }
            MorphicPillInput::MediaPrev => {
                glib::MainContext::default().spawn_local(async move {
                    let mpris = crate::ipc::mpris::get_mpris_controller();
                    let _ = mpris.player_command("Previous").await;
                });
            }
            MorphicPillInput::AutoCollapse => {
                if self.state == PillState::OsdVolume {
                    let next_state = if self.is_playing {
                        PillState::OsdMedia
                    } else {
                        PillState::Compact
                    };
                    self.state = next_state;
                    self.spring_state.borrow_mut().set_targets_for_state(next_state);
                }
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

        s.set_targets_for_state(PillState::OsdVolume);
        assert_eq!(s.width_spring.target, 290.0);
        assert_eq!(s.height_spring.target, 46.0);

        s.set_targets_for_state(PillState::OsdMedia);
        assert_eq!(s.width_spring.target, 360.0);
        assert_eq!(s.height_spring.target, 90.0);

        s.set_targets_for_state(PillState::Compact);
        assert_eq!(s.width_spring.target, 120.0);
        assert_eq!(s.height_spring.target, 28.0);
    }
}
