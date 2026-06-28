import Widget from "resource:///com/github/Aylur/ags/widget.js";
import App from "resource:///com/github/Aylur/ags/app.js";
import Audio from "resource:///com/github/Aylur/ags/service/audio.js";
import * as Utils from "resource:///com/github/Aylur/ags/utils.js";

const WINDOW_NAME = "osd";

export const OSD = (monitor = 0) => {
    let count = 0;

    // Funzione che mostra l'OSD e imposta un timer per nasconderlo dolcemente
    const showOsd = () => {
        App.openWindow(WINDOW_NAME);
        count++;
        Utils.timeout(2000, () => {
            count--;
            if (count === 0) App.closeWindow(WINDOW_NAME);
        });
    };

    let isInitial = true;

    return Widget.Window({
        name: WINDOW_NAME,
        class_name: "osd-window",
        monitor,
        layer: "overlay", // Si piazza prepotentemente sopra qualsiasi finestra o videogioco fullscreen
        anchor: ["bottom"], // Al centro in basso, stile macOS
        margins: [0, 0, 140, 0], // Distanza dal fondo dello schermo
        visible: false,
        child: Widget.Box({
            class_name: "osd-container",
            spacing: 24,
            children: [
                // Icona Dinamica del Volume
                Widget.Icon({
                    class_name: "osd-icon",
                }).hook(Audio, icon => {
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
                
                // Barra di Progresso Spessa
                Widget.LevelBar({
                    class_name: "osd-level",
                    vpack: "center",
                }).hook(Audio, bar => {
                    bar.value = Audio.speaker?.volume || 0;
                    
                    // Mostra l'OSD solo se non è il caricamento iniziale di avvio
                    if (isInitial) {
                        isInitial = false;
                        return;
                    }
                    showOsd();
                }, "speaker-changed"),
            ],
        }),
    });
};
