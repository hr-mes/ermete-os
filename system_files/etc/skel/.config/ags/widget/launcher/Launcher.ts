import Widget from "resource:///com/github/Aylur/ags/widget.js";
import App from "resource:///com/github/Aylur/ags/app.js";
import Applications from "resource:///com/github/Aylur/ags/service/applications.js";

const WINDOW_NAME = "applauncher";

// Costruisce la singola riga dell'applicazione
const AppItem = (app: any) => Widget.Button({
    class_name: "app-item",
    on_clicked: () => {
        App.closeWindow(WINDOW_NAME);
        app.launch();
    },
    attribute: { app },
    child: Widget.Box({
        children: [
            Widget.Icon({
                icon: app.icon_name || "",
                size: 42,
            }),
            Widget.Box({
                vertical: true,
                vpack: "center",
                class_name: "app-text-box",
                children: [
                    Widget.Label({
                        class_name: "title",
                        label: app.name,
                        xalign: 0,
                        vpack: "center",
                        truncate: "end",
                    }),
                    Widget.Label({
                        class_name: "description",
                        label: app.description || "",
                        wrap: true,
                        xalign: 0,
                        justification: "left",
                        vpack: "center",
                        truncate: "end",
                    }),
                ],
            }),
        ],
    }),
});

// Il Launcher Completo
export const Launcher = () => {
    // La lista che conterrà i risultati dinamicamente
    const list = Widget.Box({
        vertical: true,
        spacing: 4,
        class_name: "app-list",
    });

    // La barra di ricerca
    const entry = Widget.Entry({
        hexpand: true,
        class_name: "search-box",
        placeholder_text: "Cerca in Ermete OS...",
        
        // Motore di ricerca istantaneo
        on_change: ({ text }) => {
            // Interroga il servizio nativo AGS per le app installate
            const apps = Applications.query(text || "");
            list.children = apps.map(AppItem);
        },
        
        // Se si preme Invio, lancia la prima applicazione in elenco
        on_accept: () => {
            const results = list.children;
            if (results.length > 0) {
                App.closeWindow(WINDOW_NAME);
                results[0].attribute.app.launch();
            }
        },
    });

    return Widget.Window({
        name: WINDOW_NAME,
        class_name: "launcher-window",
        visible: false, // Parte invisibile
        keymode: "exclusive", // Cattura il focus della tastiera appena appare
        
        // Questo trucco espande la finestra a tutto schermo invisibilmente.
        // Cliccare nello spazio vuoto chiuderà il launcher.
        anchor: ["top", "bottom", "left", "right"], 
        exclusivity: "ignore",
        
        setup: self => self.hook(App, (_, windowName, visible) => {
            if (windowName !== WINDOW_NAME) return;
            // Al momento dell'apertura: resetta il testo, popola tutte le app e da il focus alla ricerca
            if (visible) {
                entry.text = "";
                entry.grab_focus();
                const apps = Applications.query("");
                list.children = apps.map(AppItem);
            }
        }),
        
        child: Widget.EventBox({
            on_primary_click: () => App.closeWindow(WINDOW_NAME), // Chiudi cliccando fuori
            child: Widget.Box({
                class_name: "launcher-container",
                vertical: true,
                vpack: "center",
                hpack: "center",
                setup: self => self.on("button-press-event", () => true), // Impedisci al click di "bucare" la finestra chiudendola
                children: [
                    entry,
                    Widget.Scrollable({
                        hscroll: "never",
                        class_name: "app-scrollable",
                        child: list,
                    }),
                ],
            }),
        }),
    });
};
