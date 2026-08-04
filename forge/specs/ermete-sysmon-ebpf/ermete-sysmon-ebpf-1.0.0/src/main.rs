use aya::programs::TracePoint;
use aya::Bpf;
use aya::maps::perf::AsyncPerfEventArray;
use aya::util::online_cpus;
use bytes::BytesMut;
use tokio::signal;
use tokio::task;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Ermete Sysmon eBPF (Ring-0 Analytics) starting...");

    // Load the compiled eBPF bytecode.
    // (Falling back to empty load for compilation/stub logic if path missing)
    let bpf_path = "target/bpfel-unknown-none/release/ermete-sysmon-ebpf";
    let mut bpf = Bpf::load_file(bpf_path).or_else(|_| Bpf::load(&[]))?;
    
    // Attach to the tracepoint
    let program: &mut TracePoint = bpf.program_mut("sched_process_exec").unwrap().try_into()?;
    program.load()?;
    program.attach("sched", "sched_process_exec")?;
    info!("eBPF hooks attached to sched:sched_process_exec.");
    
    // Read events from the kernel via AsyncPerfEventArray
    if let Ok(events_map) = bpf.take_map("EVENTS") {
        let mut perf_array = AsyncPerfEventArray::try_from(events_map)?;
        for cpu_id in online_cpus()? {
            let mut buf = perf_array.open(cpu_id, None)?;
            task::spawn(async move {
                let mut buffers = (0..10).map(|_| BytesMut::with_capacity(1024)).collect::<Vec<_>>();
                loop {
                    let events = buf.read_events(&mut buffers).await.unwrap();
                    for b in buffers.iter_mut().take(events.read) {
                        info!("Received event from kernel on CPU {} ({} bytes)", cpu_id, b.len());
                    }
                }
            });
        }
        info!("Awaiting events...");
    } else {
        warn!("EVENTS map not found in the eBPF program, running without events hook.");
    }

    // Wait for SIGINT
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
