#!/usr/bin/env python3
import os
import sys
import json
import urllib.request
import urllib.error
import argparse
from pathlib import Path

# ==============================================================================
# ERMETE FORGE - AUTONOMOUS AI HEALER
# ==============================================================================
# Intercetta i log di compilazione falliti da ermete-forge, li passa al modello 
# locale (llama-server) e tenta l'auto-guarigione del file .spec.
# ==============================================================================

LLAMA_API_URL = "http://127.0.0.1:8080/v1/chat/completions"

def call_local_llm(system_prompt: str, user_prompt: str) -> str:
    """Chiama il modello locale in ascolto sulle GPU tramite API OpenAI compatibile."""
    payload = {
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.1, # Bassissima temperatura per risposte deterministiche
        "max_tokens": 4096
    }
    
    headers = {"Content-Type": "application/json"}
    req = urllib.request.Request(
        LLAMA_API_URL, 
        data=json.dumps(payload).encode('utf-8'), 
        headers=headers, 
        method='POST'
    )
    
    try:
        with urllib.request.urlopen(req) as response:
            result = json.loads(response.read().decode('utf-8'))
            return result['choices'][0]['message']['content'].strip()
    except urllib.error.URLError as e:
        print(f"[!] Errore di connessione a llama-server: {e}")
        print("[!] Assicurati che il modello locale sia in esecuzione.")
        sys.exit(1)

def heal_spec_file(package_name: str, log_file: Path, specs_dir: Path):
    print(f"=========================================================")
    print(f" 🌋 ERMETE FORGE AI HEALER: Innesco Autoguarigione")
    print(f"=========================================================")
    print(f">>> Pacchetto target : {package_name}")
    print(f">>> File Log Analisi : {log_file}")
    
    spec_file = specs_dir / package_name / f"{package_name}.spec"
    
    if not spec_file.exists():
        print(f"[!] Spec file non trovato in {spec_file}")
        sys.exit(1)
        
    if not log_file.exists():
        print(f"[!] Log file non trovato in {log_file}")
        sys.exit(1)

    # Leggi gli ultimi 200 righe del log di errore (le più rilevanti)
    with open(log_file, "r") as f:
        log_lines = f.readlines()
        error_log = "".join(log_lines[-200:])

    with open(spec_file, "r") as f:
        original_spec = f.read()

    system_prompt = (
        "Sei 'Ermete Forge Healer', un ingegnere esperto di pacchettizzazione RPM. "
        "Il tuo compito è analizzare un log di compilazione fallito e riparare il file .spec.\n"
        "REGOLE CRITICHE:\n"
        "1. Restituisci ESATTAMENTE e SOLAMENTE il contenuto completo del file .spec corretto.\n"
        "2. Nessun blocco markdown (```), nessuna spiegazione testuale, nessuna scusa. Solo puro testo del file .spec.\n"
        "3. Rispetta la filosofia Ermete: se c'è un errore di build, disabilita tool inutili (es. %_without_tests 1) o rimuovi flag incompatibili."
    )

    user_prompt = (
        f"--- ERRORE DI COMPILAZIONE (Ultimi 200 righi) ---\n"
        f"{error_log}\n\n"
        f"--- FILE .SPEC ATTUALE ({spec_file.name}) ---\n"
        f"{original_spec}\n\n"
        "Fornisci il nuovo contenuto del file .spec corretto."
    )

    print(">>> Contattando l'oracolo locale sulle GPU...")
    new_spec_content = call_local_llm(system_prompt, user_prompt)
    
    # Rimuove eventuali blocchi markdown residui (nel caso l'LLM disobbedisca)
    if new_spec_content.startswith("```"):
        lines = new_spec_content.splitlines()
        new_spec_content = "\n".join(lines[1:-1] if lines[-1].startswith("```") else lines[1:])

    # Salvataggio del nuovo file
    with open(spec_file, "w") as f:
        f.write(new_spec_content)
        
    print(f">>> [SUCCESSO] File {spec_file.name} riparato e sovrascritto.")
    print(f">>> Pronto per ritentare la compilazione OCI.")
    print(f"=========================================================")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ermete Forge AI Healer")
    parser.add_argument("package", help="Nome del pacchetto (es. ermete-shell-rs)")
    parser.add_argument("logfile", help="Percorso del log di build fallito")
    parser.add_argument("--specs", default="../specs", help="Percorso alla directory specs/")
    
    args = parser.parse_args()
    
    heal_spec_file(args.package, Path(args.logfile), Path(args.specs).resolve())
