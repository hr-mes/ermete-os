import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { bind, Variable } from "astal"
import AstalWp from "gi://AstalWp"

const WINDOW_NAME = "osd"
let count = 0

function showOsd() {
    App.get_window(WINDOW_NAME)!.show()
    count++
    setTimeout(() => {
        count--
        if (count === 0) App.get_window(WINDOW_NAME)!.hide()
    }, 2000)
}

export default function OSD(gdkmonitor: Gdk.Monitor) {
    const speaker = AstalWp.get_default()?.audio.defaultSpeaker!
    
    speaker.connect("notify::volume", () => showOsd())
    speaker.connect("notify::mute", () => showOsd())

    return <window
        name={WINDOW_NAME}
        className="osd-window"
        monitor={gdkmonitor}
        layer={Astal.Layer.OVERLAY}
        anchor={Astal.WindowAnchor.BOTTOM}
        marginBottom={140}
        visible={false}
        application={App}>
        <box className="osd-container" spacing={24}>
            <icon className="osd-icon" iconName={bind(speaker, "volumeIcon")} />
            <levelbar className="osd-level" valign={Gtk.Align.CENTER} value={bind(speaker, "volume")} />
        </box>
    </window>
}
