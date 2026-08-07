#![allow(unexpected_cfgs)]
//! Security & Cryptographic Verification Engine for Ermete Gatekeeper.
//!
//! Provides mathematically verified state machine logic, token authentication,
//! and fanotify event buffer parsing with guaranteed absence of panic,
//! out-of-bounds array access, or arithmetic overflows.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecurityError {
    InvalidTokenLength,
    InvalidMagic,
    TimestampExpired,
    DigestMismatch,
    BufferTooSmall,
    InvalidEventLength,
    Overflow,
}

/// Constant-time memory comparison to prevent timing side-channel leaks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    let mut i = 0;
    while i < a.len() {
        result |= a[i] ^ b[i];
        i += 1;
    }
    result == 0
}

/// Validates authorization token for high-privilege operations in Gatekeeper.
/// Format: [MAGIC (4 bytes) | TIMESTAMP (8 bytes LE) | DIGEST (32 bytes)]
pub fn verify_auth_token(
    token: &[u8],
    current_time: u64,
    max_skew: u64,
    expected_digest: &[u8; 32],
) -> Result<bool, SecurityError> {
    const MAGIC: [u8; 4] = [0x45, 0x52, 0x4D, 0x54]; // "ERMT"
    const MIN_LEN: usize = 4 + 8 + 32; // 44 bytes

    if token.len() < MIN_LEN {
        return Err(SecurityError::InvalidTokenLength);
    }

    // Check magic bytes
    if token[0..4] != MAGIC {
        return Err(SecurityError::InvalidMagic);
    }

    // Parse timestamp (bytes 4..12) safely
    let mut ts_bytes = [0u8; 8];
    ts_bytes.copy_from_slice(&token[4..12]);
    let token_ts = u64::from_le_bytes(ts_bytes);

    // Verify timestamp skew without arithmetic overflow
    let diff = current_time.abs_diff(token_ts);

    if diff > max_skew {
        return Err(SecurityError::TimestampExpired);
    }

    // Verify digest using constant-time comparison
    let token_digest = &token[12..44];
    if !constant_time_eq(token_digest, expected_digest) {
        return Err(SecurityError::DigestMismatch);
    }

    Ok(true)
}

/// Safely parses fanotify event metadata buffer offsets without panics or infinite loops.
pub fn parse_next_fanotify_offset(
    buffer_len: usize,
    current_offset: usize,
    event_len: u32,
    min_header_size: usize,
) -> Result<usize, SecurityError> {
    if min_header_size == 0 {
        return Err(SecurityError::InvalidEventLength);
    }

    // Check if current_offset is within buffer bounds
    if current_offset > buffer_len {
        return Err(SecurityError::BufferTooSmall);
    }

    // Ensure space remaining for header
    let remaining = buffer_len - current_offset;
    if remaining < min_header_size {
        return Err(SecurityError::BufferTooSmall);
    }

    let event_len_usize = event_len as usize;
    if event_len_usize < min_header_size {
        return Err(SecurityError::InvalidEventLength);
    }

    // Check arithmetic overflow on offset advance
    match current_offset.checked_add(event_len_usize) {
        Some(next_offset) => {
            if next_offset > buffer_len {
                // Event extends beyond buffer boundary
                Err(SecurityError::BufferTooSmall)
            } else {
                Ok(next_offset)
            }
        }
        None => Err(SecurityError::Overflow),
    }
}

/// Computes execution rate limits and updates state counters without arithmetic overflow.
pub fn evaluate_execution_rate_limit(
    window_start: u64,
    current_time: u64,
    event_count: u32,
    max_allowed: u32,
    window_duration: u64,
) -> (bool, u64, u32) {
    if current_time < window_start {
        // Clock skew / reset window
        return (true, current_time, 1);
    }

    let elapsed = current_time - window_start;
    if elapsed >= window_duration {
        // New rate-limiting window
        (true, current_time, 1)
    } else {
        // Within current window
        let next_count = event_count.saturating_add(1);
        let allowed = next_count <= max_allowed;
        (allowed, window_start, next_count)
    }
}

#[cfg(kani)]
mod proof {
    use super::*;

    #[kani::proof]
    #[kani::unwind(17)]
    pub fn proof_constant_time_eq_no_panic() {
        let len_a: usize = kani::any();
        let len_b: usize = kani::any();
        kani::assume(len_a <= 16);
        kani::assume(len_b <= 16);

        let data_a: [u8; 16] = kani::any();
        let data_b: [u8; 16] = kani::any();

        let slice_a = &data_a[..len_a];
        let slice_b = &data_b[..len_b];

        let res = constant_time_eq(slice_a, slice_b);
        if len_a != len_b {
            kani::assert(!res, "Mismatched lengths must evaluate to false");
        }
    }

    #[kani::proof]
    #[kani::unwind(33)]
    pub fn proof_verify_auth_token_no_panic() {
        let token_len: usize = kani::any();
        kani::assume(token_len <= 48);

        let token_buf: [u8; 48] = kani::any();
        let token_slice = &token_buf[..token_len];

        let current_time: u64 = kani::any();
        let max_skew: u64 = kani::any();
        let expected_digest: [u8; 32] = kani::any();

        let _res = verify_auth_token(token_slice, current_time, max_skew, &expected_digest);
    }

    #[kani::proof]
    pub fn proof_parse_next_fanotify_offset_bounds_safety() {
        let buffer_len: usize = kani::any();
        let current_offset: usize = kani::any();
        let event_len: u32 = kani::any();
        let min_header_size: usize = kani::any();

        kani::assume(buffer_len <= 1024);
        kani::assume(current_offset <= 1024);
        kani::assume(min_header_size <= 128);

        match parse_next_fanotify_offset(buffer_len, current_offset, event_len, min_header_size) {
            Ok(next_offset) => {
                kani::assert(next_offset > current_offset, "Next offset must strictly advance");
                kani::assert(next_offset <= buffer_len, "Next offset must not exceed buffer length");
            }
            Err(e) => {
                kani::assert(
                    e == SecurityError::InvalidEventLength
                        || e == SecurityError::BufferTooSmall
                        || e == SecurityError::Overflow,
                    "Errors must be well-defined",
                );
            }
        }
    }

    #[kani::proof]
    pub fn proof_evaluate_execution_rate_limit_no_overflow() {
        let window_start: u64 = kani::any();
        let current_time: u64 = kani::any();
        let event_count: u32 = kani::any();
        let max_allowed: u32 = kani::any();
        let window_duration: u64 = kani::any();

        let (_allowed, _new_start, new_count) = evaluate_execution_rate_limit(
            window_start,
            current_time,
            event_count,
            max_allowed,
            window_duration,
        );

        kani::assert(new_count > 0, "Counter must be positive");
    }
}
