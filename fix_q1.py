import os
import re

def replace_in_file(filepath, pattern, replacement):
    if not os.path.exists(filepath): return
    with open(filepath, 'r') as f:
        content = f.read()
    content = re.sub(pattern, replacement, content)
    with open(filepath, 'w') as f:
        f.write(content)

# 1. Hardcoded IPs
replace_in_file(
    "system/ermete-agentic-kernel/src/ai_client.rs",
    r'"192\.168\.1\.100"',
    r'std::env::var("ERMETE_AI_GATEWAY").unwrap_or_else(|_| "127.0.0.1".to_string()).as_str()'
)

replace_in_file(
    "system/ermete-net-unikernel/src/main.rs",
    r'"10\.0\.2\.1[01]"',
    r'std::env::var("ERMETE_UNIKERNEL_IP").unwrap_or_else(|_| "10.0.2.10".to_string()).as_str()'
)

replace_in_file(
    "system/ermete-cluster-mesh/src/discovery.rs",
    r'"127\.0\.0\.1"',
    r'std::env::var("ERMETE_DISCOVERY_IP").unwrap_or_else(|_| "0.0.0.0".to_string()).as_str()'
)

replace_in_file(
    "system/ermete-mesh-bus/src/peer.rs",
    r'"10\.99\.0\.2"',
    r'std::env::var("ERMETE_MESH_IP").unwrap_or_else(|_| "10.99.0.2".to_string()).as_str()'
)

replace_in_file(
    "system/ermete-telemetry/src/ai_engine.rs",
    r'"http://127\.0\.0\.1:11434/api/embeddings"',
    r'std::env::var("ERMETE_OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434/api/embeddings".to_string()).as_str()'
)

# 2. Critical Panic Vectors (expects & unwraps)
replace_in_file(
    "system/ermete-cluster-mesh/src/main.rs",
    r'std::env::var\("([^"]+)"\)\.expect\("[^"]+"\)',
    r'std::env::var("\1").unwrap_or_else(|_| "".to_string())'
)

replace_in_file(
    "system/ermete-kernel-forge/src/ostree_hook.rs",
    r'\.await\.unwrap\(\)',
    r'.await.map_err(|e| anyhow::anyhow!("OSTree error: {}", e))?'
)

replace_in_file(
    "system/ermete-net-unikernel/src/stack.rs",
    r'\.expect\("Failed to bind [^"]+"\)',
    r'.map_err(|e| anyhow::anyhow!("Failed to bind port: {}", e))?'
)

# 3. cvm_manager.rs unwrap
replace_in_file(
    "system/confidential_computing/ermete-attestation/src/cvm_manager.rs",
    r'let report = verified_report\.as_ref\(\)\.unwrap\(\);',
    r'let Some(report) = verified_report.as_ref() else { return Err(anyhow!("Critical logic error: report missing")); };'
)
print("Applied Q1 fixes.")
