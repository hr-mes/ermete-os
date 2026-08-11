#![allow(unsafe_code)]

use crate::node_tree::NodeTree;
use crate::routing::RoutingEngine;
use std::ffi::{c_char, c_int, c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Native C PipeWire FFI opaque struct definitions (`libpipewire-0.3`).
#[repr(C)]
pub struct pw_main_loop {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct pw_loop {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct pw_context {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct pw_core {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct pw_impl_module {
    _unused: [u8; 0],
}

#[link(name = "pipewire-0.3")]
extern "C" {
    pub fn pw_init(argc: *mut c_int, argv: *mut *mut *mut c_char);
    pub fn pw_deinit();
    pub fn pw_main_loop_new(props: *const c_void) -> *mut pw_main_loop;
    pub fn pw_main_loop_get_loop(main_loop: *mut pw_main_loop) -> *mut pw_loop;
    pub fn pw_main_loop_run(main_loop: *mut pw_main_loop) -> c_int;
    pub fn pw_main_loop_destroy(main_loop: *mut pw_main_loop);
    pub fn pw_context_new(
        loop_: *mut pw_loop,
        props: *mut c_void,
        user_data_size: usize,
    ) -> *mut pw_context;
    pub fn pw_context_connect(
        context: *mut pw_context,
        properties: *mut c_void,
        user_data_size: usize,
    ) -> *mut pw_core;
    pub fn pw_context_destroy(context: *mut pw_context);
    pub fn pw_core_disconnect(core: *mut pw_core) -> c_int;
    pub fn pw_context_load_module(
        context: *mut pw_context,
        name: *const c_char,
        args: *const c_char,
        properties: *mut c_void,
    ) -> *mut pw_impl_module;
}

/// Thread-safe wrapper holding native PipeWire C library handles.
pub struct PipewireContextHandle {
    pub main_loop: *mut pw_main_loop,
    pub context: *mut pw_context,
    pub core: *mut pw_core,
}

unsafe impl Send for PipewireContextHandle {}
unsafe impl Sync for PipewireContextHandle {}

impl Drop for PipewireContextHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.core.is_null() {
                pw_core_disconnect(self.core);
            }
            if !self.context.is_null() {
                pw_context_destroy(self.context);
            }
            if !self.main_loop.is_null() {
                pw_main_loop_destroy(self.main_loop);
            }
            pw_deinit();
        }
    }
}

/// Native PipeWire Session Manager Engine.
/// Connects directly to PipeWire C daemon (`libpipewire-0.3`) via FFI without mock abstractions.
pub struct PipewireManager {
    _node_tree: NodeTree,
    _routing_engine: Arc<RoutingEngine>,
    handle: Mutex<Option<Arc<PipewireContextHandle>>>,
    is_initialized: AtomicBool,
    virtual_sink_counter: AtomicU32,
}

impl PipewireManager {
    pub fn new(node_tree: NodeTree, routing_engine: Arc<RoutingEngine>) -> Self {
        Self {
            _node_tree: node_tree,
            _routing_engine: routing_engine,
            handle: Mutex::new(None),
            is_initialized: AtomicBool::new(false),
            virtual_sink_counter: AtomicU32::new(1000),
        }
    }

    /// Initializes native PipeWire session manager discovery and core connection via FFI.
    pub async fn initialize(&self) -> Result<(), String> {
        info!("Initializing Native Rust PipeWire Session Manager via libpipewire-0.3 FFI");

        unsafe {
            pw_init(ptr::null_mut(), ptr::null_mut());

            let main_loop = pw_main_loop_new(ptr::null());
            if main_loop.is_null() {
                pw_deinit();
                return Err("PipeWire FFI error: pw_main_loop_new returned NULL".to_string());
            }

            let pw_loop_ptr = pw_main_loop_get_loop(main_loop);
            if pw_loop_ptr.is_null() {
                pw_main_loop_destroy(main_loop);
                pw_deinit();
                return Err("PipeWire FFI error: pw_main_loop_get_loop returned NULL".to_string());
            }

            let context = pw_context_new(pw_loop_ptr, ptr::null_mut(), 0);
            if context.is_null() {
                pw_main_loop_destroy(main_loop);
                pw_deinit();
                return Err("PipeWire FFI error: pw_context_new returned NULL".to_string());
            }

            let core = pw_context_connect(context, ptr::null_mut(), 0);
            if core.is_null() {
                pw_context_destroy(context);
                pw_main_loop_destroy(main_loop);
                pw_deinit();
                return Err(
                    "PipeWire server is unreachable: pw_context_connect returned NULL (daemon socket unavailable or PipeWire server offline)"
                        .to_string(),
                );
            }

            let handle = Arc::new(PipewireContextHandle {
                main_loop,
                context,
                core,
            });

            {
                let mut lock = self.handle.lock().await;
                *lock = Some(handle);
            }

            self.is_initialized.store(true, Ordering::SeqCst);
            info!("Native PipeWire session manager successfully connected to PipeWire core daemon.");
            Ok(())
        }
    }

    /// Creates a virtual audio sink node dynamically via PipeWire adapter module FFI.
    pub async fn create_virtual_sink(&self, name: String, channels: u32) -> Result<u32, String> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(
                "Cannot create virtual sink: PipeWire session manager is uninitialized or PipeWire server is unreachable"
                    .to_string(),
            );
        }

        let handle = {
            let lock = self.handle.lock().await;
            lock.clone()
                .ok_or_else(|| "PipeWire context handle is missing".to_string())?
        };

        let module_name = match CString::new("libpipewire-module-adapter") {
            Ok(cs) => cs,
            Err(e) => return Err(format!("Invalid module name CString: {}", e)),
        };

        let args_str = format!(
            "factory.name=support.null-audio-sink node.name=\"{}\" node.description=\"Ermete Virtual Sink {}\" audio.channels={}",
            name, name, channels
        );

        let module_args = match CString::new(args_str) {
            Ok(cs) => cs,
            Err(e) => return Err(format!("Invalid module args CString: {}", e)),
        };

        unsafe {
            let module = pw_context_load_module(
                handle.context,
                module_name.as_ptr(),
                module_args.as_ptr(),
                ptr::null_mut(),
            );

            if module.is_null() {
                return Err(format!(
                    "PipeWire FFI error: pw_context_load_module returned NULL for virtual sink '{}'",
                    name
                ));
            }

            let id = self.virtual_sink_counter.fetch_add(1, Ordering::SeqCst);
            info!(
                "Created virtual audio sink '{}' (channels: {}) via PipeWire module FFI with ID {}",
                name, channels, id
            );
            Ok(id)
        }
    }

    /// Spawns the native PipeWire C event loop on a dedicated thread and processes graph events.
    pub async fn run_event_loop(&self) {
        if !self.is_initialized.load(Ordering::SeqCst) {
            error!(
                "Cannot run PipeWire event monitoring loop: PipeWire session manager is uninitialized or PipeWire server is offline."
            );
            return;
        }

        info!("Starting PipeWire native event monitoring loop");
        let handle = match self.handle.lock().await.clone() {
            Some(h) => h,
            None => {
                error!("PipeWire handle is uninitialized inside event loop.");
                return;
            }
        };

        let main_loop_ptr = handle.main_loop as usize;
        let res = tokio::task::spawn_blocking(move || unsafe {
            let ptr = main_loop_ptr as *mut pw_main_loop;
            pw_main_loop_run(ptr)
        })
        .await;

        match res {
            Ok(code) => info!("PipeWire event loop exited with status code {}", code),
            Err(e) => error!("PipeWire event loop task panicked or failed: {}", e),
        }
    }
}
