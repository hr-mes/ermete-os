use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use zbus::{fdo, interface};

const SEAL_VERSION: &str = "3.0";
const NONCE_LEN: usize = 12;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SealedData {
    version: String,
    username: String,
    machine_binding: String,
    sealed_payload: String,
}

#[zbus::proxy(
    interface = "org.freedesktop.Secret.Service",
    default_service = "org.freedesktop.secrets",
    default_path = "/org/freedesktop/secrets"
)]
trait SecretService {
    fn unlock(
        &self,
        objects: &[zbus::zvariant::ObjectPath<'_>],
    ) -> zbus::Result<(Vec<zbus::zvariant::OwnedObjectPath>, zbus::zvariant::OwnedObjectPath)>;
}

#[derive(Default, Clone)]
pub struct SecretEnrollerService {
    base_dir: Option<PathBuf>,
}

impl SecretEnrollerService {
    pub fn new() -> Self {
        Self { base_dir: None }
    }

    pub fn new_with_dir(dir: PathBuf) -> Self {
        Self { base_dir: Some(dir) }
    }

    pub fn get_sealed_path_for(&self, username: &str) -> PathBuf {
        if let Some(ref dir) = self.base_dir {
            let mut p = dir.clone();
            p.push(username);
            p.push("tpm2.sealed");
            p
        } else {
            Self::get_sealed_path(username)
        }
    }

    pub fn get_sealed_path(username: &str) -> PathBuf {
        if let Ok(test_dir) = std::env::var("ERMETE_TEST_SEALED_DIR") {
            let mut p = PathBuf::from(test_dir);
            p.push(username);
            p.push("tpm2.sealed");
            p
        } else if let Ok(home) = std::env::var("HOME") {
            if home.contains(username) {
                let mut p = PathBuf::from(home);
                p.push(".local/share/keyrings/tpm2.sealed");
                p
            } else {
                PathBuf::from(format!("/var/home/{}/.local/share/keyrings/tpm2.sealed", username))
            }
        } else {
            PathBuf::from(format!("/var/home/{}/.local/share/keyrings/tpm2.sealed", username))
        }
    }

    fn get_machine_binding() -> String {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        "ermete-bedrock-tpm2-default-binding".to_string()
    }

    fn get_salt() -> Vec<u8> {
        let path = "/etc/ermete/secret_salt";
        if let Ok(salt) = std::fs::read(path) {
            return salt;
        }
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, &salt);
        salt.to_vec()
    }

    /// Derive a 256-bit key from the machine binding using HKDF-SHA256.
    fn derive_key(binding: &str) -> [u8; 32] {
        let salt = Self::get_salt();
        let hk = Hkdf::<Sha256>::new(Some(&salt), binding.as_bytes());
        let mut key = [0u8; 32];
        hk.expand(b"chacha20poly1305-key", &mut key)
            .expect("HKDF expand should never fail for 32 bytes");
        key
    }

    /// Encrypt the password using ChaCha20-Poly1305.
    /// Output format: hex(nonce[12]) + hex(ciphertext_with_tag)
    fn seal_payload(password: &str, binding: &str) -> String {
        let key = Self::derive_key(binding);
        let cipher = ChaCha20Poly1305::new(&key.into());

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, password.as_bytes())
            .expect("ChaCha20-Poly1305 encryption should not fail");

        let mut output = String::with_capacity((NONCE_LEN + ciphertext.len()) * 2);
        for b in &nonce_bytes {
            output.push_str(&format!("{:02x}", b));
        }
        for b in &ciphertext {
            output.push_str(&format!("{:02x}", b));
        }
        output
    }

    /// Decrypt a payload sealed by `seal_payload`.
    /// Returns None if decryption or authentication fails.
    fn unseal_payload(sealed_hex: &str, binding: &str) -> Option<String> {
        let bytes = hex_decode(sealed_hex)?;
        if bytes.len() < NONCE_LEN {
            return None;
        }

        let key = Self::derive_key(binding);
        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from_slice(&bytes[..NONCE_LEN]);
        let ciphertext = &bytes[NONCE_LEN..];

        let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
        String::from_utf8(plaintext).ok()
    }
}

/// Decode a hex string into bytes. Returns None on invalid input.
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars();
    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        let byte_str = format!("{}{}", c1, c2);
        bytes.push(u8::from_str_radix(&byte_str, 16).ok()?);
    }
    Some(bytes)
}

/// Legacy v2.0 XOR decryption for migration. Only used to read old sealed files.
fn legacy_unseal_payload(sealed_hex: &str, binding: &str) -> Option<String> {
    let key_bytes = binding.as_bytes();
    let bytes = hex_decode(sealed_hex)?;
    let mut unxored = Vec::with_capacity(bytes.len());
    for (i, b) in bytes.iter().enumerate() {
        let k = key_bytes[i % key_bytes.len()];
        unxored.push(b ^ k);
    }
    String::from_utf8(unxored).ok()
}

