use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EbpfMode {
    AiInferred,
    GamingLowLatency,
    EcoSaver,
    MaxThroughput,
}

impl EbpfMode {
    pub fn label(&self) -> &'static str {
        match self {
            EbpfMode::AiInferred => "⚡ AI-Inferred",
            EbpfMode::GamingLowLatency => "🎮 Low Latency",
            EbpfMode::EcoSaver => "🍃 Eco Saver",
            EbpfMode::MaxThroughput => "🚀 Max Throughput",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            EbpfMode::AiInferred => "RL-driven kernel thread scheduling in ring-0",
            EbpfMode::GamingLowLatency => "Zero-jitter thread prioritization & eBPF lock bypass",
            EbpfMode::EcoSaver => "Dynamic CPU frequency capping & eBPF C-state tuning",
            EbpfMode::MaxThroughput => "High bandwidth ring buffer & I/O queue optimization",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EbpfModuleData {
    pub current_mode: EbpfMode,
    pub ring0_tracepoints_active: usize,
    pub latency_reduction_pct: u8,
    pub context_switches_saved: u64,
    pub status_text: String,
}

impl Default for EbpfModuleData {
    fn default() -> Self {
        Self {
            current_mode: EbpfMode::AiInferred,
            ring0_tracepoints_active: 128,
            latency_reduction_pct: 42,
            context_switches_saved: 1_284_900,
            status_text: "Autonomous eBPF Sched Active".to_string(),
        }
    }
}

pub fn build_ebpf_widget(data: &EbpfModuleData) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // Telemetry Badge Row
    let telemetry_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let telemetry_lbl = Label::builder()
        .label(&format!(
            "⚡ eBPF Ring-0: {} Tracepoints | Latency: -{}% | CS Saved: {}",
            data.ring0_tracepoints_active,
            data.latency_reduction_pct,
            data.context_switches_saved
        ))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();

    telemetry_box.append(&telemetry_lbl);

    // Mode Selector Buttons 2x2 Grid
    let mode_grid = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    let row1 = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).homogeneous(true).build();
    let row2 = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).homogeneous(true).build();

    let modes = [
        (EbpfMode::AiInferred, &row1),
        (EbpfMode::GamingLowLatency, &row1),
        (EbpfMode::EcoSaver, &row2),
        (EbpfMode::MaxThroughput, &row2),
    ];

    let current_mode = data.current_mode.clone();

    for (mode, parent_row) in modes {
        let is_active = mode == current_mode;
        let btn = Button::builder()
            .label(mode.label())
            .css_classes(if is_active {
                vec!["cc-quick-btn", "cc-btn-active"]
            } else {
                vec!["cc-quick-btn"]
            })
            .tooltip_text(mode.description())
            .build();

        let mode_clone = mode.clone();
        btn.connect_clicked(move |_| {
            tracing::info!("Switching eBPF performance mode to {:?}", mode_clone);
        });

        parent_row.append(&btn);
    }

    mode_grid.append(&row1);
    mode_grid.append(&row2);

    // Kernel Forge Button
    let forge_btn = Button::builder()
        .label("🧬 Hardware-Tailored Kernel Optimization")
        .css_classes(["cc-quick-btn"])
        .tooltip_text("Gentoo-style hardware tailored kernel compilation with LTO & AutoFDO")
        .build();

    forge_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::OptimizeKernel);
    });

    container.append(&telemetry_box);
    container.append(&mode_grid);
    container.append(&forge_btn);
    container
}
