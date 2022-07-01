//! Contains constants used throughout this crate

#![allow(dead_code)]

use std::io::{stdout,stderr,Write};
use crate::metrics_allocator::MetricsAllocator;

/// acceptable proportional variance (acceptable measurement errors) when analysing algorithm's time & space complexities
pub const PERCENT_TOLERANCE: f64 = 0.10;

/// Function to output an `&str` -- used to sink analysis reports -- controlled by the crate's features (stdout, stderr, no_output)
pub const OUTPUT: fn(&str) = stdout_write;

//#[cfg(test)]
// cfg(test) above seems not to work for library crates. That should go into "features", then?
#[global_allocator]
/// Allows access to the metrics allocator -- replacing the Global Allocator for tests
/// (provided this crate is used as `dev-dependency`)
pub static ALLOC: MetricsAllocator<SAVE_POINT_RING_BUFFER_SIZE> = MetricsAllocator::new();

/// Regarding the [MetricsAllocator] used for space complexity analysis, this property specifies the maximum number of "save points"
/// that might be in use at the same time
pub const SAVE_POINT_RING_BUFFER_SIZE: usize = 1024;


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

