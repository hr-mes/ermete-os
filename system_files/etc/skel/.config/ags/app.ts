import { App } from "astal/gtk3"
import style from "./scss/main.scss"
import Bar from "./widget/Bar"
import Launcher from "./widget/Launcher"
import PowerMenu from "./widget/PowerMenu"
import ControlCenter from "./widget/ControlCenter"
import OSD from "./widget/OSD"
import NotificationPopups from "./widget/Popups"

App.start({
    css: style,
    instanceName: "astal",
    requestHandler(request, res) {
        if (request === "applauncher") {
            const win = App.get_window("applauncher")
            win?.set_visible(!win.visible)
        } else if (request === "powermenu") {
            const win = App.get_window("powermenu")
            win?.set_visible(!win.visible)
        } else if (request === "control-center") {
            const win = App.get_window("control-center")
            win?.set_visible(!win.visible)
        }
        res("ok")
    },
    main() {
        App.get_monitors().map(Bar)
        App.get_monitors().map(OSD)
        App.get_monitors().map(NotificationPopups)
        Launcher()
        PowerMenu()
        ControlCenter()
    },
})
