import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { Variable, bind } from "astal"
import AstalApps from "gi://AstalApps"

const WINDOW_NAME = "applauncher"

function AppItem({ app }: { app: AstalApps.Application }) {
    return <button
        className="app-item"
        onClicked={() => {
            App.get_window(WINDOW_NAME)!.hide()
            app.launch()
        }}>
        <box>
            <icon iconName={app.iconName || ""} className="app-icon" />
            <box vertical valign={Gtk.Align.CENTER} className="app-text-box">
                <label
                    className="title"
                    label={app.name}
                    xalign={0}
                    valign={Gtk.Align.CENTER}
                    truncate
                />
                {app.description && <label
                    className="description"
                    label={app.description}
                    wrap
                    xalign={0}
                    justify={Gtk.Justification.LEFT}
                    valign={Gtk.Align.CENTER}
                    truncate
                />}
            </box>
        </box>
    </button>
}

export default function Launcher() {
    const apps = new AstalApps.Apps()
    const text = Variable("")
    const list = Variable<AstalApps.Application[]>([])

    // Update list when text changes
    text.subscribe((t) => {
        list.set(apps.fuzzy_query(t).slice(0, 10))
    })

    return <window
        name={WINDOW_NAME}
        className="launcher-window"
        visible={false}
        keymode={Astal.Keymode.EXCLUSIVE}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT}
        exclusivity={Astal.Exclusivity.IGNORE}
        application={App}
        onNotifyVisible={(self) => {
            if (self.visible) {
                text.set("")
            }
        }}>
        
        <eventbox onClick={() => App.get_window(WINDOW_NAME)!.hide()}>
            <box
                className="launcher-container"
                vertical
                valign={Gtk.Align.CENTER}
                halign={Gtk.Align.CENTER}
                onButtonPressEvent={() => true}>
                
                <entry
                    hexpand
                    className="search-box"
                    placeholderText="Cerca in Ermete OS..."
                    text={bind(text)}
                    onChanged={(self) => text.set(self.text)}
                    onActivate={() => {
                        const results = list.get()
                        if (results.length > 0) {
                            App.get_window(WINDOW_NAME)!.hide()
                            results[0].launch()
                        }
                    }}
                />
                
                <scrollable hscroll={Gtk.PolicyType.NEVER} className="app-scrollable">
                    <box vertical spacing={4} className="app-list">
                        {bind(list).as(l => l.map(app => <AppItem app={app} />))}
                    </box>
                </scrollable>
            </box>
        </eventbox>
    </window>
}
