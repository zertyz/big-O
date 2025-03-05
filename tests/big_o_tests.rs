//! Applies the Big-O crate to some Rust's std lib containers

use big_o_test::*;
use std::{
    sync::Arc,
    collections::HashMap,
};
use std::time::Duration;
use ctor::ctor;
use big_o_test::RegularAsyncAnalyzerBuilder;

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


#[test]
fn quick_sort_reversed_vec() {
    const VEC1_LEN: u32 = 40000000;
    const VEC2_LEN: u32 = 80000000;
    let vec1 = parking_lot::RwLock::new(Vec::<u32>::new());
    let vec2 = parking_lot::RwLock::new(Vec::<u32>::new());
    test_algorithm(
        "Quicksort a reversed vec", 15,
        || {
            let mut vec1 = vec1.write();
            for i in 0..VEC1_LEN {
                vec1.push(i);
            }
            let mut vec2 = vec2.write();
            for i in 0..VEC2_LEN {
                vec2.push(i);
            }
        },
        VEC1_LEN, || {
            let mut vec1 = vec1.write();
            vec1.sort();
            vec1[12]
        },
        VEC2_LEN, || {
            let mut vec2 = vec2.write();
            vec2.sort();
            vec2[14]
        },
        BigOAlgorithmComplexity::ON, BigOAlgorithmComplexity::ON,
    )
}


/// Attests the best case CRUD for vectors -- Create, Read, Update and Delete... all O(1):
///   - inserts at the end (push)
///   - deletes at the end (pop)
#[test]
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
            n_threads, n_threads, n_threads, n_threads);
}

/// Attests the worst case CRUD for vectors:
///   - Create always at the beginning -- O(n)
///   - Delete always at the beginning -- O(n)
///   - Reads and updates as the usual O(1)
#[test]
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
               let val = (iterations_per_pass as u32)*5 - n;
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
           n_threads, n_threads, n_threads, n_threads);
}

/// Attests O(1) performance characteristics for HashMaps
#[test]
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
           n_threads, n_threads, n_threads, n_threads);
}

#[tokio::test]
async fn dummy_async_test() {
}

#[tokio::test]
async fn async_futures_are_send() {
    let fut =     RegularAsyncAnalyzerBuilder::new("dummy analysis")
        .first_pass(10, |_: Option<()>| tokio::time::sleep(Duration::from_millis(100)))
        .second_pass(20, |_: Option<()>| tokio::time::sleep(Duration::from_millis(200)))
        .test_algorithm();  // notice no .await here, so we get the raw `Future`
    tokio::task::spawn(fut).await.expect("Tokio Error");
}