use gtk4::glib;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

#[derive(Debug, Clone)]
pub struct OobeModel {
    current_step: usize,
    selected_language: String,
    selected_keyboard: String,
    telemetry_opt_in: bool,
    crash_reports_opt_in: bool,
    ebpf_ai_opt_in: bool,
}

#[derive(Debug)]
pub enum OobeMsg {
    NextStep,
    PrevStep,
    SelectLanguage(String),
    SelectKeyboard(String),
    ToggleTelemetry(bool),
    ToggleCrashReports(bool),
    ToggleEbpfAi(bool),
    Finish,
}

impl OobeModel {
    fn step_name(&self) -> &'static str {
        match self.current_step {
            0 => "step_welcome",
            1 => "step_lang",
            2 => "step_telemetry",
            _ => "step_completion",
        }
    }
}

fn load_custom_oobe_css() {
    let provider = gtk4::CssProvider::new();
    let css = r#"
        .oobe-root {
            background-color: #0f111a;
            color: #c0caf5;
            font-family: sans-serif;
        }
        .oobe-header {
            background: rgba(26, 27, 38, 0.6);
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            padding: 16px 24px;
        }
        .oobe-card {
            background: rgba(30, 32, 48, 0.85);
            border: 1px solid rgba(122, 162, 247, 0.25);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
            min-width: 540px;
        }
        .oobe-subcard {
            background: rgba(20, 21, 33, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 16px;
        }
        .oobe-privacy-banner {
            background: rgba(122, 162, 247, 0.1);
            border: 1px solid rgba(122, 162, 247, 0.3);
            border-radius: 10px;
            padding: 12px 16px;
        }
        .oobe-step-badge {
            background: rgba(255, 255, 255, 0.08);
            color: #a9b1d6;
            border-radius: 12px;
            padding: 6px 14px;
            font-size: 13px;
        }
        .oobe-step-badge-active {
            background: #7aa2f7;
            color: #15161e;
            border-radius: 12px;
            padding: 6px 14px;
            font-weight: bold;
            font-size: 13px;
        }
        .oobe-primary-btn {
            background: linear-gradient(135deg, #7aa2f7 0%, #bb9af7 100%);
            color: #15161e;
            font-weight: bold;
            font-size: 15px;
            border-radius: 12px;
            padding: 12px 28px;
            border: none;
        }
        .oobe-sec-btn {
            background: rgba(255, 255, 255, 0.08);
            color: #c0caf5;
            border-radius: 12px;
            padding: 12px 24px;
            border: 1px solid rgba(255, 255, 255, 0.15);
        }
    "#;
    provider.load_from_data(css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 100,
        );
    }
}

#[relm4::component(pub)]
impl SimpleComponent for OobeModel {
    type Init = ();
    type Input = OobeMsg;
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Ermete OS - Out Of Box Experience"),
            set_fullscreened: true,
            set_decorated: false,
            add_css_class: "oobe-root",

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,

                // Header Banner
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_margin_all: 24,
                    set_halign: gtk4::Align::Fill,
                    add_css_class: "oobe-header",

                    gtk4::Label {
                        set_markup: "<span size='20000' weight='bold' foreground='#7AA2F7'>ERMETE</span><span size='20000' weight='light' foreground='#A9B1D6'> OS</span>",
                        set_halign: gtk4::Align::Start,
                        set_hexpand: true,
                    },

                    // Step Indicator Dots / Badges
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk4::Align::End,

                        gtk4::Label {
                            set_label: "1. Benvenuto",
                            add_css_class: if model.current_step == 0 { "oobe-step-badge-active" } else { "oobe-step-badge" },
                        },
                        gtk4::Label {
                            set_label: "2. Lingua",
                            add_css_class: if model.current_step == 1 { "oobe-step-badge-active" } else { "oobe-step-badge" },
                        },
                        gtk4::Label {
                            set_label: "3. Telemetria",
                            add_css_class: if model.current_step == 2 { "oobe-step-badge-active" } else { "oobe-step-badge" },
                        },
                        gtk4::Label {
                            set_label: "4. Fine",
                            add_css_class: if model.current_step == 3 { "oobe-step-badge-active" } else { "oobe-step-badge" },
                        },
                    }
                },

                // Main Stack Container
                #[name = "wizard_stack"]
                gtk4::Stack {
                    set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
                    set_transition_duration: 300,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_visible_child_name: model.step_name(),

                    // Step 1: Welcome
                    #[name = "step_welcome"]
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center,
                        set_spacing: 24,
                        set_margin_all: 32,
                        add_css_class: "oobe-card",

                        gtk4::Label {
                            set_markup: "<span size='32000' weight='bold' foreground='#7AA2F7'>Benvenuto in Ermete OS</span>",
                            set_justify: gtk4::Justification::Center,
                        },
                        gtk4::Label {
                            set_markup: "<span size='15000' alpha='80%'>L'ecosistema Cloud-Native, Zero-Trust e AI-Driven di nuova generazione.</span>",
                            set_justify: gtk4::Justification::Center,
                        },
                        gtk4::Label {
                            set_markup: "<span size='12000' foreground='#BB9AF7'>✦ System State: Level 5 Singularity Ready ✦</span>",
                            set_justify: gtk4::Justification::Center,
                        },

                        // Feature Highlights
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_margin_top: 16,
                            set_margin_bottom: 16,

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 8,
                                add_css_class: "oobe-subcard",
                                gtk4::Label { set_markup: "<b>🔒 Zero-Trust Kernel</b>" },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Isolamento MicroVM cROSVM</span>" },
                            },
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 8,
                                add_css_class: "oobe-subcard",
                                gtk4::Label { set_markup: "<b>🌌 Post-Quantum Mesh</b>" },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Kyber-1024 &amp; Dilithium5</span>" },
                            },
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 8,
                                add_css_class: "oobe-subcard",
                                gtk4::Label { set_markup: "<b>🧠 eBPF AI System</b>" },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Auto-scheduling in Ring-0</span>" },
                            },
                        },

                        gtk4::Button {
                            set_label: "Inizia Configurazione →",
                            add_css_class: "oobe-primary-btn",
                            connect_clicked => OobeMsg::NextStep,
                        }
                    },

                    // Step 2: Language & Keyboard Selection
                    #[name = "step_lang"]
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center,
                        set_spacing: 20,
                        set_margin_all: 32,
                        add_css_class: "oobe-card",

                        gtk4::Label {
                            set_markup: "<span size='24000' weight='bold'>Selezione Lingua e Tastiera</span>",
                        },
                        gtk4::Label {
                            set_markup: "<span size='13000' alpha='70%'>Scegli la tua lingua principale e la configurazione della tastiera.</span>",
                        },

                        // Language Selection
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 8,
                            set_halign: gtk4::Align::Fill,

                            gtk4::Label {
                                set_markup: "<b>Lingua di Sistema:</b>",
                                set_halign: gtk4::Align::Start,
                            },

                            gtk4::DropDown::from_strings(&[
                                "🇮🇹 Italiano (Italia)",
                                "🇺🇸 English (United States)",
                                "🇪🇸 Español (España)",
                                "🇩🇪 Deutsch (Deutschland)",
                                "🇫🇷 Français (France)"
                            ]) {
                                connect_selected_notify[sender] => move |dropdown| {
                                    if let Some(str_val) = dropdown.selected_item().and_then(|obj| obj.downcast::<gtk4::StringObject>().ok()) {
                                        sender.input(OobeMsg::SelectLanguage(str_val.string().to_string()));
                                    }
                                }
                            }
                        },

                        // Keyboard Selection
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 8,
                            set_halign: gtk4::Align::Fill,

                            gtk4::Label {
                                set_markup: "<b>Layout Tastiera:</b>",
                                set_halign: gtk4::Align::Start,
                            },

                            gtk4::DropDown::from_strings(&[
                                "it - Italiano (QWERTY)",
                                "us - English (QWERTY)",
                                "es - Español (QWERTY)",
                                "de - Deutsch (QWERTZ)",
                                "fr - Français (AZERTY)"
                            ]) {
                                connect_selected_notify[sender] => move |dropdown| {
                                    if let Some(str_val) = dropdown.selected_item().and_then(|obj| obj.downcast::<gtk4::StringObject>().ok()) {
                                        sender.input(OobeMsg::SelectKeyboard(str_val.string().to_string()));
                                    }
                                }
                            }
                        },

                        // Keyboard Test Area
                        gtk4::Entry {
                            set_placeholder_text: Some("Digita qui per testare il layout..."),
                            set_width_request: 360,
                        },

                        // Nav buttons
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_margin_top: 12,

                            gtk4::Button {
                                set_label: "← Indietro",
                                add_css_class: "oobe-sec-btn",
                                connect_clicked => OobeMsg::PrevStep,
                            },
                            gtk4::Button {
                                set_label: "Avanti →",
                                add_css_class: "oobe-primary-btn",
                                connect_clicked => OobeMsg::NextStep,
                            }
                        }
                    },

                    // Step 3: Telemetry & Privacy Opt-in
                    #[name = "step_telemetry"]
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center,
                        set_spacing: 20,
                        set_margin_all: 32,
                        add_css_class: "oobe-card",

                        gtk4::Label {
                            set_markup: "<span size='24000' weight='bold'>Telemetria &amp; Privacy Opt-in</span>",
                        },
                        gtk4::Label {
                            set_markup: "<span size='13000' alpha='70%'>In Ermete OS la tua privacy è protetta dall'architettura Zero-Trust.</span>",
                        },

                        // Toggle 1
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_halign: gtk4::Align::Fill,

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_hexpand: true,
                                set_halign: gtk4::Align::Start,
                                gtk4::Label { set_markup: "<b>Telemetria Anonima di Sistema</b>", set_halign: gtk4::Align::Start },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Metriche anonime di prestazioni crittografate.</span>", set_halign: gtk4::Align::Start },
                            },
                            gtk4::Switch {
                                set_active: model.telemetry_opt_in,
                                connect_state_set[sender] => move |_, state| {
                                    sender.input(OobeMsg::ToggleTelemetry(state));
                                    glib::Propagation::Proceed
                                }
                            }
                        },

                        // Toggle 2
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_halign: gtk4::Align::Fill,

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_hexpand: true,
                                set_halign: gtk4::Align::Start,
                                gtk4::Label { set_markup: "<b>Crash Reports Automatici</b>", set_halign: gtk4::Align::Start },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Report di errore per l'auto-healing del sistema.</span>", set_halign: gtk4::Align::Start },
                            },
                            gtk4::Switch {
                                set_active: model.crash_reports_opt_in,
                                connect_state_set[sender] => move |_, state| {
                                    sender.input(OobeMsg::ToggleCrashReports(state));
                                    glib::Propagation::Proceed
                                }
                            }
                        },

                        // Toggle 3
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_halign: gtk4::Align::Fill,

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_hexpand: true,
                                set_halign: gtk4::Align::Start,
                                gtk4::Label { set_markup: "<b>eBPF AI Feedback Loop</b>", set_halign: gtk4::Align::Start },
                                gtk4::Label { set_markup: "<span size='10000' alpha='70%'>Apprendimento locale per la schedulazione ottimizzata.</span>", set_halign: gtk4::Align::Start },
                            },
                            gtk4::Switch {
                                set_active: model.ebpf_ai_opt_in,
                                connect_state_set[sender] => move |_, state| {
                                    sender.input(OobeMsg::ToggleEbpfAi(state));
                                    glib::Propagation::Proceed
                                }
                            }
                        },

                        // Privacy Card
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 12,
                            add_css_class: "oobe-privacy-banner",
                            set_margin_top: 8,

                            gtk4::Label {
                                set_markup: "🛡️ <i>Nessun dato personale viene mai raccolto o condiviso all'esterno.</i>",
                            }
                        },

                        // Nav buttons
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_margin_top: 12,

                            gtk4::Button {
                                set_label: "← Indietro",
                                add_css_class: "oobe-sec-btn",
                                connect_clicked => OobeMsg::PrevStep,
                            },
                            gtk4::Button {
                                set_label: "Avanti →",
                                add_css_class: "oobe-primary-btn",
                                connect_clicked => OobeMsg::NextStep,
                            }
                        }
                    },

                    // Step 4: Completion
                    #[name = "step_completion"]
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center,
                        set_spacing: 24,
                        set_margin_all: 32,
                        add_css_class: "oobe-card",

                        gtk4::Label {
                            set_markup: "<span size='32000'>🎉</span>",
                        },
                        gtk4::Label {
                            set_markup: "<span size='28000' weight='bold' foreground='#9ECE6A'>Tutto Pronto!</span>",
                        },
                        gtk4::Label {
                            set_markup: "<span size='14000' alpha='80%'>La configurazione iniziale di Ermete OS è stata completata con successo.</span>",
                        },

                        // Summary Box
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 8,
                            add_css_class: "oobe-subcard",
                            set_width_request: 380,

                            gtk4::Label {
                                set_markup: &format!("<b>Lingua:</b> {}", model.selected_language),
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Label {
                                set_markup: &format!("<b>Tastiera:</b> {}", model.selected_keyboard),
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Label {
                                set_markup: &format!("<b>Telemetria:</b> {}", if model.telemetry_opt_in { "Attiva" } else { "Disattiva" }),
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Label {
                                set_markup: &format!("<b>Crash Reports:</b> {}", if model.crash_reports_opt_in { "Attivi" } else { "Disattivi" }),
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Label {
                                set_markup: &format!("<b>eBPF AI Loop:</b> {}", if model.ebpf_ai_opt_in { "Attivo" } else { "Disattivo" }),
                                set_halign: gtk4::Align::Start,
                            },
                        },

                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 16,
                            set_margin_top: 12,

                            gtk4::Button {
                                set_label: "← Indietro",
                                add_css_class: "oobe-sec-btn",
                                connect_clicked => OobeMsg::PrevStep,
                            },
                            gtk4::Button {
                                set_label: "🚀 Inizia a usare Ermete OS",
                                add_css_class: "oobe-primary-btn",
                                connect_clicked => OobeMsg::Finish,
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        ermete_style::load_glass_theme();
        load_custom_oobe_css();

        let model = OobeModel {
            current_step: 0,
            selected_language: "🇮🇹 Italiano (Italia)".to_string(),
            selected_keyboard: "it - Italiano (QWERTY)".to_string(),
            telemetry_opt_in: true,
            crash_reports_opt_in: true,
            ebpf_ai_opt_in: true,
        };

        let widgets = view_output!();

        widgets.wizard_stack.add_named(&widgets.step_welcome, Some("step_welcome"));
        widgets.wizard_stack.add_named(&widgets.step_lang, Some("step_lang"));
        widgets.wizard_stack.add_named(&widgets.step_telemetry, Some("step_telemetry"));
        widgets.wizard_stack.add_named(&widgets.step_completion, Some("step_completion"));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            OobeMsg::NextStep => {
                if self.current_step < 3 {
                    self.current_step += 1;
                }
            }
            OobeMsg::PrevStep => {
                if self.current_step > 0 {
                    self.current_step -= 1;
                }
            }
            OobeMsg::SelectLanguage(lang) => {
                self.selected_language = lang;
            }
            OobeMsg::SelectKeyboard(kb) => {
                self.selected_keyboard = kb;
            }
            OobeMsg::ToggleTelemetry(val) => {
                self.telemetry_opt_in = val;
            }
            OobeMsg::ToggleCrashReports(val) => {
                self.crash_reports_opt_in = val;
            }
            OobeMsg::ToggleEbpfAi(val) => {
                self.ebpf_ai_opt_in = val;
            }
            OobeMsg::Finish => {
                println!("[ERMETE-OOBE] Configuration finished! Quitting wizard...");
                relm4::main_application().quit();
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("os.ermete.oobe");
    app.run::<OobeModel>(());
}
