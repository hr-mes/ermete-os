import os
import json
import asyncio
import aiohttp
import tempfile

# Configuration
CONFIG_PATH = os.path.expanduser("~/.config/ermete/widgets.json")
OLLAMA_API_URL = "http://localhost:11434/api/generate"
MODEL_NAME = "llama3.2:1b"
IPC_SOCKET = os.path.expanduser("~/.run/ermete-ui-agent.sock")

current_context = {
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
    
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(OLLAMA_API_URL, json=payload, timeout=10) as response:
                if response.status == 200:
                    resp_data = await response.json()
                    return resp_data.get("response", "{}")
    except Exception as e:
        print(f"Errore di connessione a Ollama: {e}")
    return None

def update_widgets(json_str):
    if not json_str:
        return
    try:
        data = json.loads(json_str)
        if "widgets" in data:
            os.makedirs(os.path.dirname(CONFIG_PATH), exist_ok=True)
            fd, temp_path = tempfile.mkstemp(dir=os.path.dirname(CONFIG_PATH))
            try:
                with os.fdopen(fd, "w") as f:
                    json.dump(data, f, indent=4)
                os.replace(temp_path, CONFIG_PATH)
                print("[\u2713] Widget layout aggiornato con successo dall'IA!")
            except Exception:
                os.unlink(temp_path)
                raise
    except json.JSONDecodeError:
        print("[\u2717] Il modello ha restituito un JSON non valido.")

async def handle_ipc(reader, writer):
    global current_context
    data = await reader.read(1024)
    message = data.decode().strip()
    try:
        new_ctx = json.loads(message)
        current_context.update(new_ctx)
        print(f"Context updated via IPC: {current_context}")
        
        # Trigger LLM generation
        new_layout = await query_llm(current_context)
        update_widgets(new_layout)
        writer.write(b"OK\n")
    except json.JSONDecodeError:
        writer.write(b"ERROR: Invalid JSON\n")
    
    await writer.drain()
    writer.close()

async def main():
    print("Ermete UI Agent Avviato in attesa di eventi IPC...")
    
    # Setup IPC socket
    os.makedirs(os.path.dirname(IPC_SOCKET), exist_ok=True)
    if os.path.exists(IPC_SOCKET):
        os.remove(IPC_SOCKET)
        
    server = await asyncio.start_unix_server(handle_ipc, path=IPC_SOCKET)
    
    async with server:
        await server.serve_forever()

if __name__ == "__main__":
    asyncio.run(main())
