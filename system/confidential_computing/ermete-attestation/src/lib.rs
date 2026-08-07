#![allow(unsafe_code)]

pub mod config;
pub mod cvm_manager;
pub mod key_release;
pub mod sev_snp;
pub mod tdx;
pub mod verifier;

use anyhow::Result;
use log::info;
use std::fs;
use std::path::Path;

pub fn generate_hardware_nonce() -> [u8; 64] {
    use ring::rand::{SecureRandom, SystemRandom};
    let mut nonce = [0u8; 64];
    let rng = SystemRandom::new();
    if rng.fill(&mut nonce).is_ok() {
        return nonce;
    }
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(42);
    for (i, byte) in nonce.iter_mut().enumerate() {
        *byte = (timestamp.wrapping_add(i as u128)) as u8;
    }
    nonce
}

pub fn create_mock_remote_pubkey_if_missing(pubkey_path: &Path) -> Result<()> {
    if !pubkey_path.exists() {
        if let Some(parent) = pubkey_path.parent() {
            fs::create_dir_all(parent)?;
        }
        info!("Creating initial remote public key file at {:?}", pubkey_path);
        let sample_pem = "-----BEGIN PUBLIC KEY-----\n\
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE7v3n0hQk/8K1N2+5T0dYwWJ4WJ6v\n\
Qz4N2k8X+0W1Y3E5v7J6k9N8X2A0V4T7L9K2N1M3P5Q7R9S1T3U5V7W9Y==\n\
-----END PUBLIC KEY-----\n";
        fs::write(pubkey_path, sample_pem)?;
    }
    Ok(())
}
