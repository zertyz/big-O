//! THIS IS IN lib.rs

pub mod crud_analysis;
pub mod low_level_analysis;
pub mod metrics_allocator;
pub mod configs;


// exported symbols
pub use {
    low_level_analysis::types::{
        BigOAlgorithmComplexity,
        TimeUnits,
    },
    configs::{ALLOC,OUTPUT},
    crud_analysis::test_crud_algorithms,
};
