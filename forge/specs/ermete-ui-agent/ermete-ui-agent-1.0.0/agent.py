#!/usr/bin/env python3
import json
import time
import subprocess
import requests

WIDGETS_FILE = "/tmp/widgets.json"
OLLAMA_URL = "http://localhost:11434/api/generate"

def get_active_app():
    try:
        out = subprocess.check_output(["niri", "msg", "-j", "focused-window"])
        data = json.loads(out)
        return data.get("app_id", "")
    except Exception:
        return ""

def get_battery():
    try:
        with open("/sys/class/power_supply/BAT0/capacity", "r") as f:
            return int(f.read().strip())
    except Exception:
        return 100

def get_context():
    return {
        "time": time.strftime("%H:%M"),
        "active_app": get_active_app(),
        "battery_level": get_battery()
    }

def main():
    last_context = None
    try:
        with open("SYSTEM_PROMPT.md", "r") as f:
            prompt = f.read()
    except Exception:
        prompt = ""
    
    while True:
        ctx = get_context()
        if ctx != last_context:
            last_context = ctx
            try:
                res = requests.post(OLLAMA_URL, json={
                    "model": "llama3.2:1b",
                    "prompt": f"{prompt}\nContext: {json.dumps(ctx)}\nReturn ONLY the JSON object.",
                    "stream": False,
                    "format": "json"
                })
                if res.status_code == 200:
                    widgets = res.json().get("response", "{}")
                    with open(WIDGETS_FILE, "w") as f:
                        f.write(widgets)
            except Exception as e:
                print(f"Error: {e}")
        time.sleep(5)

if __name__ == "__main__":
    main()
