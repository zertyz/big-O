//! Contains code for handling the Cargo features used to compile this crate.
#![allow(dead_code)]

use std::io::{stdout,stderr,Write};
use crate::metrics_allocator::MetricsAllocator;

#[cfg(any(feature = "tolerance_10_percent", not(any(feature = "tolerance_25_percent"))))]
/// acceptable proportional variance (acceptable measurement errors) when analysing algorithm's time & space complexities
pub const PERCENT_TOLERANCE: f64 = 0.10;

#[cfg(feature = "tolerance_25_percent")]
/// acceptable proportional variance (acceptable measurement errors) when analysing algorithm's time & space complexities
pub const PERCENT_TOLERANCE: f64 = 0.25;

#[cfg(feature = "report_stdout")]
/// Function to output an `&str` -- used to sink analysis reports -- controlled by the crate's features (stdout, stderr, no_output)
pub const OUTPUT: fn(&str) = stdout_write;

#[cfg(feature = "report_stderr")]
/// Function to output an `&str` -- used to sink analysis reports -- controlled by the crate's features (stdout, stderr, no_output)
pub const OUTPUT: fn(&str) = stderr_write;

#[cfg(not(any(feature = "report_stdout", feature = "report_stderr")))]
/// Function to output an `&str` -- used to sink analysis reports -- controlled by the crate's features (stdout, stderr, no_output)
pub const OUTPUT: fn(&str) = null_write;

#[cfg(not(feature = "no_allocator_metrics"))]
#[global_allocator]
/// Allows access to the metrics allocator -- replacing the Global Allocator for tests
/// (provided this crate is used as `dev-dependency`).
/// NOTE: as mentioned in the README, if you want ot use this crate in integration tests, you should
///       have a feature in your project to only include this crate if you are compiling for integration tests
pub static ALLOC: MetricsAllocator<SAVE_POINT_RING_BUFFER_SIZE> = MetricsAllocator::new();

/// Regarding the [MetricsAllocator] used for space complexity analysis, this property specifies the maximum number of "save points"
/// that might be in use at the same time
pub const SAVE_POINT_RING_BUFFER_SIZE: usize = 1024;


fn stdout_write(buf: &str) {
    sync_outputs();
    print!("{}", buf);
    sync_outputs();
}

fn stderr_write(buf: &str) {
    sync_outputs();
    eprint!("{}", buf);
    sync_outputs();
}

/// Flushes both stdout and stderr so the next output will be in sync with everything that came before
fn sync_outputs() {
    _ = stdout().flush();
    _ = stderr().flush();
}

fn null_write(_buf: &str) {
    // release compilations will optimize out this call for '_buf' is not used
}

