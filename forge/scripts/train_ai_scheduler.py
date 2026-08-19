#!/usr/bin/env python3
"""
Ermete OS - AI Scheduler Base Model Generator
Trains a TinyML model (Linear Layer 4 -> 3) to classify Linux workloads.
Exports to .safetensors for zero-overhead inference in Rust (candle-core).
"""

import os
import torch
import torch.nn as nn
import torch.optim as optim
from safetensors.torch import save_file

# --- 1. DATASET UNIVERSALE (Conoscenza Base) ---
# Formato: (comm, filename, target_score, target_class, target_weight)
# Class: > 0.0 = InteractiveUi, <= 0.0 = BatchCompute/Background
DATA = [
    # UI e Interattivi (Alta priorità, latenza zero)
    ("niri", "/usr/bin/niri", 0.99, 1.0, 900),
    ("waybar", "/usr/bin/waybar", 0.95, 1.0, 800),
    ("firefox", "/usr/lib64/firefox/firefox", 0.90, 1.0, 700),
    ("kitty", "/usr/bin/kitty", 0.95, 1.0, 850),
    
    # Processi di Sistema (Media priorità)
    ("systemd", "/usr/lib/systemd/systemd", 0.50, -1.0, 500),
    ("sshd", "/usr/sbin/sshd", 0.60, -1.0, 400),
    
    # Batch / Calcolo Pesante (Bassa priorità, background)
    ("cargo", "/home/user/.cargo/bin/cargo", 0.90, -1.0, 100),
    ("rustc", "/usr/bin/rustc", 0.95, -1.0, 100),
    ("gcc", "/usr/bin/gcc", 0.95, -1.0, 100),
    ("ffmpeg", "/usr/bin/ffmpeg", 0.85, -1.0, 100),
    
    # Sconosciuti generici / user space
    ("my_script", "/home/user/my_script.sh", 0.10, -1.0, 300),
]

def extract_features(comm, filename):
    """Estrae le stesse feature usate dal demone Rust (f1, f2, f3, f4)"""
    f1 = float(len(comm))
    f2 = float(filename.count('/'))
    f3 = 1.0 if filename.startswith("/usr") else 0.0
    f4 = 1.0 if "wayland" in comm or "niri" in comm else 0.0
    return [f1, f2, f3, f4]

# --- 2. IL MODELLO (TinyML) ---
class SchedModel(nn.Module):
    def __init__(self):
        super().__init__()
        # 4 Input Features -> 3 Output (Score, Class, Weight)
        self.fc = nn.Linear(4, 3)

    def forward(self, x):
        return self.fc(x)

def train_model():
    print("🧠 Generazione del Modello Base Universale per Ermete OS...")
    
    # Preparazione tensori
    X = torch.tensor([extract_features(c, f) for c, f, _, _, _ in DATA], dtype=torch.float32)
    
    # Normalizzazione target per la loss function
    # target_score, target_class, normalized_weight (weight / 500.0)
    Y = torch.tensor([[s, c, w/500.0] for _, _, s, c, w in DATA], dtype=torch.float32)

    model = SchedModel()
    optimizer = optim.Adam(model.parameters(), lr=0.05)
    criterion = nn.MSELoss()

    print("🏋️ Addestramento (Epochs: 1000)...")
    for epoch in range(1000):
        optimizer.zero_grad()
        output = model(X)
        loss = criterion(output, Y)
        loss.backward()
        optimizer.step()
        
        if epoch % 200 == 0:
            print(f"   -> Epoch {epoch:4d} | Loss: {loss.item():.4f}")

    print("✅ Addestramento completato.")
    return model

def main():
    model = train_model()
    
    # --- 3. EXPORT IN SAFETENSORS ---
    out_dir = "system/ermete-ai-daemon/models"
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "base_model.safetensors")
    
    tensors = {
        "weight": model.fc.weight.contiguous(),
        "bias": model.fc.bias.contiguous()
    }
    
    save_file(tensors, out_path)
    
    size_bytes = os.path.getsize(out_path)
    print(f"📦 Modello esportato con successo in: {out_path}")
    print(f"⚖️  Dimensione Modello Quantizzato: {size_bytes} bytes (~{size_bytes/1024:.2f} KB)")

if __name__ == "__main__":
    main()
