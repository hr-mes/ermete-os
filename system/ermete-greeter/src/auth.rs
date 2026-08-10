use anyhow::{anyhow, Result};
use log::{info, warn};
use std::fs;

/// Performs real PAM authentication for UNIX accounts.
pub fn authenticate_user(username: &str, password: &str) -> Result<()> {
    if username.trim().is_empty() {
        return Err(anyhow!("Authentication failed: Username cannot be empty"));
    }
    if password.is_empty() {
        return Err(anyhow!("Authentication failed: Password cannot be empty"));
    }

    info!("Performing PAM authentication for user '{}'...", username);

    let services = ["login", "system-auth", "passwd", "sudo", "other"];
    let mut pam_attempts = Vec::new();

    for service in &services {
        match pam::Authenticator::with_password(service) {
            Ok(mut auth) => {
                auth.get_handler().set_credentials(username, password);
                match auth.authenticate() {
                    Ok(_) => {
                        info!("PAM authentication succeeded via service '{}' for user '{}'", service, username);
                        return Ok(());
                    }
                    Err(err) => {
                        pam_attempts.push(format!("{}: {:?}", service, err));
                    }
                }
            }
            Err(err) => {
                pam_attempts.push(format!("{}: failed to initialize ({:?})", service, err));
            }
        }
    }

    warn!("PAM authentication failed for user '{}'. Attempts: {:?}", username, pam_attempts);

    // Inspect /etc/shadow for locked or disabled accounts
    if let Ok(shadow_data) = fs::read_to_string("/etc/shadow") {
        for line in shadow_data.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[0] == username {
                let shadow_hash = parts[1];
                if shadow_hash == "*" || shadow_hash == "!" || shadow_hash.starts_with('!') || shadow_hash.is_empty() {
                    return Err(anyhow!("Account '{}' is locked or disabled in /etc/shadow", username));
                }
            }
        }
    }

    Err(anyhow!(
        "Authentication failed for user '{}': PAM rejected credentials ({})",
        username,
        pam_attempts.join(", ")
    ))
}
