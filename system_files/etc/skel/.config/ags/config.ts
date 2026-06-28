import App from "resource:///com/github/Aylur/ags/app.js";
import * as Utils from "resource:///com/github/Aylur/ags/utils.js";

// --- Importazione Moduli UI (saranno attivati man mano che li costruiamo) ---
import { Bar } from "./widget/bar/Bar.js";
import { Launcher } from "./widget/launcher/Launcher.js";
import { ControlCenter } from "./widget/quicksettings/ControlCenter.js";
import { OSD } from "./widget/osd/OSD.js";
import { NotificationPopups } from "./widget/notifications/Popups.js";
import { PowerMenu } from "./widget/powermenu/PowerMenu.js";

// --- Compilazione SCSS a runtime ---
// Sfruttiamo dart-sass (installato precedentemente) per compilare i nostri stili.
// L'uso di Utils.monitorFile ci permetterà, in fase di sviluppo, di ricaricare 
// automaticamente il tema ogni volta che salviamo un file .scss!
const scss = `${App.configDir}/scss/main.scss`;
const css  = `/tmp/ags-ermete-style.css`; // Usiamo /tmp per non sporcare la directory utente con file generati

try {
    Utils.exec(`sass ${scss} ${css}`);
    console.log("SCSS Compilato con successo in fase di avvio.");
} catch (error) {
    console.error("Errore nella compilazione SCSS:", error);
}

// Ricaricamento dinamico dello stile per "Live Reload" durante lo sviluppo
Utils.monitorFile(
    `${App.configDir}/scss`,
    function() {
        try {
            Utils.exec(`sass ${scss} ${css}`);
            App.resetCss();
            App.applyCss(css);
            console.log("Stile aggiornato dinamicamente!");
        } catch (error) {
            console.error("Errore durante il reload SCSS:", error);
        }
    },
);

// --- Configurazione Principale Demone AGS ---
export default {
    // Il CSS compilato
    style: css,
    
    // Lista delle finestre LayerShell da disegnare a schermo
    windows: [
        // Decommenteremo questi componenti man mano che li programmiamo
        Bar(0), // Barra per il monitor principale (DP-1)
        Bar(1), // Barra per il monitor secondario (HDMI-A-1)
        Launcher(),
        ControlCenter(),
        OSD(0),
        NotificationPopups(0),
        PowerMenu(),
    ],
    
    // Configurazione del comportamento del demone
    closeWindowDelay: {
        // Aggiunge un ritardo alla chiusura delle finestre per permettere l'esecuzione delle animazioni CSS in uscita (fondamentale per i 360Hz!)
        "launcher": 150,
        "control-center": 200,
    },
};
