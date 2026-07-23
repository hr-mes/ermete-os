import os
import json
import time
import urllib.request
import urllib.error
import tempfile
import asyncio

# Configuration
CONFIG_PATH = os.path.expanduser("~/.config/ermete/widgets.json")
OLLAMA_API_URL = "http://localhost:11434/api/generate"
MODEL_NAME = "llama3.2:1b"  # Fast, edge-optimized model for zero-latency UI generation

def get_system_context():
    """
    Gathers zero-cost context from the OS.
    In a fully integrated version, this hooks into Wayland/Niri IPC to get active_app.
    """
    # Removed time to prevent constant battery drain
    return {
        "active_app": "Terminal", 
        "battery_level": 85
    }

_CACHED_PROMPT = None
def read_system_prompt():
    global _CACHED_PROMPT
    if _CACHED_PROMPT is not None:
        return _CACHED_PROMPT
    prompt_path = os.path.join(os.path.dirname(__file__), "SYSTEM_PROMPT.md")
    try:
        with open(prompt_path, "r") as f:
            _CACHED_PROMPT = f.read()
            return _CACHED_PROMPT
    except FileNotFoundError:
        return "You are an AI generating JSON. Respond only in JSON format."

async def query_llm(context):
    system_prompt = read_system_prompt()
    user_prompt = f"Contesto attuale: {json.dumps(context)}\nGenera il nuovo layout."
    
    payload = {
        "model": MODEL_NAME,
        "system": system_prompt,
        "prompt": user_prompt,
        "stream": False,
        "format": "json"
    }
    
    def fetch():
        req = urllib.request.Request(
            OLLAMA_API_URL, 
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode('utf-8'))
                    return data.get("response", "{}")
        except urllib.error.URLError as e:
            print(f"Errore di connessione a Ollama: {e}")
        return None

    return await asyncio.to_thread(fetch)

def update_widgets(json_str):
    if not json_str:
        return
    try:
        data = json.loads(json_str)
        if "widgets" in data:
            os.makedirs(os.path.dirname(CONFIG_PATH), exist_ok=True)
            fd, tmp_path = tempfile.mkstemp(dir=os.path.dirname(CONFIG_PATH))
            with os.fdopen(fd, "w") as f:
                json.dump(data, f, indent=4)
            os.replace(tmp_path, CONFIG_PATH)
            print("[\u2713] Widget layout aggiornato con successo dall'IA!")
    except json.JSONDecodeError:
        print("[\u2717] Il modello ha restituito un JSON non valido. Rigettato per sicurezza.")

async def main():
    print("Ermete UI Agent Avviato. In attesa di cambi di contesto...")
    last_context = None
    
    while True:
        current_context = get_system_context()
        
        if current_context != last_context:
            print(f"\nCambio di contesto rilevato: {current_context}")
            print(f"Interrogazione del modello {MODEL_NAME} in corso...")
            
            new_layout_json = await query_llm(current_context)
            update_widgets(new_layout_json)
            
            last_context = current_context
            
        await asyncio.sleep(5)

if __name__ == "__main__":
    asyncio.run(main())
