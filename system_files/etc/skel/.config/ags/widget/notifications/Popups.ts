import Widget from "resource:///com/github/Aylur/ags/widget.js";
import Notifications from "resource:///com/github/Aylur/ags/service/notifications.js";

// Genera l'icona della notifica (immagine integrata o icona dell'app)
const NotificationIcon = (notif: any) => {
    if (notif.image) {
        return Widget.Box({
            vpack: "start",
            class_name: "notif-icon image",
            css: `background-image: url('${notif.image}');`,
        });
    }
    return Widget.Icon({
        vpack: "start",
        class_name: "notif-icon",
        icon: notif.app_icon || notif.app_entry || "dialog-information-symbolic",
    });
};

// Layout della singola notifica
const Notification = (notif: any) => Widget.EventBox({
    class_name: `notification ${notif.urgency}`,
    // Chiude la notifica al click sinistro
    on_primary_click: () => notif.dismiss(),
    child: Widget.Box({
        vertical: true,
        class_name: "notif-content",
        children: [
            Widget.Box({
                spacing: 16,
                children: [
                    NotificationIcon(notif),
                    Widget.Box({
                        vertical: true,
                        vpack: "center",
                        children: [
                            Widget.Label({
                                class_name: "notif-title",
                                xalign: 0,
                                justification: "left",
                                truncate: "end",
                                label: notif.summary,
                            }),
                            Widget.Label({
                                class_name: "notif-body",
                                xalign: 0,
                                justification: "left",
                                wrap: true,
                                label: notif.body,
                            }),
                        ],
                    }),
                ],
            }),
            // Se la notifica ha dei bottoni ("Rispondi", "Apri", ecc.)
            Widget.Box({
                class_name: "notif-actions",
                spacing: 8,
                children: notif.actions.map((action: any) => Widget.Button({
                    class_name: "notif-action-btn",
                    on_clicked: () => notif.invoke(action.id),
                    hexpand: true,
                    child: Widget.Label(action.label),
                })),
            }),
        ],
    }),
});

// Finestra che contiene l'elenco dei Pop-up attivi
export const NotificationPopups = (monitor = 0) => Widget.Window({
    name: `notifications-${monitor}`,
    class_name: "notification-popups",
    monitor,
    anchor: ["top", "right"], // Scendono in alto a destra
    margins: [12, 12], // Sotto la barra
    child: Widget.Box({
        vertical: true,
        spacing: 12,
        // Esegue il "bind" sui popups: la GUI si aggiorna automaticamente
        children: Notifications.bind("popups").as(popups =>
            popups.map(Notification)
        ),
    }),
});