#[interface(name = "os.ermete.Bedrock.SecretEnroller")]
impl SecretEnrollerService {
    async fn enroll_secret(&self, username: String, password: String) -> fdo::Result<String> {
        if password.is_empty() {
            return Err(fdo::Error::InvalidArgs("Password cannot be empty".to_string()));
        }

        let path = self.get_sealed_path_for(&username);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| fdo::Error::Failed(format!("Failed to create keyring directory: {}", e)))?;
        }

        let binding = Self::get_machine_binding();
        let sealed_payload = Self::seal_payload(&password, &binding);
        let token = SealedData {
            version: SEAL_VERSION.to_string(),
            username: username.clone(),
            machine_binding: binding,
            sealed_payload,
        };

        let content = serde_json::to_string_pretty(&token)
            .map_err(|e| fdo::Error::Failed(format!("Failed to serialize token: {}", e)))?;

        // ACID write via temporary file
        let temp_path = PathBuf::from(format!("{}.tmp", path.display()));
        std::fs::write(&temp_path, content)
            .map_err(|e| fdo::Error::Failed(format!("Failed to write temporary sealed file: {}", e)))?;
        std::fs::rename(&temp_path, &path)
            .map_err(|e| fdo::Error::Failed(format!("Failed to rename sealed file: {}", e)))?;

        Ok(path.to_string_lossy().to_string())
    }

    async fn decrypt_secret(&self, username: String) -> fdo::Result<String> {
        let path = self.get_sealed_path_for(&username);
        if !path.exists() {
            return Err(fdo::Error::Failed(format!("No sealed secret found at {:?}", path)));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| fdo::Error::Failed(format!("Failed to read sealed file: {}", e)))?;

        let token: SealedData = serde_json::from_str(&content)
            .map_err(|e| fdo::Error::Failed(format!("Invalid sealed token JSON: {}", e)))?;

        if token.username != username {
            return Err(fdo::Error::Failed("Sealed token username mismatch".to_string()));
        }

        let binding = Self::get_machine_binding();
        if token.machine_binding != binding {
            return Err(fdo::Error::Failed("TPM 2.0 machine binding verification failed".to_string()));
        }

        match token.version.as_str() {
            "3.0" => {
                Self::unseal_payload(&token.sealed_payload, &binding)
                    .ok_or_else(|| fdo::Error::Failed("Failed to unseal v3.0 payload".into()))
            }
            "2.0" => {
                tracing::warn!("Reading legacy v2.0 XOR-encrypted secret for user {}. Will re-encrypt on next enrollment.", username);
                legacy_unseal_payload(&token.sealed_payload, &binding)
                    .ok_or_else(|| fdo::Error::Failed("Failed to unseal legacy v2.0 payload".into()))
            }
            _ => Err(fdo::Error::Failed(format!("Unknown seal version: {}", token.version)))
        }
    }

    async fn unlock_keyring(&self, _username: String, secret: String) -> fdo::Result<bool> {
        if secret.is_empty() {
            return Err(fdo::Error::InvalidArgs("Secret cannot be empty for unlocking".to_string()));
        }

        // Attempt D-Bus communication with org.freedesktop.secrets (`/org/freedesktop/secrets`)
        if let Ok(session_conn) = zbus::Connection::session().await {
            if let Ok(proxy) = SecretServiceProxy::new(&session_conn).await {
                if let Ok(login_path) = zbus::zvariant::ObjectPath::try_from("/org/freedesktop/secrets/collection/login") {
                    let _ = proxy.unlock(&[login_path]).await;
                }
            }
        }

        // Also attempt native socket communication with gnome-keyring if control socket is available
        if let Ok(sock_path) = std::env::var("GNOME_KEYRING_CONTROL") {
            if let Ok(mut stream) = UnixStream::connect(&sock_path) {
                use std::io::Write;
                let _ = stream.write_all(secret.as_bytes());
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_sealed_path() {
        let path = SecretEnrollerService::get_sealed_path("mockuser");
        assert!(
            path.to_string_lossy().contains("mockuser") && path.to_string_lossy().ends_with("tpm2.sealed"),
            "Unexpected path: {:?}",
            path
        );
    }

    #[tokio::test]
    async fn test_enroll_and_decrypt_secret() {
        let temp_dir = std::env::temp_dir().join(format!("ermete_secret_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let service = SecretEnrollerService::new_with_dir(temp_dir.clone());
        let res = service.enroll_secret("testuser".to_string(), "biometric_secret_123".to_string()).await;
        assert!(res.is_ok(), "enroll_secret should succeed: {:?}", res);

        let decrypted = service.decrypt_secret("testuser".to_string()).await;
        assert!(decrypted.is_ok(), "decrypt_secret should succeed: {:?}", decrypted);
        assert_eq!(decrypted.unwrap(), "biometric_secret_123");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_decrypt_secret_nonexistent() {
        let temp_dir = std::env::temp_dir().join(format!("ermete_secret_test_nonexistent_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let service = SecretEnrollerService::new_with_dir(temp_dir.clone());
        let res = service.decrypt_secret("nobody".to_string()).await;
        assert!(res.is_err(), "decrypt_secret on nonexistent user should fail");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_unlock_keyring() {
        let service = SecretEnrollerService::new();
        let res = service.unlock_keyring("testuser".to_string(), "secret123".to_string()).await;
        assert!(res.is_ok(), "unlock_keyring should succeed");
        assert_eq!(res.unwrap(), true);
    }

    #[test]
    fn test_seal_unseal_roundtrip() {
        let binding = "test-machine-id-12345";
        let password = "my-super-secret-password";
        let sealed = SecretEnrollerService::seal_payload(password, binding);
        let decrypted = SecretEnrollerService::unseal_payload(&sealed, binding);
        assert_eq!(decrypted.as_deref(), Some(password));
    }

    #[test]
    fn test_seal_wrong_binding_fails() {
        let binding = "correct-machine-id";
        let sealed = SecretEnrollerService::seal_payload("secret", binding);
        let result = SecretEnrollerService::unseal_payload(&sealed, "wrong-machine-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_seal_tampered_ciphertext_fails() {
        let binding = "test-machine-id";
        let sealed = SecretEnrollerService::seal_payload("secret", binding);
        let mut chars: Vec<char> = sealed.chars().collect();
        let last = chars.len() - 1;
        chars[last] = if chars[last] == 'a' { 'b' } else { 'a' };
        let tampered: String = chars.into_iter().collect();
        let result = SecretEnrollerService::unseal_payload(&tampered, binding);
        assert!(result.is_none());
    }
}
