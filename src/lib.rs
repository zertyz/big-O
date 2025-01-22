#![doc = include_str!("../README.md")]


pub mod api;
pub mod runners;
pub mod low_level_analysis;
pub mod metrics_allocator;
mod features;
pub(crate) mod utils;

// exported symbols
pub use {
    features::{ALLOC, OUTPUT},
    low_level_analysis::types::{
        BigOAlgorithmComplexity
    },
    runners::{
        standard::{test_algorithm,test_constant_set_iterator_algorithm,test_set_resizing_iterator_algorithm},
        crud::test_crud_algorithms,
    },
};
