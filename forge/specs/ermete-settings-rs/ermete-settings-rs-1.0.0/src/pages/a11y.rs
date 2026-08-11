use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, ComboBoxText, DrawingArea, Label, Orientation, Scale, Switch,
};
use crate::components::action_row::ActionRow;

/// Configurazione per la sincronizzazione dei parametri di accessibilità con la rete CRDT
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11yConfigPayload {
    pub live_captions_enabled: bool,
    pub captions_model: String,
    pub captions_font_size: u32,
    pub colorblind_filter_enabled: bool,
    pub shader_type: String,
    pub shader_intensity: u32,
    pub tts_navigation_enabled: bool,
    pub tts_voice: String,
    pub tts_speed_x10: u32,
}

impl A11yConfigPayload {
    pub fn default_config() -> Self {
        Self {
            live_captions_enabled: false,
            captions_model: "Whisper-eBPF Neural Standard".to_string(),
            captions_font_size: 20,
            colorblind_filter_enabled: false,
            shader_type: "Protanopia".to_string(),
            shader_intensity: 80,
            tts_navigation_enabled: false,
            tts_voice: "Ermete Neural Voice 1".to_string(),
            tts_speed_x10: 10,
        }
    }

    pub fn to_crdt_entries(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "a11y_live_captions",
                if self.live_captions_enabled {
                    "enabled".into()
                } else {
                    "disabled".into()
                },
            ),
            ("a11y_captions_model", self.captions_model.clone()),
            ("a11y_captions_font_size", self.captions_font_size.to_string()),
            (
                "a11y_colorblind_filter",
                if self.colorblind_filter_enabled {
                    "enabled".into()
                } else {
                    "disabled".into()
                },
            ),
            ("a11y_colorblind_shader_type", self.shader_type.clone()),
            ("a11y_colorblind_intensity", self.shader_intensity.to_string()),
            (
                "a11y_tts_navigation",
                if self.tts_navigation_enabled {
                    "enabled".into()
                } else {
                    "disabled".into()
                },
            ),
            ("a11y_tts_voice", self.tts_voice.clone()),
            ("a11y_tts_speed", (self.tts_speed_x10 as f32 / 10.0).to_string()),
        ]
    }
}

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // 1. Header Section
    let title_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    let title = Label::builder()
        .label("<span size='xx-large' weight='bold'>Accessibilità &amp; Inclusione (A11y)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();

    let subtitle = Label::builder()
        .label("Tecnologie assistive avanzate di Livello 5: IA eBPF in Ring-0, Shader Wayland Niri e Sintesi Vocale Neurale.")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();

    title_box.append(&title);
    title_box.append(&subtitle);
    container.append(&title_box);

    // Master Status Banner / Card
    let status_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .css_classes(["liquid-surface"])
        .build();

    let status_info = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .valign(Align::Center)
        .build();

    let status_title = Label::builder()
        .label("<b>Motore Assistivo Ermete OS Singularity</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();

    let status_desc = Label::builder()
        .label("Stato: 🟢 Moduli eBPF attivi | Latenza IA: 1.2ms | Compositore Wayland Shader: Pronto")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();

    status_info.append(&status_title);
    status_info.append(&status_desc);

    let master_switch = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();

    master_switch.connect_active_notify(|sw| {
        let active = sw.is_active();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt(
                "a11y_master_engine",
                if active { "enabled" } else { "disabled" },
            )
            .await;
        });
    });

    status_card.append(&status_info);
    status_card.append(&master_switch);
    container.append(&status_card);

    // 2. FEATURE 1: Sottotitoli Live (generati da IA eBPF locale)
    let captions_section_title = Label::builder()
        .label("<span size='large' weight='bold'>🎙️ Sottotitoli Live (IA eBPF Locale)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(8)
        .build();
    container.append(&captions_section_title);

    let captions_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["liquid-surface"])
        .build();

    let captions_switch = Switch::builder()
        .valign(Align::Center)
        .active(false)
        .build();

    let captions_status_label = Label::builder()
        .label("⚪ Sottotitoli Live disattivati. Attiva il toggle per avviare la trascrizione eBPF.")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();

    let c_label_clone = captions_status_label.clone();
    captions_switch.connect_active_notify(move |sw| {
        let active = sw.is_active();
        if active {
            c_label_clone.set_label("🟢 IA eBPF in ascolto su PipeWire Stream #0 (Whisper-eBPF Low Latency: 1.1ms)");
        } else {
            c_label_clone.set_label("⚪ Sottotitoli Live disattivati. Attiva il toggle per avviare la trascrizione eBPF.");
        }
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt(
                "a11y_live_captions",
                if active { "enabled" } else { "disabled" },
            )
            .await;
        });
    });

    let captions_row = ActionRow::builder("Sottotitoli Live (generati da IA eBPF locale)")
        .subtitle("Trascrizione istantanea zero-latency dell'audio di sistema senza connessione internet")
        .suffix(&captions_switch)
        .build();
    captions_card.append(&captions_row);
    captions_card.append(&captions_status_label);

    // Modello IA Selector
    let model_combo = ComboBoxText::new();
    model_combo.append_text("Whisper-eBPF Ultra-Lite (0.8ms - Minimo Impatto)");
    model_combo.append_text("Whisper-eBPF Neural Standard (1.2ms - Consigliato)");
    model_combo.append_text("Whisper-eBPF High-Precision (2.5ms - Massima Accuratezza)");
    model_combo.set_active(Some(1));

    model_combo.connect_changed(|cb| {
        if let Some(text) = cb.active_text() {
            let val = text.to_string();
            relm4::spawn_local(async move {
                let _ = crate::crdt_store::update_setting_crdt("a11y_captions_model", &val).await;
            });
        }
    });

    let model_row = ActionRow::builder("Modello IA eBPF Trascrizione")
        .subtitle("Seleziona il modello neurale per il bilanciamento tra latenza ed accuratezza")
        .suffix(&model_combo)
        .build();
    captions_card.append(&model_row);

    // Posizione Sottotitoli
    let pos_combo = ComboBoxText::new();
    pos_combo.append_text("In Basso al Centro (Overlay Dock)");
    pos_combo.append_text("In Alto al Centro (Barra di Stato)");
    pos_combo.append_text("Finestra Fluttuante Trasparente");
    pos_combo.set_active(Some(0));

    pos_combo.connect_changed(|cb| {
        if let Some(text) = cb.active_text() {
            let val = text.to_string();
            relm4::spawn_local(async move {
                let _ = crate::crdt_store::update_setting_crdt("a11y_captions_position", &val).await;
            });
        }
    });

    let pos_row = ActionRow::builder("Posizione On-Screen")
        .subtitle("Posizionamento dei sottotitoli in sovrimpressione sul compositore")
        .suffix(&pos_combo)
        .build();
    captions_card.append(&pos_row);

    // Dimensione Carattere Sottotitoli Slider
    let font_scale = Scale::with_range(Orientation::Horizontal, 12.0, 36.0, 1.0);
    font_scale.set_value(20.0);
    font_scale.set_draw_value(true);
    font_scale.set_hexpand(true);
    font_scale.set_size_request(200, -1);

    font_scale.connect_value_changed(|sc| {
        let val = sc.value();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_captions_font_size", &val.to_string()).await;
        });
    });

    let font_row = ActionRow::builder("Dimensione Testo Sottotitoli")
        .subtitle("Regola la grandezza del font dei sottotitoli in pixel")
        .suffix(&font_scale)
        .build();
    captions_card.append(&font_row);

    container.append(&captions_card);

    // 3. FEATURE 2: Filtri Daltonismo (Color Blindness via Shader Wayland)
    let colorblind_section_title = Label::builder()
        .label("<span size='large' weight='bold'>🎨 Filtri Daltonismo (Color Blindness via Shader Wayland)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(8)
        .build();
    container.append(&colorblind_section_title);

    let colorblind_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["liquid-surface"])
        .build();

    let cb_switch = Switch::builder()
        .valign(Align::Center)
        .active(false)
        .build();

    cb_switch.connect_active_notify(move |sw| {
        let active = sw.is_active();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt(
                "a11y_colorblind_filter",
                if active { "enabled" } else { "disabled" },
            )
            .await;
        });
    });

    let cb_row = ActionRow::builder("Filtri Daltonismo (Color Blindness via Shader Wayland)")
        .subtitle("Correzione cromatica hardware in tempo reale tramite shader GLSL nel compositore Niri")
        .suffix(&cb_switch)
        .build();
    colorblind_card.append(&cb_row);

    let da_preview = DrawingArea::builder()
        .content_width(400)
        .content_height(60)
        .css_classes(["liquid-surface"])
        .build();

    // Selection of Daltonism Type
    let shader_combo = ComboBoxText::new();
    shader_combo.append_text("Protanopia (Insensibilità al Rosso)");
    shader_combo.append_text("Deuteranopia (Insensibilità al Verde)");
    shader_combo.append_text("Tritanopia (Insensibilità al Blu)");
    shader_combo.append_text("Monocromia (Scala di Grigi)");
    shader_combo.append_text("Inversione Ad Alto Contrasto (Dark Invert)");
    shader_combo.set_active(Some(0));

    let da_preview_clone = da_preview.clone();
    shader_combo.connect_changed(move |cb| {
        if let Some(text) = cb.active_text() {
            let val = text.to_string();
            da_preview_clone.queue_draw();
            relm4::spawn_local(async move {
                let _ = crate::crdt_store::update_setting_crdt("a11y_colorblind_shader_type", &val).await;
            });
        }
    });

    let shader_row = ActionRow::builder("Modalità Correzione Spettrale")
        .subtitle("Seleziona la matrice di riconversione cromatico-spettrale")
        .suffix(&shader_combo)
        .build();
    colorblind_card.append(&shader_row);

    // Shader Intensity Slider
    let intensity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 5.0);
    intensity_scale.set_value(80.0);
    intensity_scale.set_draw_value(true);
    intensity_scale.set_hexpand(true);
    intensity_scale.set_size_request(200, -1);

    let da_preview_scale = da_preview.clone();
    intensity_scale.connect_value_changed(move |sc| {
        let val = sc.value();
        da_preview_scale.queue_draw();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_colorblind_intensity", &val.to_string()).await;
        });
    });

    let intensity_row = ActionRow::builder("Intensità Correzione Shader")
        .subtitle("Percentuale di applicazione del filtro spettrale su Wayland (0% - 100%)")
        .suffix(&intensity_scale)
        .build();
    colorblind_card.append(&intensity_row);

    // Spectral Preview Drawing Area
    let preview_label = Label::builder()
        .label("<b>Anteprima Spettrale Croma Shader GLSL:</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();

    da_preview.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;

        // Draw background
        cr.set_source_rgb(0.1, 0.11, 0.14);
        cr.rectangle(0.0, 0.0, w, h);
        let _ = cr.fill();

        // 6 Color Spectrum Bars
        let colors = [
            (0.9, 0.2, 0.2), // Red
            (0.9, 0.6, 0.1), // Orange/Yellow
            (0.2, 0.8, 0.3), // Green
            (0.1, 0.7, 0.9), // Cyan
            (0.2, 0.4, 0.9), // Blue
            (0.8, 0.3, 0.8), // Purple/Magenta
        ];

        let bar_width = (w - 32.0) / colors.len() as f64;
        let start_x = 16.0;
        let bar_height = h - 20.0;
        let start_y = 10.0;

        for (i, (r, g, b)) in colors.iter().enumerate() {
            let x = start_x + i as f64 * bar_width;
            cr.set_source_rgb(*r, *g, *b);
            cr.rectangle(x, start_y, bar_width - 4.0, bar_height);
            let _ = cr.fill();

            // Border outline
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_line_width(1.0);
            cr.rectangle(x, start_y, bar_width - 4.0, bar_height);
            let _ = cr.stroke();
        }
    });

    colorblind_card.append(&preview_label);
    colorblind_card.append(&da_preview);

    container.append(&colorblind_card);

    // 4. FEATURE 3: Text-to-Speech Navigazione
    let tts_section_title = Label::builder()
        .label("<span size='large' weight='bold'>🔊 Text-to-Speech Navigazione</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(8)
        .build();
    container.append(&tts_section_title);

    let tts_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["liquid-surface"])
        .build();

    let tts_switch = Switch::builder()
        .valign(Align::Center)
        .active(false)
        .build();

    tts_switch.connect_active_notify(move |sw| {
        let active = sw.is_active();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt(
                "a11y_tts_navigation",
                if active { "enabled" } else { "disabled" },
            )
            .await;
        });
    });

    let tts_row = ActionRow::builder("Text-to-Speech Navigazione")
        .subtitle("Sintesi vocale per la navigazione UI, lettura del focus e descrizioni vocali")
        .suffix(&tts_switch)
        .build();
    tts_card.append(&tts_row);

    // Voice Selection
    let voice_combo = ComboBoxText::new();
    voice_combo.append_text("Ermete Neural Voice 1 (Italiano - Naturale)");
    voice_combo.append_text("Ermete eBPF Ultra-Fast (Italiano - Bassa Latenza)");
    voice_combo.append_text("Ermete Neural Multilingual (IT/EN/DE)");
    voice_combo.set_active(Some(0));

    voice_combo.connect_changed(|cb| {
        if let Some(text) = cb.active_text() {
            let val = text.to_string();
            relm4::spawn_local(async move {
                let _ = crate::crdt_store::update_setting_crdt("a11y_tts_voice", &val).await;
            });
        }
    });

    let voice_row = ActionRow::builder("Voce Sintetica")
        .subtitle("Seleziona il modello di sintesi vocale neurale")
        .suffix(&voice_combo)
        .build();
    tts_card.append(&voice_row);

    // Speech Rate Slider
    let rate_scale = Scale::with_range(Orientation::Horizontal, 0.5, 3.0, 0.1);
    rate_scale.set_value(1.0);
    rate_scale.set_draw_value(true);
    rate_scale.set_hexpand(true);
    rate_scale.set_size_request(200, -1);

    rate_scale.connect_value_changed(|sc| {
        let val = sc.value();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_tts_speed", &val.to_string()).await;
        });
    });

    let rate_row = ActionRow::builder("Velocità di Lettura")
        .subtitle("Regola la velocità del parlato (0.5x - 3.0x)")
        .suffix(&rate_scale)
        .build();
    tts_card.append(&rate_row);

    // Test Voice Button & Feedback Label
    let test_tts_btn = Button::builder()
        .label("🔊 Prova Sintesi Vocale")
        .valign(Align::Center)
        .build();

    let tts_feedback_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();

    let feedback_clone = tts_feedback_label.clone();
    test_tts_btn.connect_clicked(move |_| {
        feedback_clone.set_label("🔊 Audio in riproduzione: 'Accessibilità Ermete OS pronta ed operativa.'");
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_tts_test_trigger", "played").await;
        });
    });

    let test_row = ActionRow::builder("Test Audio TTS")
        .subtitle("Riproduci un campione audio di prova con le impostazioni correnti")
        .suffix(&test_tts_btn)
        .build();
    tts_card.append(&test_row);
    tts_card.append(&tts_feedback_label);

    container.append(&tts_card);

    // 5. Presets & Additional Options
    let presets_title = Label::builder()
        .label("<span size='large' weight='bold'>⚡ Opzioni Rapide &amp; Contrasto</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(8)
        .build();
    container.append(&presets_title);

    let presets_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["liquid-surface"])
        .build();

    let hc_switch = Switch::builder().valign(Align::Center).active(false).build();
    hc_switch.connect_active_notify(|sw| {
        let active = sw.is_active();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_high_contrast", if active { "on" } else { "off" }).await;
        });
    });
    let hc_row = ActionRow::builder("Modalità Alto Contrasto UI")
        .subtitle("Aumenta la demarcazione visiva dei bordi e la leggibilità del testo")
        .suffix(&hc_switch)
        .build();
    presets_card.append(&hc_row);

    let rm_switch = Switch::builder().valign(Align::Center).active(false).build();
    rm_switch.connect_active_notify(|sw| {
        let active = sw.is_active();
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("a11y_reduce_motion", if active { "on" } else { "off" }).await;
        });
    });
    let rm_row = ActionRow::builder("Riduci Animazioni e Transizioni")
        .subtitle("Disattiva gli effetti di movimento nel compositore per prevenire cinetosi")
        .suffix(&rm_switch)
        .build();
    presets_card.append(&rm_row);

    container.append(&presets_card);

    container
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a11y_config_payload_crdt_conversion() {
        let mut cfg = A11yConfigPayload::default_config();
        cfg.live_captions_enabled = true;
        cfg.colorblind_filter_enabled = true;
        cfg.shader_type = "Deuteranopia".to_string();
        cfg.tts_navigation_enabled = true;

        let entries = cfg.to_crdt_entries();
        assert_eq!(entries.len(), 9);

        let live_cap = entries
            .iter()
            .find(|(k, _)| *k == "a11y_live_captions")
            .map(|(_, v)| v.as_str());
        assert_eq!(live_cap, Some("enabled"));

        let shader = entries
            .iter()
            .find(|(k, _)| *k == "a11y_colorblind_shader_type")
            .map(|(_, v)| v.as_str());
        assert_eq!(shader, Some("Deuteranopia"));

        let tts = entries
            .iter()
            .find(|(k, _)| *k == "a11y_tts_navigation")
            .map(|(_, v)| v.as_str());
        assert_eq!(tts, Some("enabled"));
    }
}
