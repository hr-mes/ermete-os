import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { execAsync } from "astal/process"

const WINDOW_NAME = "powermenu"

function PowerButton({ name, iconName, command, colorClass = "" }: { name: string, iconName: string, command: string, colorClass?: string }) {
    return <button
        className={`power-btn ${colorClass}`}
        onClicked={() => {
            App.get_window(WINDOW_NAME)!.hide()
            execAsync(command).catch(print)
        }}>
        <box vertical valign={Gtk.Align.CENTER} spacing={16}>
            <icon iconName={iconName} className="power-icon" />
            <label label={name} className="power-label" />
        </box>
    </button>
}

export default function PowerMenu() {
    return <window
        name={WINDOW_NAME}
        className="powermenu-window"
        visible={false}
        keymode={Astal.Keymode.EXCLUSIVE}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT}
        exclusivity={Astal.Exclusivity.IGNORE}
        application={App}>
        
        <eventbox onClick={() => App.get_window(WINDOW_NAME)!.hide()}>
            <box
                vertical
                valign={Gtk.Align.CENTER}
                halign={Gtk.Align.CENTER}
                spacing={32}
                className="powermenu-container"
                onButtonPressEvent={() => true}>
                <label label="Ermete OS Session" className="powermenu-title" />
                <box spacing={24} halign={Gtk.Align.CENTER}>
                    <PowerButton name="Sospendi" iconName="system-suspend-symbolic" command="systemctl suspend" />
                    <PowerButton name="Riavvia" iconName="system-reboot-symbolic" command="systemctl reboot" />
                    <PowerButton name="Spegni" iconName="system-shutdown-symbolic" command="systemctl poweroff" colorClass="power-danger" />
                </box>
            </box>
        </eventbox>
    </window>
}
