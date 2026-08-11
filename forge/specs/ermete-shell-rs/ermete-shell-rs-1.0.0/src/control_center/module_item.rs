use relm4::gtk;
use relm4::factory::{FactoryComponent, FactorySender};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Orientation};

use super::network::{NetworkModuleData, build_network_widget};
use super::audio::{AudioModuleData, build_audio_widget};
use super::display::{DisplayModuleData, build_display_widget};
use super::ebpf::{EbpfModuleData, build_ebpf_widget};

#[derive(Debug, Clone)]
pub enum ModuleContent {
    Network(NetworkModuleData),
    Audio(AudioModuleData),
    Display(DisplayModuleData),
    Ebpf(EbpfModuleData),
}

#[derive(Debug, Clone)]
pub struct CcModuleItem {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub content: ModuleContent,
}

impl CcModuleItem {
    pub fn build_body(&self) -> GtkBox {
        match &self.content {
            ModuleContent::Network(data) => build_network_widget(data),
            ModuleContent::Audio(data) => build_audio_widget(data),
            ModuleContent::Display(data) => build_display_widget(data),
            ModuleContent::Ebpf(data) => build_ebpf_widget(data),
        }
    }
}

#[derive(Debug)]
pub enum CcModuleMsg {
    UpdateContent(ModuleContent),
}

#[relm4::factory(pub)]
impl FactoryComponent for CcModuleItem {
    type Init = CcModuleItem;
    type Input = CcModuleMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = GtkBox;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            add_css_class: "liquid-surface",
            add_css_class: "cc-module-card",
            add_css_class: "glassmorphism",
            set_spacing: 8,
            set_margin_start: 12,
            set_margin_end: 12,
            set_margin_top: 6,
            set_margin_bottom: 6,

            // Header Box
            gtk::Box {
                set_orientation: Orientation::Horizontal,
                set_spacing: 10,
                set_valign: Align::Center,

                gtk::Label {
                    #[watch]
                    set_label: &self.icon,
                    add_css_class: "cc-module-icon",
                },

                gtk::Label {
                    #[watch]
                    set_label: &self.title,
                    add_css_class: "cc-label-title",
                    set_halign: Align::Start,
                    set_hexpand: true,
                },
            },

            append = &self.build_body(),
        }
    }

    fn init_model(init: Self::Init, _index: &relm4::factory::DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            CcModuleMsg::UpdateContent(new_content) => {
                self.content = new_content;
            }
        }
    }
}
