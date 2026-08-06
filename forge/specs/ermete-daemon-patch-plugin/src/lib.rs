#![allow(unsafe_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};

use std::os::raw::c_char;

/// Dynamic hot-patch handler exported by the .so plugin.
/// When loaded into `ermete-daemon-rs` RAM via `libloading` / `load_patch_so`,
/// the ACTIVE_PATCH_FN function pointer is atomically updated to point to this function.
#[no_mangle]
pub unsafe extern "C" fn ermete_zbus_patch_handler(
    method: *const c_char,
    input: *const c_char,
) -> *mut c_char {
    if method.is_null() {
        return std::ptr::null_mut();
    }

    let method_str = CStr::from_ptr(method).to_string_lossy();
    let _input_str = if !input.is_null() {
        CStr::from_ptr(input).to_string_lossy().to_string()
    } else {
        String::new()
    };

    match method_str.as_ref() {
        "ping" => {
            let resp = CString::new("pong-v2-live-patched-zero-downtime").unwrap();
            resp.into_raw()
        }
        "custom_method" => {
            let resp = CString::new("{\"status\": \"ok\", \"patched_by\": \"ebpf_uprobe_live_patch\"}").unwrap();
            resp.into_raw()
        }
        _ => std::ptr::null_mut(),
    }
}
