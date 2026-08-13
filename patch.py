import os
import sys

def patch_wg_manager(path):
    with open(path, 'r') as f:
        content = f.read()
    
    target = '        info!("Node Dilithium5 ML-DSA Public Key: {}", dilithium_pk_base64);\n\n        info!("Level 13 Post-Quantum WireGuard mesh tunnel scaffolding initialized.");'
    replacement = '''        info!("Node Dilithium5 ML-DSA Public Key: {}", dilithium_pk_base64);

        if let Ok(conn) = zbus::Connection::session().await {
            let _ = conn.emit_signal(
                None::<()>,
                "/org/ermete/Security",
                "org.ermete.Security.Events",
                "TunnelPQCEstablished",
                &("Tunnel PQC Stabilito",),
            ).await;
        }

        info!("Level 13 Post-Quantum WireGuard mesh tunnel scaffolding initialized.");'''
    
    if target in content:
        content = content.replace(target, replacement)
        with open(path, 'w') as f:
            f.write(content)
        print("wg_manager.rs patched")
    else:
        print("Could not find target in wg_manager.rs")

def patch_tpm(path):
    with open(path, 'r') as f:
        content = f.read()
    
    target_err = '            return Err(anyhow::anyhow!(TpmError::HardwareMissing));'
    replacement_err = '''            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Ok(conn) = zbus::Connection::session().await {
                        let _ = conn.emit_signal(
                            None::<()>,
                            "/org/ermete/Security",
                            "org.ermete.Security.Events",
                            "TpmUnsealFailed",
                            &("Unseal TPM Fallito: Hardware Missing",),
                        ).await;
                    }
                })
            });
            return Err(anyhow::anyhow!(TpmError::HardwareMissing));'''

    target_success = '        info!("TPM 2.0: Key share unsealed successfully.");'
    replacement_success = '''        info!("TPM 2.0: Key share unsealed successfully.");
        let _ = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Ok(conn) = zbus::Connection::session().await {
                    let _ = conn.emit_signal(
                        None::<()>,
                        "/org/ermete/Security",
                        "org.ermete.Security.Events",
                        "TpmUnsealSuccess",
                        &("Unseal TPM Successo",),
                    ).await;
                }
            })
        });'''
    
    if target_err in content and target_success in content:
        content = content.replace(target_err, replacement_err)
        content = content.replace(target_success, replacement_success)
        with open(path, 'w') as f:
            f.write(content)
        print("tpm.rs patched")
    else:
        print("Could not find target in tpm.rs")

if __name__ == '__main__':
    wg_path = '/home/ermete/.gemini/antigravity-cli/brain/6c56cba7-8eff-4414-a62f-43c968bed459/.system_generated/worktrees/subagent-Security-Event-Emitter-self-fb39e92e/forge/specs/ermete-mesh-sync/ermete-mesh-sync-1.0.0/src/wg_manager.rs'
    tpm_path = '/home/ermete/.gemini/antigravity-cli/brain/6c56cba7-8eff-4414-a62f-43c968bed459/.system_generated/worktrees/subagent-Security-Event-Emitter-self-fb39e92e/system/ermete-greeter/src/tpm.rs'
    patch_wg_manager(wg_path)
    patch_tpm(tpm_path)
