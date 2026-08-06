use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Widget};

/// Widget GTK4 puro che emula una `AdwActionRow`
/// senza dipendenze esterne. Segue la Ponytail Rule.
pub struct ActionRow;

impl ActionRow {
    /// Costruisce una row GTK4 con titolo, sottotitolo opzionale e widget suffisso opzionale.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        title: &str,
        subtitle: Option<&str>,
        suffix: Option<&impl IsA<Widget>>,
    ) -> GtkBox {
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .css_classes(["action-row"])
            .build();

        let text_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Center)
            .hexpand(true)
            .build();

        let title_label = Label::builder()
            .label(title)
            .halign(Align::Start)
            .css_classes(["action-row-title"])
            .build();
        text_box.append(&title_label);

        if let Some(sub) = subtitle {
            let subtitle_label = Label::builder()
                .label(sub)
                .halign(Align::Start)
                .css_classes(["action-row-subtitle"])
                .build();
            text_box.append(&subtitle_label);
        }

        container.append(&text_box);

        if let Some(suffix_widget) = suffix {
            let suffix_box = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .valign(Align::Center)
                .halign(Align::End)
                .css_classes(["action-row-suffix"])
                .build();
            suffix_box.append(suffix_widget);
            container.append(&suffix_box);
        }

        container
    }

    /// Builder fluido per composizione flessibile e leggibile.
    pub fn builder(title: &str) -> ActionRowBuilder {
        ActionRowBuilder::new(title)
    }
}

/// Builder per la costruzione incrementale di `ActionRow`.
pub struct ActionRowBuilder {
    title: String,
    subtitle: Option<String>,
    suffix: Option<Widget>,
}

impl ActionRowBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            subtitle: None,
            suffix: None,
        }
    }

    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn suffix(mut self, widget: &impl IsA<Widget>) -> Self {
        self.suffix = Some(widget.clone().upcast());
        self
    }

    pub fn build(self) -> GtkBox {
        ActionRow::new(
            &self.title,
            self.subtitle.as_deref(),
            self.suffix.as_ref(),
        )
    }
}
