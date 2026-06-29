import { App, Astal, Gtk, Gdk } from "astal/gtk3"
import { bind, Variable } from "astal"
import AstalWp from "gi://AstalWp"
import AstalNetwork from "gi://AstalNetwork"
import AstalBluetooth from "gi://AstalBluetooth"

const WINDOW_NAME = "control-center"

function VolumeSlider() {
    const speaker = AstalWp.get_default()?.audio.defaultSpeaker!
    return <box className="slider-box">
        <button onClicked={() => speaker.set_mute(!speaker.mute)}>
            <icon iconName={bind(speaker, "volumeIcon")} />
        </button>
        <slider
            hexpand
            drawValue={false}
            className="vol-slider"
            value={bind(speaker, "volume")}
            onDragged={({ value }) => speaker.set_volume(value)}
        />
    </box>
}

function MicSlider() {
    const mic = AstalWp.get_default()?.audio.defaultMicrophone!
    return <box className="slider-box">
        <button onClicked={() => mic.set_mute(!mic.mute)}>
            <icon iconName={bind(mic, "volumeIcon")} />
        </button>
        <slider
            hexpand
            drawValue={false}
            className="mic-slider"
            value={bind(mic, "volume")}
            onDragged={({ value }) => mic.set_volume(value)}
        />
    </box>
}

function NetworkButton() {
    const net = AstalNetwork.get_default()
    return <button className="qs-button network" hexpand onClicked={() => print("Toggle WiFi")}>
        <box className="qs-button-content">
            <icon className="qs-icon" iconName={bind(net, "primary").as(p => p === AstalNetwork.Primary.WIFI ? net.wifi.iconName : net.wired.iconName)} />
            <box vertical valign={Gtk.Align.CENTER}>
                <label className="qs-title" xalign={0} label="Wi-Fi" />
                <label className="qs-subtitle" xalign={0} truncate label={bind(net, "primary").as(p => p === AstalNetwork.Primary.WIFI ? net.wifi.ssid : "Connesso")} />
            </box>
        </box>
    </button>
}

function BluetoothButton() {
    const bt = AstalBluetooth.get_default()
    return <button className="qs-button bluetooth" hexpand onClicked={() => bt.toggle()}>
        <box className="qs-button-content">
            <icon className="qs-icon" iconName={bind(bt, "isPowered").as(p => p ? "bluetooth-active-symbolic" : "bluetooth-disabled-symbolic")} />
            <box vertical valign={Gtk.Align.CENTER}>
                <label className="qs-title" xalign={0} label="Bluetooth" />
                <label className="qs-subtitle" xalign={0} label={bind(bt, "isPowered").as(p => p ? "Attivo" : "Spento")} />
            </box>
        </box>
    </button>
}

export default function ControlCenter() {
    return <window
        name={WINDOW_NAME}
        className="control-center-window"
        visible={false}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT}
        margin={12}
        application={App}>
        <box className="control-center-container" vertical spacing={20}>
            <box spacing={12}>
                <NetworkButton />
                <BluetoothButton />
            </box>
            <box className="audio-section" vertical spacing={12}>
                <label className="section-title" xalign={0} label="Audio & Input" />
                <VolumeSlider />
                <MicSlider />
            </box>
        </box>
    </window>
}
