#!/usr/bin/env python3
import os
import sys
import json
import time
import socket
import logging
import urllib.request
import urllib.error

# Niri AI Auto-Tiling Neurale IPC Daemon
# Connette al socket Niri, calcola il peso cognitivo usando ermete-ai-daemon
# e ri-organizza il tiling dinamicamente.

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger("niri-ai-tiling")

class NiriIPC:
    def __init__(self):
        self.socket_path = os.environ.get("NIRI_SOCKET")
        if not self.socket_path:
            uid = os.getuid()
            self.socket_path = f"/run/user/{uid}/niri.sock"
        
    def _send_request(self, req: dict) -> dict:
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.connect(self.socket_path)
                s.sendall((json.dumps(req) + "\n").encode('utf-8'))
                
                # Semplice lettura della risposta
                response_data = b""
                while True:
                    chunk = s.recv(4096)
                    if not chunk:
                        break
                    response_data += chunk
                    if b"\n" in chunk:
                        break
                if not response_data:
                    return {}
                return json.loads(response_data.decode('utf-8').strip())
        except Exception as e:
            logger.error(f"Errore IPC Niri: {e}")
            return {}

    def get_windows(self):
        # Ipotizziamo che 'Action' -> 'Windows' ritorni la lista delle finestre
        return self._send_request({"Action": "Windows"})

    def focus_window(self, window_id):
        return self._send_request({"Action": "FocusWindow", "id": window_id})

    def set_window_width(self, window_id, width):
        return self._send_request({"Action": "SetWindowWidth", "id": window_id, "width": width})


class ErmeteAIDaemon:
    def __init__(self, endpoint="http://localhost:11434/api/generate"):
        # Ipotizziamo un endpoint locale stile Ollama per ermete-ai-daemon
        self.endpoint = endpoint

    def analyze_cognitive_load(self, window_title, window_app_id):
        prompt = f"""Analizza il carico cognitivo di questa finestra e restituisci un peso da 1 (basso) a 10 (alto).
Nome app: {window_app_id}
Titolo: {window_title}
Rispondi SOLO con un numero intero."""
        
        data = {
            "model": "ermete-npu",
            "prompt": prompt,
            "stream": False
        }
        
        try:
            req = urllib.request.Request(self.endpoint, data=json.dumps(data).encode('utf-8'), headers={'Content-Type': 'application/json'})
            with urllib.request.urlopen(req, timeout=2) as response:
                result = json.loads(response.read().decode('utf-8'))
                weight = int(result.get("response", "5").strip())
                return max(1, min(10, weight))
        except Exception as e:
            logger.debug(f"AI Daemon irraggiungibile ({e}), uso fallback euristico.")
            return self._fallback_heuristic(window_title, window_app_id)

    def _fallback_heuristic(self, title, app_id):
        title = (title or "").lower()
        app_id = (app_id or "").lower()
        if "terminal" in app_id or "kitty" in app_id or "alacritty" in app_id:
            return 8
        if "code" in app_id or "ide" in app_id:
            return 9
        if "browser" in app_id or "firefox" in app_id:
            return 6
        return 4


def main():
    logger.info("Avvio Niri AI Tiling Architect Daemon...")
    niri = NiriIPC()
    ai = ErmeteAIDaemon()

    while True:
        try:
            windows_data = niri.get_windows()
            windows = windows_data.get("windows", [])
            
            if not windows:
                time.sleep(2)
                continue
            
            # Calcola pesi cognitivi
            scored_windows = []
            for w in windows:
                w_id = w.get("id")
                title = w.get("title", "")
                app_id = w.get("app_id", "")
                weight = ai.analyze_cognitive_load(title, app_id)
                scored_windows.append((weight, w_id, title))
            
            # Ordina le finestre per carico cognitivo
            scored_windows.sort(key=lambda x: x[0], reverse=True)
            
            logger.info(f"Tree analizzato. Finestra principale: {scored_windows[0][2]} (Peso: {scored_windows[0][0]})")
            
            # Applica dimensionamento basato sul peso
            # Ad esempio alla finestra con carico massimo diamo una larghezza maggiore
            total_weight = sum(w[0] for w in scored_windows)
            
            for weight, w_id, title in scored_windows:
                # Logica semplificata: calcolo della larghezza ottimale in %
                target_width_percent = (weight / total_weight) * 100
                logger.debug(f"Modifico larghezza di '{title}' a {target_width_percent:.1f}%")
                # IPC a Niri (usando l'API ipotetica)
                niri.set_window_width(w_id, max(20.0, target_width_percent))
            
            # Pausa per non saturare la NPU/CPU
            time.sleep(10)
            
        except KeyboardInterrupt:
            logger.info("Chiusura demone...")
            break
        except Exception as e:
            logger.error(f"Errore nel main loop: {e}")
            time.sleep(5)

if __name__ == "__main__":
    main()
