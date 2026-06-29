import Widget from "resource:///com/github/Aylur/ags/widget.js";
import App from "resource:///com/github/Aylur/ags/app.js";
import * as Utils from "resource:///com/github/Aylur/ags/utils.js";

const WINDOW_NAME = "powermenu";

const PowerButton = (name: string, icon: string, command: string, colorClass: string = "") => Widget.Button({
    class_name: `power-btn ${colorClass}`,
    on_clicked: () => {
        App.closeWindow(WINDOW_NAME);
        // Esegue il comando di sistema via Utils
        Utils.execAsync(command).catch(console.error);
    },
    child: Widget.Box({
        vertical: true,
        vpack: "center",
        spacing: 16,
        children: [
            Widget.Icon({ icon, class_name: "power-icon" }),
            Widget.Label({ label: name, class_name: "power-label" }),
        ],
    }),
});

export const PowerMenu = () => Widget.Window({
    name: WINDOW_NAME,
    class_name: "powermenu-window",
    visible: false,
    keymode: "exclusive",
    // Ancora a tutti e 4 i bordi per creare l'overlay a tutto schermo
    anchor: ["top", "bottom", "left", "right"],
    exclusivity: "ignore",
    
    child: Widget.EventBox({
        // Click sullo sfondo = chiudi
        on_primary_click: () => App.closeWindow(WINDOW_NAME),
        child: Widget.Box({
            vertical: true,
            vpack: "center",
            hpack: "center",
            spacing: 32,
            class_name: "powermenu-container",
            // Evita che il click sui pulsanti propaghi la chiusura allo sfondo
            setup: self => self.on("button-press-event", () => true),
            children: [
                Widget.Label({ label: "Ermete OS Session", class_name: "powermenu-title" }),
                Widget.Box({
                    spacing: 24,
                    hpack: "center",
                    children: [
                        PowerButton("Sospendi", "system-suspend-symbolic", "systemctl suspend"),
                        PowerButton("Riavvia", "system-reboot-symbolic", "systemctl reboot"),
                        PowerButton("Spegni", "system-shutdown-symbolic", "systemctl poweroff", "power-danger"),
                    ],
                }),
            ],
        }),
    }),
});
