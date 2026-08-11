use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use tracing::{debug, info};

#[derive(Debug, serde::Serialize)]
struct HardwareProfile {
    cpu_model: String,
    pci_devices: Vec<String>,
    usb_devices: Vec<String>,
}

pub fn generate_hardware_hash() -> Result<String> {
    info!("Inizio scansione approfondita dell'hardware (Fase Vitreol)...");

    let cpu_model = scan_cpu()?;
    let mut pci_devices = scan_pci()?;
    let mut usb_devices = scan_usb()?;

    // Determinismo: l'ordine dei device nei bus può variare, ordiniamo alfabeticamente
    pci_devices.sort();
    usb_devices.sort();

    let profile = HardwareProfile {
        cpu_model,
        pci_devices,
        usb_devices,
    };

    debug!("Hardware Profile Generato: {:#?}", profile);

    // Serializzazione in JSON string per garantire determinismo di formattazione
    let profile_json = serde_json::to_string(&profile)
        .context("Fallimento serializzazione Hardware Profile")?;

    // Hashing SHA256 (Vitreol standard per cache P2P)
    let mut hasher = Sha256::new();
    hasher.update(profile_json.as_bytes());
    let result = hasher.finalize();
    
    let hash_string = hex::encode(result);
    info!("Hardware Hash Deterministicamente Calcolato: {}", hash_string);

    Ok(hash_string)
}

fn scan_cpu() -> Result<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default();
    
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                return Ok(parts[1].trim().to_string());
            }
        }
    }
    Ok("Unknown CPU".to_string())
}

fn scan_pci() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let pci_path = "/sys/bus/pci/devices/";
    
    if let Ok(entries) = fs::read_dir(pci_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let vendor_path = path.join("vendor");
            let device_path = path.join("device");
            
            if vendor_path.exists() && device_path.exists() {
                if let (Ok(vendor), Ok(device)) = (fs::read_to_string(vendor_path), fs::read_to_string(device_path)) {
                    // Rimuove lo 0x iniziale e \n
                    let v = vendor.trim().replace("0x", "");
                    let d = device.trim().replace("0x", "");
                    devices.push(format!("{}:{}", v, d));
                }
            }
        }
    }
    
    Ok(devices)
}

fn scan_usb() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let usb_path = "/sys/bus/usb/devices/";
    
    if let Ok(entries) = fs::read_dir(usb_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let vendor_path = path.join("idVendor");
            let product_path = path.join("idProduct");
            
            if vendor_path.exists() && product_path.exists() {
                if let (Ok(vendor), Ok(product)) = (fs::read_to_string(vendor_path), fs::read_to_string(product_path)) {
                    let v = vendor.trim();
                    let p = product.trim();
                    // Evita gli hub root fittizi
                    if v != "0000" && p != "0000" {
                        devices.push(format!("{}:{}", v, p));
                    }
                }
            }
        }
    }
    
    Ok(devices)
}
