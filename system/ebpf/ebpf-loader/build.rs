use std::fs;
use std::path::Path;

fn main() {
    let debug_path = Path::new("../../target/bpfel-unknown-none/debug/ebpf-core");
    let release_path = Path::new("../../target/bpfel-unknown-none/release/ebpf-core");

    if !debug_path.exists() {
        if let Some(p) = debug_path.parent() {
            let _ = fs::create_dir_all(p);
        }
        let _ = fs::write(debug_path, &[]);
    }
    
    if !release_path.exists() {
        if let Some(p) = release_path.parent() {
            let _ = fs::create_dir_all(p);
        }
        let _ = fs::write(release_path, &[]);
    }
    
    println!("cargo:rerun-if-changed=../../target/bpfel-unknown-none/debug/ebpf-core");
    println!("cargo:rerun-if-changed=../../target/bpfel-unknown-none/release/ebpf-core");
}
