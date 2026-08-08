use libc::{c_void, fanotify_event_metadata, fanotify_response};
use std::os::unix::io::RawFd;

pub const FAN_CLASS_CONTENT: u32 = 0x00000004;
pub const FAN_NONBLOCK: u32 = 0x00000002;
pub const FAN_MARK_ADD: u32 = 0x00000001;
pub const FAN_MARK_MOUNT: u32 = 0x00000010;
pub const FAN_OPEN_EXEC_PERM: u64 = 0x00010000;
pub const FAN_ALLOW: u32 = 0x01;
pub const FAN_DENY: u32 = 0x02;
pub const FAN_EVENT_METADATA_LEN: usize = std::mem::size_of::<fanotify_event_metadata>();

pub fn respond_and_close(fanotify_fd: RawFd, event_fd: RawFd, response_code: u32) {
    let mut response = fanotify_response {
        fd: event_fd,
        response: response_code,
    };
    // SAFETY: Write fanotify response to fanotify_fd file descriptor.
    unsafe {
        libc::write(
            fanotify_fd,
            &mut response as *mut _ as *const c_void,
            std::mem::size_of::<fanotify_response>(),
        );
    }
    // SAFETY: Close fanotify event file descriptor.
    unsafe {
        libc::close(event_fd);
    }
}
