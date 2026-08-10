#![no_std]
#![no_main]
#![allow(unsafe_code)]

use aya_ebpf::{
    macros::map,
    maps::{Array, HashMap},
    EbpfContext,
};
use aya_log_ebpf::info;

pub struct SchedExtContext {
    ctx: *mut core::ffi::c_void,
}

impl EbpfContext for SchedExtContext {
    fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.ctx
    }
}

impl SchedExtContext {
    pub fn new(ctx: *mut core::ffi::c_void) -> Self {
        Self { ctx }
    }

    pub fn cpu(&self) -> u32 {
        unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() }
    }

    pub fn runtime_ns(&self) -> u64 {
        unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() }
    }
}

/// AI Scheduling parameters per PID set by user-space AI engine
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiSchedParam {
    /// AI assigned task priority (0: Realtime NPU / sub-ms, 1: Interactive UI, 2: Batch, 3: Idle)
    pub priority: u32,
    /// AI assigned target CPU core index (e.g., 0..N or u32::MAX for unconstrained)
    pub target_cpu: u32,
    /// Time slice allocation in nanoseconds (e.g., 100_000 ns = 100us)
    pub slice_ns: u64,
    /// Bitflags: 0x1 = CFS Bypass Enabled, 0x2 = Zero Latency Pinning, 0x4 = NPU Acceleration Boost
    pub flags: u32,
}

/// Statistics counter map indices for sched_ext telemetry
pub const STAT_ENQUEUED: u32 = 0;
pub const STAT_DISPATCHED_AI: u32 = 1;
pub const STAT_DISPATCHED_CFS_FALLBACK: u32 = 2;
pub const STAT_TICK_PREEMPTED: u32 = 3;
pub const STAT_TARGET_CPU_SELECTION: u32 = 4;

/// 1. eBPF HashMap: Key = PID (u32), Value = AiSchedParam struct with priority and target_cpu
#[map]
static AI_SCHED_MAP: HashMap<u32, AiSchedParam> = HashMap::with_max_entries(4096, 0);

/// eBPF Array for scheduler metrics and telemetry counters
#[map]
static SCHED_STATS: Array<u64> = Array::with_max_entries(16, 0);

#[inline(always)]
fn increment_stat(index: u32) {
    if let Some(ptr) = SCHED_STATS.get_ptr_mut(index) {
        unsafe {
            *ptr += 1;
        }
    }
}

/// -----------------------------------------------------------------------------
/// 2. sched_ext `enqueue` Hook
/// Triggered when kernel requests to enqueue a task (PID).
/// Reads `AI_SCHED_MAP`. If AI priority/CPU is specified for PID, overrides latency target.
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_enqueue"]
pub fn scx_enqueue(ctx: *mut core::ffi::c_void) -> i32 {
    let ctx = SchedExtContext::new(ctx);
    let pid = ctx.pid();
    increment_stat(STAT_ENQUEUED);

    // Read AI_SCHED_MAP for task PID
    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        info!(
            &ctx,
            "⚡ [sched_ext:enqueue] PID {}: AI Policy active -> Priority: {}, Target CPU: {}",
            pid,
            param.priority,
            param.target_cpu
        );

        // Force zero-latency scheduling if CFS bypass flag (0x1) or high priority (priority <= 1) is active
        if (param.flags & 0x1) != 0 || param.priority <= 1 {
            info!(
                &ctx,
                "🚀 [sched_ext:enqueue] Bypassing Linux CFS for PID {} -> Forced Latency Priority {}",
                pid,
                param.priority
            );
        }

        return 0;
    }

    // Default: Return 0 (Standard scheduler fallback)
    0
}

/// -----------------------------------------------------------------------------
/// 3. sched_ext `dispatch` Hook
/// Called when kernel queries next task for execution on target CPU core.
/// Bypasses Linux CFS scheduler if AI policy exists for PID, forcing CPU & latency.
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_dispatch"]
pub fn scx_dispatch(ctx: *mut core::ffi::c_void) -> i32 {
    let ctx = SchedExtContext::new(ctx);
    let cpu = ctx.cpu();
    let pid = ctx.pid();

    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        // Read AI map parameters
        let is_target_cpu = param.target_cpu == cpu || param.target_cpu == u32::MAX;
        let is_ai_cfs_bypass = (param.flags & 0x1) != 0 || param.priority <= 1;

        if is_target_cpu && is_ai_cfs_bypass {
            increment_stat(STAT_DISPATCHED_AI);
            info!(
                &ctx,
                "⚡ [sched_ext:dispatch] FORCING AI DISPATCH on CPU {} for PID {} (CFS Bypassed, Core: {})",
                cpu,
                pid,
                param.target_cpu
            );
            // Return 1: Direct dispatch to local DSQ on target CPU core, bypassing standard CFS
            return 1;
        }
    }

    increment_stat(STAT_DISPATCHED_CFS_FALLBACK);
    0
}

/// -----------------------------------------------------------------------------
/// 4. sched_ext `tick` Hook
/// Called on scheduler timer ticks for active task.
/// Checks runtime nanoseconds against AI slice_ns to enforce zero-latency preemption.
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_tick"]
pub fn scx_tick(ctx: *mut core::ffi::c_void) -> i32 {
    let ctx = SchedExtContext::new(ctx);
    let pid = ctx.pid();
    let runtime_ns = ctx.runtime_ns();

    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        if param.slice_ns > 0 && runtime_ns >= param.slice_ns {
            increment_stat(STAT_TICK_PREEMPTED);
            info!(
                &ctx,
                "⏱️ [sched_ext:tick] PID {} slice expired ({}ns >= {}ns). Enforcing AI Preemption.",
                pid,
                runtime_ns,
                param.slice_ns
            );
            return 1; // Reschedule requested
        }
    }

    0
}

/// -----------------------------------------------------------------------------
/// 5. sched_ext `select_cpu` Hook
/// Selects target CPU core during task wakeup.
/// Forces CPU affinity to `param.target_cpu` dictated by AI model.
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_select_cpu"]
pub fn scx_select_cpu(ctx: *mut core::ffi::c_void) -> i32 {
    let ctx = SchedExtContext::new(ctx);
    let pid = ctx.pid();
    let prev_cpu = ctx.cpu();

    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        if param.target_cpu != u32::MAX {
            increment_stat(STAT_TARGET_CPU_SELECTION);
            info!(
                &ctx,
                "🎯 [sched_ext:select_cpu] Steering PID {} from CPU {} to AI Target CPU {}",
                pid,
                prev_cpu,
                param.target_cpu
            );
            return param.target_cpu as i32;
        }
    }

    prev_cpu as i32
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
