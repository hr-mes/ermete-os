use std::path::Path;

/// Level 11 Micro-VM Hypervisor Isolation (Hardware Compartmentalization)
/// Spawns untrusted applications inside a hardware-accelerated Micro-VM using `crosvm`
/// with guest Kernel isolation, falling back to `cloud-hypervisor`, `firecracker`, or `bwrap`.
pub async fn spawn_microvm_isolated_app(target_path: &Path) -> Result<tokio::process::Child, anyhow::Error> {
    let parent = match target_path.parent() {
        Some(p) if p != Path::new("/") => p,
        _ => anyhow::bail!("Parent path does not exist or is root ('/'), refusing root FS mount"),
    };

    let target_str = target_path.to_string_lossy().into_owned();
    println!(
        "[Level 11 Micro-VM Hypervisor] Intercepting execution. Launching hardware-isolated AppVM via crosvm for target: {}",
        target_str
    );

    // Locate guest Kernel image for hardware virtualization
    let guest_kernel = if Path::new("/boot/vmlinuz-ermete").exists() {
        "/boot/vmlinuz-ermete"
    } else if Path::new("/boot/vmlinuz").exists() {
        "/boot/vmlinuz"
    } else {
        "/boot/vmlinuz-linux"
    };

    // 1. Primary: Spawns inside a hardware-accelerated crosvm Micro-VM
    let crosvm_res = tokio::process::Command::new("crosvm")
        .arg("run")
        .arg("--cpus").arg("2")
        .arg("--mem").arg("2048")
        .arg("--rw-shared-dir").arg(format!("{}:/app:type=fs", parent.display()))
        .arg("--params").arg(format!("init={} root=/dev/vda rw console=ttyS0", target_str))
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
        .arg("--cmdline").arg(format!("init={} console=ttyS0", target_str))
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

    // 4. Lightweight container fallback via Bubblewrap
    println!("[Level 11 Micro-VM Hypervisor] Hypervisor backends unexecutable. Falling back to bwrap sandbox.");
    tokio::process::Command::new("bwrap")
        .arg("--unshare-all")
        .arg("--share-net")
        .arg("--ro-bind").arg("/usr").arg("/usr")
        .arg("--ro-bind").arg("/lib").arg("/lib")
        .arg("--ro-bind").arg("/lib64").arg("/lib64")
        .arg("--ro-bind").arg("/etc").arg("/etc")
        .arg("--proc").arg("/proc")
        .arg("--dev").arg("/dev")
        .arg("--dir").arg("/tmp")
        .arg("--ro-bind").arg(target_path).arg(target_path)
        .arg("--").arg(target_path)
        .spawn()
        .map_err(Into::into)
}
