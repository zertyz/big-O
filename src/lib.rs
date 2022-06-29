#![doc = include_str!("../README.md")]

pub mod runners;
pub mod low_level_analysis;
pub mod metrics_allocator;
pub mod configs;


// exported symbols
pub use {
    configs::{ALLOC, OUTPUT},
    low_level_analysis::types::{
        BigOAlgorithmComplexity,
        TimeUnits,
    },
    runners::{
        standard::test_constant_set_algorithm,
        crud::test_crud_algorithms,
    },
};
