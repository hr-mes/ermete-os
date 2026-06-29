import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { bind, Variable } from "astal"
import AstalNotifd from "gi://AstalNotifd"

function NotificationIcon({ notif }: { notif: AstalNotifd.Notification }) {
    if (notif.image) {
        return <box
            valign={Gtk.Align.START}
            className="notif-icon image"
            css={`background-image: url('${notif.image}');`}
        />
    }
    return <icon
        valign={Gtk.Align.START}
        className="notif-icon"
        iconName={notif.appIcon || notif.desktopEntry || "dialog-information-symbolic"}
    />
}

function Notification({ notif }: { notif: AstalNotifd.Notification }) {
    return <eventbox
        className={`notification ${notif.urgency === AstalNotifd.Urgency.CRITICAL ? "critical" : ""}`}
        onClick={() => notif.dismiss()}>
        <box vertical className="notif-content">
            <box spacing={16}>
                <NotificationIcon notif={notif} />
                <box vertical valign={Gtk.Align.CENTER}>
                    <label className="notif-title" xalign={0} justify={Gtk.Justification.LEFT} truncate label={notif.summary} />
                    <label className="notif-body" xalign={0} justify={Gtk.Justification.LEFT} wrap label={notif.body} />
                </box>
            </box>
            {notif.get_actions().length > 0 && (
                <box className="notif-actions" spacing={8}>
                    {notif.get_actions().map(action => (
                        <button className="notif-action-btn" hexpand onClicked={() => notif.invoke(action.id)}>
                            <label label={action.label} />
                        </button>
                    ))}
                </box>
            )}
        </box>
    </eventbox>
}

export default function NotificationPopups(gdkmonitor: Gdk.Monitor) {
    const notifd = AstalNotifd.get_default()

    return <window
        name={`notifications-${gdkmonitor.get_model()}`}
        className="notification-popups"
        monitor={gdkmonitor}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT}
        margin={12}
        application={App}>
        <box vertical spacing={12}>
            {bind(notifd, "notifications").as(n => n.map(notif => <Notification notif={notif} />))}
        </box>
    </window>
}
