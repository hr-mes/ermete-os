use aya::programs::TracePoint;
use aya::Bpf;
use aya::maps::perf::PerfEventArray;
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
    let program: &mut TracePoint = bpf.program_mut("sched_process_exec").ok_or("program 'sched_process_exec' not found")?.try_into()?;
    program.load()?;
    program.attach("sched", "sched_process_exec")?;
    info!("eBPF hooks attached to sched:sched_process_exec.");
    
    // Read events from the kernel via PerfEventArray
    if let Ok(events_map) = bpf.map_mut("EVENTS") {
        let mut perf_array = PerfEventArray::try_from(events_map)?;
        for cpu_id in online_cpus()? {
            let mut buf = perf_array.open(cpu_id, None)?;
            task::spawn(async move {
                let mut buffers = (0..10).map(|_| BytesMut::with_capacity(1024)).collect::<Vec<_>>();
                loop {
                    match buf.read_events(&mut buffers) {
                        Ok(events) => {
                            for b in buffers.iter_mut().take(events.read) {
                                info!("Received event from kernel on CPU {} ({} bytes)", cpu_id, b.len());
                            }
                        }
                        Err(e) => {
                            warn!("Error reading perf events on CPU {}: {}", cpu_id, e);
                            break;
                        }
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
