import Widget from "resource:///com/github/Aylur/ags/widget.js";
import App from "resource:///com/github/Aylur/ags/app.js";
import Audio from "resource:///com/github/Aylur/ags/service/audio.js";
import Network from "resource:///com/github/Aylur/ags/service/network.js";
import Bluetooth from "resource:///com/github/Aylur/ags/service/bluetooth.js";

const WINDOW_NAME = "control-center";

// --- Sezione Audio (Il killer di pavucontrol) ---
const VolumeSlider = () => Widget.Box({
    class_name: "slider-box",
    children: [
        Widget.Button({
            on_clicked: () => Audio.speaker.is_muted = !Audio.speaker.is_muted,
            child: Widget.Icon().hook(Audio, icon => {
                if (!Audio.speaker) return;
                const vol = Audio.speaker.volume * 100;
                const isMuted = Audio.speaker.is_muted;
                const iconName = [
                    [101, 'overamplified'],
                    [67, 'high'],
                    [34, 'medium'],
                    [1, 'low'],
                    [0, 'muted'],
                ].find(([threshold]) => Number(threshold) <= vol)?.[1];
                icon.icon = isMuted ? "audio-volume-muted-symbolic" : `audio-volume-${iconName}-symbolic`;
            }, "speaker-changed"),
        }),
        Widget.Slider({
            hexpand: true,
            draw_value: false,
            class_name: "vol-slider",
            on_change: ({ value }) => Audio.speaker.volume = value,
            setup: self => self.hook(Audio, () => {
                self.value = Audio.speaker?.volume || 0;
            }, "speaker-changed"),
        }),
    ],
});

const MicSlider = () => Widget.Box({
    class_name: "slider-box",
    children: [
        Widget.Button({
            on_clicked: () => Audio.microphone.is_muted = !Audio.microphone.is_muted,
            child: Widget.Icon().hook(Audio, icon => {
                icon.icon = Audio.microphone?.is_muted 
                    ? "audio-input-microphone-muted-symbolic" 
                    : "audio-input-microphone-symbolic";
            }, "microphone-changed"),
        }),
        Widget.Slider({
            hexpand: true,
            draw_value: false,
            class_name: "mic-slider",
            on_change: ({ value }) => Audio.microphone.volume = value,
            setup: self => self.hook(Audio, () => {
                self.value = Audio.microphone?.volume || 0;
            }, "microphone-changed"),
        }),
    ],
});

// --- Pulsanti Giganti "Quick Toggles" (I killer di nm-applet e blueman) ---
const NetworkButton = () => Widget.Button({
    class_name: "qs-button network",
    hexpand: true,
    // Qui andrebbe l'apertura del sottomenu per la lista reti (da implementare nel dettaglio)
    on_clicked: () => Network.toggleWifi(), 
    child: Widget.Box({
        class_name: "qs-button-content",
        children: [
            Widget.Icon({ class_name: "qs-icon" }).hook(Network, icon => {
                icon.icon = Network.wifi?.icon_name || Network.wired?.icon_name || "network-wireless-offline-symbolic";
            }),
            Widget.Box({
                vertical: true,
                vpack: "center",
                children: [
                    Widget.Label({ label: "Wi-Fi", xalign: 0, class_name: "qs-title" }),
                    Widget.Label({ xalign: 0, class_name: "qs-subtitle", truncate: "end" }).hook(Network, label => {
                        label.label = Network.wifi?.ssid || Network.wired?.internet || "Disconnesso";
                    }),
                ],
            }),
        ],
    }),
});

const BluetoothButton = () => Widget.Button({
    class_name: "qs-button bluetooth",
    hexpand: true,
    on_clicked: () => Bluetooth.toggle(),
    child: Widget.Box({
        class_name: "qs-button-content",
        children: [
            Widget.Icon({ class_name: "qs-icon" }).hook(Bluetooth, icon => {
                icon.icon = Bluetooth.enabled ? "bluetooth-active-symbolic" : "bluetooth-disabled-symbolic";
            }),
            Widget.Box({
                vertical: true,
                vpack: "center",
                children: [
                    Widget.Label({ label: "Bluetooth", xalign: 0, class_name: "qs-title" }),
                    Widget.Label({ xalign: 0, class_name: "qs-subtitle" }).hook(Bluetooth, label => {
                        label.label = Bluetooth.enabled 
                            ? (Bluetooth.connected_devices.length > 0 ? `${Bluetooth.connected_devices.length} Connessi` : "Attivo") 
                            : "Spento";
                    }),
                ],
            }),
        ],
    }),
});

// --- Finestra Principale Control Center ---
export const ControlCenter = () => Widget.Window({
    name: WINDOW_NAME,
    class_name: "control-center-window",
    visible: false,
    anchor: ["top", "right"], // Fissato in alto a destra sotto la barra
    margins: [4, 12], // Margine per distanziarlo dalla barra superiore e dal bordo destro
    
    // Essendo un popup, possiamo chiuderlo cliccando fuori se aggiungiamo logica specifica,
    // oppure gestirlo via pulsante toggle nella barra.
    
    child: Widget.Box({
        class_name: "control-center-container",
        vertical: true,
        spacing: 20,
        children: [
            // Riga dei Quick Settings (Pulsanti Giganti)
            Widget.Box({
                spacing: 12,
                children: [
                    NetworkButton(),
                    BluetoothButton(),
                ],
            }),
            // Sezione Audio Dettagliata
            Widget.Box({
                class_name: "audio-section",
                vertical: true,
                spacing: 12,
                children: [
                    Widget.Label({ label: "Audio & Input", xalign: 0, class_name: "section-title" }),
                    VolumeSlider(),
                    MicSlider(),
                ],
            }),
        ],
    }),
});
