use std::os::unix::fs::OpenOptionsExt;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ebpf/src");
    println!("cargo:rerun-if-changed=ebpf/Cargo.toml");

    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let out_path = PathBuf::from(&out_dir).join("ermete-ebpf-sched-bpf");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // Try building eBPF bytecode target bpfel-unknown-none
    let status = Command::new(&cargo)
        .env("LD_LIBRARY_PATH", "/usr/lib64")
        .args([
            "+nightly",
            "build",
            "-Z",
            "build-std=core",
            "--manifest-path",
            "ebpf/Cargo.toml",
            "--target",
            "bpfel-unknown-none",
            "--release",
        ])
        .status();

    if !matches!(status, Ok(s) if s.success()) {
        let _ = Command::new(&cargo)
            .env("LD_LIBRARY_PATH", "/usr/lib64")
            .args([
                "build",
                "--manifest-path",
                "ebpf/Cargo.toml",
                "--target",
                "bpfel-unknown-none",
                "--release",
            ])
            .status();
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let bpf_compiled_path = Path::new(&manifest_dir)
        .join("ebpf")
        .join("target")
        .join("bpfel-unknown-none")
        .join("release")
        .join("ermete-ebpf-sched-bpf");

    if bpf_compiled_path.exists() {
        if let Err(e) = fs::copy(&bpf_compiled_path, &out_path) {
            println!("cargo:warning=Failed to copy eBPF bytecode to OUT_DIR: {}", e);
        } else {
            println!("cargo:warning=eBPF scheduler bytecode compiled and copied to OUT_DIR successfully.");
        }
    } else {
        // Fallback: Write empty byte array so include_bytes! macro does not panic during cargo check/build
        if let Err(e) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&out_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, &[]))
        {
            println!("cargo:warning=Failed to write placeholder eBPF file to {:?}: {}", out_path, e);
        }
        println!("cargo:warning=eBPF scheduler build failed or produced no output; placeholder created in OUT_DIR.");
    }
}
