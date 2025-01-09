#![doc = include_str!("../README.md")]


pub mod runners;
pub mod low_level_analysis;
pub mod metrics_allocator;
mod features;

// exported symbols
pub use {
    features::{ALLOC, OUTPUT},
    low_level_analysis::types::{
        BigOAlgorithmComplexity,
        TimeUnits,
    },
    runners::{
        standard::{test_algorithm,test_constant_set_iterator_algorithm,test_set_resizing_iterator_algorithm},
        crud::test_crud_algorithms,
    },
};
