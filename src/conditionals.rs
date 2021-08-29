//! Contains conditional compilation definitions attending to:
//!   - "features" definitions, client project's Cargo "[dependencies]" declarations
//!   - Release / Debug compilations

#![warn(dead_code)]

use std::io::{stdout,stderr,Write};
use crate::metrics_allocator::MetricsAllocator;

#[cfg(debug_assertions)]
/// loop multiplier for debug compilation
pub const LOOP_MULTIPLIER: u32 = 1;
#[cfg(not(debug_assertions))]
/// loop multiplier for release compilation
pub const LOOP_MULTIPLIER: u32 = 100;

// if features = stdout
pub const OUTPUT: fn(&str) = stdout_write;

fn stdout_write(buf: &str) {
    stdout().flush().unwrap();
    stderr().flush().unwrap();
    print!("{}", buf);
    stdout().flush().unwrap();
    stderr().flush().unwrap();
}

fn stderr_write(buf: &str) {
    stdout().flush().unwrap();
    stderr().flush().unwrap();
    eprint!("{}", buf);
    stdout().flush().unwrap();
    stderr().flush().unwrap();
}

fn null_write(_buf: &str) {
    // release compilations will optimize out this call for '_buf' is not used
}

/// maximum number of "save points" that might be in use at the same time
/// (for which a call to [MetricsAllocatorStatistics.delta_statistics] will still be made)
const SAVE_POINT_RING_BUFFER_SIZE: usize = 16;

#[cfg(test)]
#[global_allocator]
/// Custom allocator when running tests
pub static ALLOC: MetricsAllocator/*<SAVE_POINT_RING_BUFFER_SIZE>*/ = MetricsAllocator::new();