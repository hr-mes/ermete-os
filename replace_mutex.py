import os
import re

dirs = [
    "forge/specs/ermete-shell-rs",
    "forge/specs/ermete-settings-rs"
]

for d in dirs:
    for root, _, files in os.walk(d):
        for file in files:
            if not file.endswith(".rs"): continue
            filepath = os.path.join(root, file)
            with open(filepath, "r") as f:
                content = f.read()

            if "Mutex" not in content: continue

            # Replace imports
            content = re.sub(r'std::sync::\{(.*?)Mutex(.*?)\}', r'std::sync::{\1\2}\nuse tokio::sync::Mutex;', content)
            content = re.sub(r'std::sync::Mutex', r'tokio::sync::Mutex', content)
            
            # Clean up empty imports
            content = re.sub(r'std::sync::\{\s*,\s*', r'std::sync::{', content)
            content = re.sub(r',\s*,\s*', r', ', content)
            content = re.sub(r'std::sync::\{\s*\}', r'', content)
            content = re.sub(r'std::sync::\{\s*([^,]+)\s*,\s*\}', r'std::sync::{\1}', content)

            # Replace locks
            content = re.sub(r'(\w+)\.lock\(\)\.unwrap\(\)', r'\1.blocking_lock()', content)
            content = re.sub(r'(\w+)\.lock\(\)\.expect\([^)]+\)', r'\1.blocking_lock()', content)
            content = re.sub(r'(\w+)\.lock\(\)\.unwrap_or_else\([^)]+\)', r'\1.blocking_lock()', content)
            
            # For if let Ok(mut c) = self.cached_volume.lock()
            # It's tricky to know if we are in async context for the script.
            # But tokio::sync::Mutex lock() returns a MutexGuard, not a Result.
            # So if let Ok(guard) = .lock() won't compile because it doesn't return Result.
            content = re.sub(r'if let Ok\((mut\s+)?(\w+)\)\s*=\s*(.*?)\.lock\(\)\s*\{', r'let \1\2 = \3.blocking_lock();\n        {', content)

            with open(filepath, "w") as f:
                f.write(content)
