import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { Variable, GLib, bind } from "astal"
import AstalTray from "gi://AstalTray"

const time = Variable("").poll(1000, 'date "+%H:%M"')
const dateStr = Variable("").poll(60000, 'date "+%A %d %B"')

function Workspaces() {
    return <box className="workspaces module">
        {[1, 2, 3, 4, 5].map(i => (
            <button 
                className={`workspace-btn ${i === 1 ? 'active' : ''}`}
                onClicked={() => print(`Switch to Niri workspace ${i}`)}>
                {i}
            </button>
        ))}
    </box>
}

function Clock() {
    return <box className="clock module">
        <label className="date" label={dateStr()} />
        <label className="time" label={time()} />
    </box>
}

function SysTray() {
    const tray = AstalTray.get_default()

    return <box className="systray module">
        {bind(tray, "items").as(items => items.map(item => (
            <button
                tooltipMarkup={bind(item, "tooltipMarkup")}
                onClickRelease={(_, event) => {
                    if (event.button === 1) item.activate(0, 0)
                    if (event.button === 3) item.open_menu(0, 0)
                }}>
                <icon gicon={bind(item, "gicon")} />
            </button>
        )))}
    </box>
}

function ControlCenterToggle() {
    return <button 
        className="control-center-toggle module" 
        onClicked={() => print("Apri Control Center")}>
        <icon iconName="preferences-system-symbolic" />
    </button>
}

export default function Bar(gdkmonitor: Gdk.Monitor) {
    return <window
        name={`bar-${gdkmonitor.get_model()}`}
        className="bar"
        monitor={gdkmonitor}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT}
        exclusivity={Astal.Exclusivity.EXCLUSIVE}
        application={App}>
        <centerbox>
            <box spacing={8}>
                <Workspaces />
            </box>
            <box spacing={8}>
                <Clock />
            </box>
            <box spacing={8} halign={Gtk.Align.END}>
                <SysTray />
                <ControlCenterToggle />
            </box>
        </centerbox>
    </window>
}
