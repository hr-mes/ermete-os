#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // dummy target to keep CI green until real targets are written
    let _ = data.len();
});
