//! Applies the Big-O crate to some Rust's std lib containers

use big_o_test::*;
use std::{
    sync::Arc,
    collections::HashMap,
};
use ctor::ctor;


#[cfg(debug_assertions)]
/// loop multiplier for debug compilation
pub const LOOP_MULTIPLIER: u32 = 1;
#[cfg(not(debug_assertions))]
/// loop multiplier for release compilation
pub const LOOP_MULTIPLIER: u32 = 64;


/// Sets up the ENV, affecting the Rust's test runner
#[ctor]
fn setup_env() {
    // cause tests to run serially -- this may be replaced by using the `serial_test` crate
    std::env::set_var("RUST_TEST_THREADS", "1");
}


/// Attests the best case CRUD for vectors -- Create, Read, Update and Delete... all O(1):
///   - inserts at the end (push)
///   - deletes at the end (pop)
#[cfg_attr(not(feature = "dox"), test)]
fn vec_best_case_algorithm_analysis() {
    let iterations_per_pass: u32 = 250_000 * LOOP_MULTIPLIER;
    let n_threads = 1;
    let vec_locker = parking_lot::RwLock::new(Vec::<u32>::with_capacity(0));
    test_crud_algorithms("Vec Push & Pop (best case) with ParkingLot", 15,
            |_n| {
                let mut vec = vec_locker.write();
                vec.clear();
                vec.shrink_to_fit();
                vec.len() as u32
            },
            |n| {
                let mut vec = vec_locker.write();
                vec.push(n);
                vec.len() as u32
            }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
            |n| {
                let vec = vec_locker.read();
                vec[n as usize]
            }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
            |n| {
                let mut vec = vec_locker.write();
                vec[n as usize] = n+1;
                vec.len() as u32
            }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
            |_n| {
                let mut vec = vec_locker.write();
                vec.pop().unwrap()
            }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
            25, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
            n_threads, n_threads, n_threads, n_threads,
            &TimeUnits::MICROSECOND);
}

/// Attests the worst case CRUD for vectors:
///   - Create always at the beginning -- O(n)
///   - Delete always at the beginning -- O(n)
///   - Reads and updates as the usual O(1)
#[cfg_attr(not(feature = "dox"), test)]
fn vec_worst_case_algorithm_analysis() {
    let iterations_per_pass: u32 = 16_384 * std::cmp::min(2, LOOP_MULTIPLIER);
    let n_threads = 1;
    let vec_locker = parking_lot::RwLock::new(Vec::<u32>::with_capacity(0));
    test_crud_algorithms("Vec Insert & Remove (worst case) with ParkingLot", 15,
           |_n| {
               let mut vec = vec_locker.write();
               vec.clear();
               vec.shrink_to_fit();     // needed for retries (even if warmup is disabled)
               vec.len() as u32
           },
           |n| {
               let val = (iterations_per_pass as u32)*2 - n;
               let mut vec = vec_locker.write();
               vec.insert(0, val);
               val
           }, BigOAlgorithmComplexity::ON, BigOAlgorithmComplexity::O1,
           |n| {
               let vec = vec_locker.read();
               let len = vec.len();
               vec[n as usize % len]
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           |n| {
               let mut vec = vec_locker.write();
               let len = vec.len();
               vec[n as usize % len] = n+1;
               n+1
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           |_n| {
               let mut vec = vec_locker.write();
               vec.remove(0)
           }, BigOAlgorithmComplexity::ON, BigOAlgorithmComplexity::O1,
           0, iterations_per_pass, iterations_per_pass*10, iterations_per_pass*10, iterations_per_pass,
           n_threads, n_threads, n_threads, n_threads,
           &TimeUnits::MICROSECOND);
}

/// Attests O(1) performance characteristics for HashMaps
#[cfg_attr(not(feature = "dox"), test)]
fn hashmap_algorithm_analysis() {
    let iterations_per_pass = 30_000 * LOOP_MULTIPLIER;
    let n_threads = 1;
    let allocator_save_point = ALLOC.save_point();
    let map_locker = Arc::new(parking_lot::RwLock::new(HashMap::<String, u32>::with_capacity(2 * iterations_per_pass as usize)));
    let hashmap_allocation_statistics = ALLOC.delta_statistics(&allocator_save_point);
    OUTPUT(&format!("Pre-allocated the HashMap with {} buckets consumed {} bytes", 2*iterations_per_pass, hashmap_allocation_statistics.allocated_bytes - hashmap_allocation_statistics.deallocated_bytes));
    test_crud_algorithms("Pre-allocated Hashmap<String, u32> with ParkingLot", 15,
           |_n| {
               let mut hashmap = map_locker.write();
               hashmap.clear();
               //hashmap.shrink_to_fit();   we're using a pre-allocated hash map
               hashmap.len() as u32
           },
           |n| {
               let key = format!("key for {}", n);
               let mut hashmap = map_locker.write();
               hashmap.insert(key, n);
               hashmap.len() as u32
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           |n| {
               let key = format!("key for {}", n);
               let hashmap = map_locker.read();
               hashmap[&key]
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           |n| {
               let key = format!("key for {}", n);
               let mut hashmap = map_locker.write();
               hashmap.insert(key, n+1);
               hashmap.len() as u32
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           |n| {
               let key = format!("key for {}", n);
               let mut hashmap = map_locker.write();
               hashmap.remove(&key).unwrap_or_default()
           }, BigOAlgorithmComplexity::O1, BigOAlgorithmComplexity::O1,
           20, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
           n_threads, n_threads, n_threads, n_threads,
           &TimeUnits::MICROSECOND);
}