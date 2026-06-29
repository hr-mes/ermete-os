import Widget from "resource:///com/github/Aylur/ags/widget.js";
import Variable from "resource:///com/github/Aylur/ags/variable.js";
import SystemTray from "resource:///com/github/Aylur/ags/service/systemtray.js";

// --- Variabili di Stato ---
// L'orologio si aggiorna automaticamente ogni secondo in modo asincrono
const time = Variable("", {
    poll: [1000, 'date "+%H:%M"'],
});

const dateStr = Variable("", {
    poll: [60000, 'date "+%A %d %B"'],
});

// --- Componenti Sinistra ---
// Workspaces (Per ora statici, pronti per essere agganciati al socket di Niri)
const Workspaces = () => Widget.Box({
    class_name: "workspaces module",
    children: [1, 2, 3, 4, 5].map(i => Widget.Button({
        class_name: `workspace-btn ${i === 1 ? 'active' : ''}`, // Simula il primo attivo
        label: `${i}`,
        on_clicked: () => console.log(`Switch to Niri workspace ${i}`),
    })),
});

const Left = () => Widget.Box({
    spacing: 8,
    children: [
        Workspaces(),
    ],
});

// --- Componenti Centro ---
// Orologio Pillola Stile macOS/iOS
const Clock = () => Widget.Box({
    class_name: "clock module",
    children: [
        Widget.Label({
            class_name: "date",
            label: dateStr.bind(),
        }),
        Widget.Label({
            class_name: "time",
            label: time.bind(),
        }),
    ]
});

const Center = () => Widget.Box({
    spacing: 8,
    children: [
        Clock(),
    ],
});

// --- Componenti Destra ---
// System Tray nativa Wayland
const SysTray = () => Widget.Box({
    class_name: "systray module",
    children: SystemTray.bind("items").as(items =>
        items.map(item => Widget.Button({
            child: Widget.Icon({ icon: item.bind("icon") }),
            on_primary_click: (_, event) => item.activate(event),
            on_secondary_click: (_, event) => item.openMenu(event),
            tooltip_markup: item.bind("tooltip_markup"),
        }))
    ),
});

// Bottone Toggle per il futuro Control Center
const ControlCenterToggle = () => Widget.Button({
    class_name: "control-center-toggle module",
    on_clicked: () => console.log("Apri Control Center (Da Implementare)"),
    child: Widget.Icon("preferences-system-symbolic"),
});

const Right = () => Widget.Box({
    hpack: "end",
    spacing: 8,
    children: [
        SysTray(),
        ControlCenterToggle(),
    ],
});

// --- Finestra Principale (Esportata per config.ts) ---
export const Bar = (monitor: number = 0) => Widget.Window({
    name: `bar-${monitor}`, // Nome univoco necessario per multi-monitor
    monitor,
    anchor: ["top", "left", "right"],
    exclusivity: "exclusive", // Riserva lo spazio: Niri non ci disegnerà finestre sotto!
    child: Widget.CenterBox({
        class_name: "bar",
        start_widget: Left(),
        center_widget: Center(),
        end_widget: Right(),
    }),
});
