#![no_std]
#![no_main]

use aya_ebpf::{
    macros::map,
    maps::{Array, HashMap},
    EbpfContext,
};

/// -----------------------------------------------------------------------------
/// FFI declarations for kernel sched_ext kfuncs and bpf helpers
/// -----------------------------------------------------------------------------
extern "C" {
    fn scx_bpf_dispatch(p: *mut core::ffi::c_void, dsq_id: u64, slice: u64, enq_flags: u64);
    fn scx_bpf_dsq_insert(p: *mut core::ffi::c_void, dsq_id: u64, slice: u64, enq_flags: u64);
    fn bpf_task_pt_regs(task: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

pub const SCX_DSQ_GLOBAL: u64 = 0;
pub const SCX_DSQ_LOCAL: u64 = 1;

pub struct StructOpsContext {
    ctx: *mut core::ffi::c_void,
}

impl EbpfContext for StructOpsContext {
    fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.ctx
    }
}

impl StructOpsContext {
    pub fn new(ctx: *mut core::ffi::c_void) -> Self {
        Self { ctx }
    }

    pub fn arg(&self, index: isize) -> u64 {
        unsafe {
            let ctx_ptr = self.ctx as *const u64;
            *ctx_ptr.offset(index)
        }
    }
}

/// AI Scheduling parameters per PID set by user-space AI engine
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiSchedParam {
    pub pid: u32,
    pub target_core: u32,       
    pub core_type: u8,          
    pub _pad: [u8; 3],          
    pub cpu_weight: u32,
    pub slice_us: u64,
    pub sched_class: u32,       
    pub latency_target_us: u64,
    pub flags: u32,
}

/// Statistics counter map indices for sched_ext telemetry
pub const STAT_ENQUEUED: u32 = 0;
pub const STAT_DISPATCHED_AI: u32 = 1;
pub const STAT_DISPATCHED_CFS_FALLBACK: u32 = 2;
pub const STAT_TICK_PREEMPTED: u32 = 3;
pub const STAT_TARGET_CPU_SELECTION: u32 = 4;

#[map]
static AI_SCHED_MAP: HashMap<u32, AiSchedParam> = HashMap::with_max_entries(4096, 0);

#[map]
static SCHED_STATS: Array<u64> = Array::with_max_entries(16, 0);

#[inline(always)]
fn increment_stat(index: u32) {
    if let Some(ptr) = SCHED_STATS.get_ptr_mut(index) {
        unsafe { *ptr += 1; }
    }
}

/// Helper to read PID from task_struct safely
#[inline(always)]
unsafe fn get_task_pid(task: *mut core::ffi::c_void) -> u32 {
    let mut pid: u32 = 0;
    // Offset 0x548 is typical for pid in x86_64 6.x kernels, using safe probe read.
    // In a real BTF-enabled compile, one uses bpf_core_read.
    let pid_ptr = (task as *const u8).add(0x548) as *const u32;
    let _ = aya_ebpf::helpers::bpf_probe_read_kernel(
        &mut pid as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        pid_ptr as *const core::ffi::c_void,
    );
    pid
}

/// -----------------------------------------------------------------------------
/// sched_ext `enqueue` Hook
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_enqueue"]
pub fn scx_enqueue(ctx: *mut core::ffi::c_void) -> i32 {
    let s_ctx = StructOpsContext::new(ctx);
    let task = s_ctx.arg(0) as *mut core::ffi::c_void;
    let enq_flags = s_ctx.arg(1);
    
    let pid = unsafe { get_task_pid(task) };
    increment_stat(STAT_ENQUEUED);

    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        // Validation check for deviation or memory poisoning
        // 0x8000_0000 or invalid extreme slices indicate poison/deviation
        if (param.flags & 0x8000_0000) != 0 || param.slice_us > 1_000_000 {
            unsafe {
                // IMMEDIATE KILL (SIGKILL = 9) of unvalidated/poisoned node
                aya_ebpf::helpers::bpf_send_signal(9);
            }
            increment_stat(STAT_DISPATCHED_CFS_FALLBACK);
            // Brutal fallback to pure CFS
            return 0;
        }

        // If AI specifies a slice, we can dispatch directly to local or global DSQ
        if (param.flags & 0x1) != 0 || param.sched_class <= 1 {
            unsafe {
                // Dispatch directly to local queue to bypass standard scheduler
                // Convert slice_us to nanoseconds
                scx_bpf_dispatch(task, SCX_DSQ_LOCAL, param.slice_us * 1000, enq_flags);
            }
            increment_stat(STAT_DISPATCHED_AI);
            return 0; 
        }
    }

    increment_stat(STAT_DISPATCHED_CFS_FALLBACK);
    // Return 0 means standard queuing
    0
}

/// -----------------------------------------------------------------------------
/// sched_ext `dispatch` Hook
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_dispatch"]
pub fn scx_dispatch(ctx: *mut core::ffi::c_void) -> i32 {
    // Usually invoked when a CPU is idle and needs a task.
    // Standard return 0 allows the kernel to pull from the global queue.
    0
}

/// -----------------------------------------------------------------------------
/// sched_ext `tick` Hook
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_tick"]
pub fn scx_tick(ctx: *mut core::ffi::c_void) -> i32 {
    // We can preempt here if the task exceeded its slice.
    0
}

/// -----------------------------------------------------------------------------
/// sched_ext `select_cpu` Hook
/// -----------------------------------------------------------------------------
#[no_mangle]
#[link_section = "struct_ops/scx_select_cpu"]
pub fn scx_select_cpu(ctx: *mut core::ffi::c_void) -> i32 {
    let s_ctx = StructOpsContext::new(ctx);
    let task = s_ctx.arg(0) as *mut core::ffi::c_void;
    let prev_cpu = s_ctx.arg(1) as i32;
    // let wake_flags = s_ctx.arg(2);
    
    let pid = unsafe { get_task_pid(task) };

    if let Some(param) = unsafe { AI_SCHED_MAP.get(&pid) } {
        if param.target_core != u32::MAX {
            increment_stat(STAT_TARGET_CPU_SELECTION);
            return param.target_core as i32;
        }
    }

    prev_cpu
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
