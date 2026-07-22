import os
import json
import time
import requests

# Configuration
CONFIG_PATH = os.path.expanduser("~/.config/ermete/widgets.json")
OLLAMA_API_URL = "http://localhost:11434/api/generate"
MODEL_NAME = "llama3.2:1b"  # Fast, edge-optimized model for zero-latency UI generation

def get_system_context():
    """
    Gathers zero-cost context from the OS.
    In a fully integrated version, this hooks into Wayland/Niri IPC to get active_app.
    """
    # Placeholder simulation for PoC
    return {
        "time": time.strftime("%H:%M"),
        "active_app": "Terminal", 
        "battery_level": 85
    }

def read_system_prompt():
    prompt_path = os.path.join(os.path.dirname(__file__), "SYSTEM_PROMPT.md")
    try:
        with open(prompt_path, "r") as f:
            return f.read()
    except FileNotFoundError:
        return "You are an AI generating JSON. Respond only in JSON format."

def query_llm(context):
    system_prompt = read_system_prompt()
    user_prompt = f"Contesto attuale: {json.dumps(context)}\nGenera il nuovo layout."
    
    payload = {
        "model": MODEL_NAME,
        "system": system_prompt,
        "prompt": user_prompt,
        "stream": False,
        "format": "json"  # Native JSON mode ensures no Markdown/text artifacts
    }
    
    try:
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=10)
        if response.status_code == 200:
            return response.json().get("response", "{}")
    except requests.exceptions.RequestException as e:
        print(f"Errore di connessione a Ollama: {e}")
    return None

def update_widgets(json_str):
    if not json_str:
        return
    try:
        # Validate JSON strictness before applying it to the GTK4 Engine
        data = json.loads(json_str)
        if "widgets" in data:
            os.makedirs(os.path.dirname(CONFIG_PATH), exist_ok=True)
            with open(CONFIG_PATH, "w") as f:
                json.dump(data, f, indent=4)
            print("[\u2713] Widget layout aggiornato con successo dall'IA!")
    except json.JSONDecodeError:
        print("[\u2717] Il modello ha restituito un JSON non valido. Rigettato per sicurezza.")

def main():
    print("Ermete UI Agent Avviato. In attesa di cambi di contesto...")
    last_context = None
    
    while True:
        current_context = get_system_context()
        
        # Trigger generation ONLY on context mutation to save battery
        if current_context != last_context:
            print(f"\nCambio di contesto rilevato: {current_context}")
            print(f"Interrogazione del modello {MODEL_NAME} in corso...")
            
            new_layout_json = query_llm(current_context)
            update_widgets(new_layout_json)
            
            last_context = current_context
            
        time.sleep(5)  # Lightweight sleep loop

if __name__ == "__main__":
    main()
