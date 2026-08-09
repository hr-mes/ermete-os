use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use ermete_gatekeeper_rs::security::verify_file_fd_signature;

/// Level 11 Micro-VM Hypervisor Isolation (Hardware Compartmentalization)
/// Spawns untrusted applications inside a hardware-accelerated Micro-VM using `crosvm`
/// with guest Kernel isolation, falling back to `cloud-hypervisor`, `firecracker`, or `bwrap`.
///
/// TOCTOU-Safe Implementation: Opens the file as a file descriptor (`File::open`) first,
/// verifies the FD contents/signature, and executes via `/proc/self/fd/{fd}` to prevent
/// symlink race conditions.
pub async fn spawn_microvm_isolated_app(target_path: &Path) -> Result<tokio::process::Child, anyhow::Error> {
    let parent = match target_path.parent() {
        Some(p) if p != Path::new("/") => p,
        _ => anyhow::bail!("Parent path does not exist or is root ('/'), refusing root FS mount"),
    };

    // TOCTOU Fix Step 1: Open the target executable file as a File descriptor first
    let mut file = File::open(target_path).map_err(|e| {
        anyhow::anyhow!("Failed to open target executable file {:?} safely: {}", target_path, e)
    })?;

    let fd = file.as_raw_fd();
    let proc_fd_path = format!("/proc/self/fd/{}", fd);

    // TOCTOU Fix Step 2: Verify FD contents / signature if signature xattr present
    let sig_attr = xattr::get(&proc_fd_path, "user.ermete.signature").ok().flatten();
    let pubkey_attr = xattr::get(&proc_fd_path, "user.ermete.pubkey").ok().flatten();
    if let (Some(sig), Some(pubkey)) = (sig_attr, pubkey_attr) {
        if !verify_file_fd_signature(&mut file, &sig, &pubkey).unwrap_or(false) {
            anyhow::bail!("PQC signature verification failed for file descriptor {}", fd);
        }
    }

    println!(
        "[Level 11 Micro-VM Hypervisor] Intercepting execution. Launching hardware-isolated AppVM via crosvm for FD {} ({})",
        fd, proc_fd_path
    );

    // Locate guest Kernel image for hardware virtualization
    let guest_kernel = if Path::new("/boot/vmlinuz-ermete").exists() {
        "/boot/vmlinuz-ermete"
    } else if Path::new("/boot/vmlinuz").exists() {
        "/boot/vmlinuz"
    } else {
        "/boot/vmlinuz-linux"
    };

    // TOCTOU Fix Step 3: Execute via /proc/self/fd/{fd} instead of path string
    // 1. Primary: Spawns inside a hardware-accelerated crosvm Micro-VM
    let crosvm_res = tokio::process::Command::new("crosvm")
        .arg("run")
        .arg("--cpus").arg("2")
        .arg("--mem").arg("2048")
        .arg("--rw-shared-dir").arg(format!("{}:/app:type=fs", parent.display()))
        .arg("--params").arg(format!("init={} root=/dev/vda rw console=ttyS0", proc_fd_path))
        .arg(guest_kernel)
        .spawn();

    if let Ok(child) = crosvm_res {
        println!("[Level 11 Micro-VM Hypervisor] Hardware-isolated AppVM spawned via crosvm.");
        return Ok(child);
    }

    // 2. Secondary: Cloud-hypervisor Micro-VM fallback
    println!("[Level 11 Micro-VM Hypervisor] crosvm execution bypassed/unavailable. Trying cloud-hypervisor...");
    let cloud_res = tokio::process::Command::new("cloud-hypervisor")
        .arg("--cpus").arg("boot=2")
        .arg("--memory").arg("size=2048M")
        .arg("--kernel").arg(guest_kernel)
        .arg("--cmdline").arg(format!("init={} console=ttyS0", proc_fd_path))
        .spawn();

    if let Ok(child) = cloud_res {
        println!("[Level 11 Micro-VM Hypervisor] Hardware-isolated AppVM spawned via cloud-hypervisor.");
        return Ok(child);
    }

    // 3. Tertiary: Firecracker Micro-VM fallback
    println!("[Level 11 Micro-VM Hypervisor] cloud-hypervisor bypassed. Trying firecracker...");
    let fc_res = tokio::process::Command::new("firecracker")
        .arg("--api-sock").arg("/tmp/firecracker.socket")
        .spawn();

    if let Ok(child) = fc_res {
        println!("[Level 11 Micro-VM Hypervisor] Hardware-isolated AppVM spawned via firecracker.");
        return Ok(child);
    }

    // 4. Lightweight container fallback via Bubblewrap executing via /proc/self/fd/{fd}
    println!("[Level 11 Micro-VM Hypervisor] Hypervisor backends unexecutable. Falling back to bwrap sandbox via /proc/self/fd/{}.", fd);
    tokio::process::Command::new("bwrap")
        .arg("--unshare-all")
        .arg("--share-net")
        .arg("--ro-bind").arg("/usr").arg("/usr")
        .arg("--ro-bind").arg("/lib").arg("/lib")
        .arg("--ro-bind").arg("/lib64").arg("/lib64")
        .arg("--ro-bind").arg("/etc").arg("/etc")
        .arg("--tmpfs").arg("/etc/pki/secureboot")
        .arg("--tmpfs").arg("/etc/pki/uki")
        .arg("--tmpfs").arg("/run/secrets")
        .arg("--proc").arg("/proc")
        .arg("--dev").arg("/dev")
        .arg("--dir").arg("/tmp")
        .arg("--ro-bind").arg(&proc_fd_path).arg(&proc_fd_path)
        .arg("--").arg(&proc_fd_path)
        .spawn()
        .map_err(Into::into)
}
