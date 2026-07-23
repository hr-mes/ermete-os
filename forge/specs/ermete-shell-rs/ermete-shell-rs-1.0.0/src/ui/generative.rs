use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct IntentPayload {
    pub id: String,
    pub action: String,
    pub ui_elements: Vec<UiElement>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum UiElement {
    Button { label: String, action_id: String },
    Slider { min: f64, max: f64, value: f64, action_id: String },
    Label { text: String },
}

pub struct GenerativeUiEngine {
    container: gtk::Box,
}

impl GenerativeUiEngine {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
        Self { container }
    }

    pub fn container(&self) -> &gtk::Box {
        &self.container
    }

    pub fn handle_intent(&self, payload_json: &str) {
        match serde_json::from_str::<IntentPayload>(payload_json) {
            Ok(payload) => {
                self.build_ui(payload.ui_elements);
            }
            Err(e) => {
                eprintln!("Failed to parse intent payload: {}", e);
            }
        }
    }

    fn build_ui(&self, elements: Vec<UiElement>) {
        // Clear existing children
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        for element in elements {
            match element {
                UiElement::Button { label, action_id } => {
                    let btn = gtk::Button::builder().label(&label).build();
                    let action_id_clone = action_id.clone();
                    btn.connect_clicked(move |_| {
                        println!("Action triggered: {}", action_id_clone);
                        // Send response back to `ermete-daemon-rs`
                    });
                    self.container.append(&btn);
                }
                UiElement::Slider { min, max, value, action_id } => {
                    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, 1.0);
                    scale.set_value(value);
                    let action_id_clone = action_id.clone();
                    scale.connect_value_changed(move |s| {
                        println!("Slider '{}' changed to {}", action_id_clone, s.value());
                        // Send response back to `ermete-daemon-rs`
                    });
                    self.container.append(&scale);
                }
                UiElement::Label { text } => {
                    let lbl = gtk::Label::new(Some(&text));
                    self.container.append(&lbl);
                }
            }
        }
    }
}
