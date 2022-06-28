//! Global allocator (wrapper around the System's default allocator) capable of gathering allocation/de-allocation/re-allocation metrics
//! and (min, max) memory usage between two (or more) points in time.
//!
//! Activate it with:
//! ```no_compile
//!     use crate::big_O::metrics_allocator::MetricsAllocator;
//!     #[global_allocator]
//!     static ALLOC: MetricsAllocator = MetricsAllocator::new();
//! ```
//!
//! Usage example:
//! ```rust
//!    use crate::big_o::configs::ALLOC;
//!     let save_point = ALLOC.save_point();
//!     let _vec = Vec::<u32>::with_capacity(1024);
//!     let metrics = ALLOC.delta_statistics(&save_point);
//!     println!("Allocator Metrics for the Vec allocation: {}", metrics);

mod metrics_allocator;
pub use metrics_allocator::*;
pub mod ring_buffer;
