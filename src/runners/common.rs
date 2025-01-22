//! Contains code shared between this module's submodules

use crate::{
    features,
    low_level_analysis::types::*,
};
use std::{
    ops::Range,
    time::{Instant, Duration},
};
use std::fmt::Debug;
use std::future::Future;
use std::hint::black_box;

/// wrap around the original [run_iterator_pass()] to output progress & intermediate results
pub fn run_iterator_pass_verbosely<'a, _IteratorAlgorithmClosure: Fn(u32) -> u32 + Sync,
                                       _OutputClosure:            FnMut(&str)>
                                  (result_prefix:      &str,
                                   result_suffix:      &str,
                                   iterator_algorithm: &_IteratorAlgorithmClosure,
                                   algorithm_type:     &BigOIteratorAlgorithmType,
                                   range:              Range<u32>,
                                   threads:            u32,
                                   mut output:         _OutputClosure)
                                  -> (PassResult, u32) {
    let (pass_result, r) = run_iterator_pass(iterator_algorithm, algorithm_type, range, threads);
    output(&format!("{}{:?}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
    (pass_result, r)
}

/// wrap around the original [run_sync_pass()] to output progress & intermediate results
pub fn run_sync_pass_verbosely<'a, _OutputClosure:    FnMut(&str)>
                              (result_prefix:  &str,
                               result_suffix:  &str,
                               algorithm:      impl FnMut() -> u32,
                               mut output:     _OutputClosure)
                              -> (PassResult, u32) {
    let (pass_result, r) = run_sync_pass(algorithm);
    output(&format!("{}{:?}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
    (pass_result, r)
}

/// wrap around the original [run_async_pass()] to output progress & intermediate results
pub async fn run_async_pass_verbosely<AlgorithmPassFn:   FnMut(Option<AlgoDataType>) -> AlgorithmPassFut + Send + Sync,
                                      AlgorithmPassFut:  Future<Output=AlgoDataType> + Send,
                                      AlgoDataType:      Send + Sync + Debug>
                                     (result_prefix:      &str,
                                      result_suffix:      &str,
                                      algo_data:          Option<AlgoDataType>,
                                      algorithm_pass_fn:  AlgorithmPassFn,
                                      mut output:         impl FnMut(&str))
                                     -> (PassResult, AlgoDataType) {
    let (pass_result, algo_data) = run_async_pass(algo_data, algorithm_pass_fn).await;
    output(&format!("{}{:?}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
    (pass_result, algo_data)
}



/// Runs a pass on the given `iterator_algorithm` callback function or closure,
/// measuring (and returning) the time it took to run all iterations specified in `range`
/// -- with the option to run the iteration of the given number of `threads`.\
/// An `iterator_algorithm` is one that provides 1 element on each call or processes 1 element on each call.\
/// See [run_sync_pass()] for algorithms which generates or operates on several elements per call.
/// ```
///     /// Iterator Algorithm function under analysis -- receives the iteration number on each call
///     /// (for set changing algorithms) or the set size (for constant set algorithms).
///     /// Returns a(ny) computed number based on the input -- to avoid compiler call cancellation optimizations
///     fn iterator_algorithm(i: u32) -> u32 {0}
/// ```
/// returns: tuple with ([PassResult], computed_number: u32)
pub(crate) fn run_iterator_pass<'a, _AlgorithmClosure: Fn(u32) -> u32 + Sync>
                               (iterator_algorithm: &_AlgorithmClosure,
                                algorithm_type:     &BigOIteratorAlgorithmType,
                                range:              Range<u32>,
                                threads:            u32)
                               -> (PassResult, u32) {

    type ThreadLoopResult = (Duration, u32);

    fn thread_loop<_AlgorithmClosure: Fn(u32) -> u32 + Sync>
                  (iterator_algorithm: &_AlgorithmClosure, algorithm_type: &BigOIteratorAlgorithmType, range: Range<u32>)
                   -> ThreadLoopResult {
        let mut thread_r: u32 = range.end;

        let thread_start = Instant::now();

        // run 'algorithm()' allowing normal or reversed order
        match algorithm_type {
            BigOIteratorAlgorithmType::ConstantSet => {
                if range.end < range.start {
                    for e in (range.end..range.start).rev() {
                        thread_r ^= iterator_algorithm(e);
                    }
                } else {
                    for e in range {
                        thread_r ^= iterator_algorithm(e);
                    }
                }
            },
            BigOIteratorAlgorithmType::SetResizing => {
                if range.end < range.start {
                    for e in (range.end..range.start).rev() {
                        thread_r ^= iterator_algorithm(e);
                    }
                } else {
                    for e in range {
                        thread_r ^= iterator_algorithm(e);
                    }
                }
            },
        }

        let thread_end = Instant::now();
        let thread_duration = thread_end.duration_since(thread_start);

        (thread_duration, thread_r)
    }

    // use crossbeam's scoped threads to avoid requiring a 'static lifetime for our algorithm's closure
    crossbeam::scope(|scope| {

        // start all threads
        let i32_range = range.end as i32 .. range.start as i32;
        let chunk_size = (i32_range.end-i32_range.start)/threads as i32;
        let mut thread_handlers: Vec<crossbeam::thread::ScopedJoinHandle<ThreadLoopResult>> = Vec::with_capacity(threads as usize);
        let allocator_savepoint = features::ALLOC.save_point();
        for n in 0..threads as i32 {
            let chunked_range = i32_range.start+chunk_size*n..i32_range.start+chunk_size*(n+1);
            thread_handlers.push( scope.spawn(move |_| thread_loop(iterator_algorithm, algorithm_type, chunked_range.start as u32 .. chunked_range.end as u32)) );
        }

        // wait for them all to finish
        let mut r = range.start+1;
        let mut elapsed_seconds_average = 0.0f64;
        for handler in thread_handlers {
            let joining_result = handler.join();
            if joining_result.is_err() {
                panic!("Panic! while running provided 'algorithm' closure: algo type: {:?}, range: {:?}: Error: {:?}", algorithm_type, range, joining_result.unwrap_err())
            }
            let (thread_duration, thread_r) = joining_result.unwrap();
            let thread_elapsed_seconds = thread_duration.as_secs_f64();
            elapsed_seconds_average += thread_elapsed_seconds as f64 / threads as f64;
            r ^= thread_r;
        }

        let allocator_statistics = features::ALLOC.delta_statistics(&allocator_savepoint);

        (PassResult {
            time_measurements:  Duration::from_secs_f64(elapsed_seconds_average),
            space_measurements: BigOSpacePassMeasurements {
                used_memory_before: allocator_savepoint.metrics.current_used_memory,
                used_memory_after:  allocator_statistics.current_used_memory,
                min_used_memory:    allocator_statistics.min_used_memory,
                max_used_memory:    allocator_statistics.max_used_memory,
            },
        }, r)

    }).unwrap()

}

/// Runs a pass on the given synchronous `algorithm` callback function or closure,
/// measuring (and returning) the time it took to run it.\
/// See [run_iterator_pass()] for algorithms which generates or operates on a single element per call.
/// ```
///     /// Algorithm function under analysis.
///     /// Returns a(ny) computed number to avoid compiler call cancellation optimizations
///     fn algorithm() -> u32 {0}
/// ```
/// returns: tuple with ([PassResult]], computed_number: u32)
///
/// See also [run_async_pass()]
pub(crate) fn run_sync_pass(mut algorithm:  impl FnMut() -> u32)
                           -> (PassResult, u32) {

    let allocator_savepoint = features::ALLOC.save_point();
    let start = Instant::now();
    let r = algorithm();
    let duration = start.elapsed();
    let allocator_statistics = features::ALLOC.delta_statistics(&allocator_savepoint);

    (PassResult {
        time_measurements:  duration,
        space_measurements: BigOSpacePassMeasurements {
            used_memory_before: allocator_savepoint.metrics.current_used_memory,
            used_memory_after:  allocator_statistics.current_used_memory,
            min_used_memory:    allocator_statistics.min_used_memory,
            max_used_memory:    allocator_statistics.max_used_memory,
        },
    }, r)
}

/// Runs a pass on the given asynchronous `algorithm` callback function or closure,
/// measuring (and returning) the time it took to run it.\
/// See [run_iterator_pass()] for algorithms which generates or operates on a single element per call.
/// ```nocompile
///     /// Algorithm function under analysis.
///     /// Returns a(ny) computed number to avoid compiler call cancellation optimizations
///     async fn algorithm(algo_data: Option<AlgoDataType>) -> AlgoDataType { ... }
/// ```
/// returns: tuple with ([PassResult]], algo_data: `AlgoDataType`)
///
/// See also [run_async_pass()]
pub(crate) async fn run_async_pass<AlgorithmPassFn:   FnMut(Option<AlgoDataType>) -> AlgorithmPassFut + Send + Sync,
                                   AlgorithmPassFut:  Future<Output=AlgoDataType> + Send,
                                   AlgoDataType:      Send + Sync + Debug>
                                  (algo_data:              Option<AlgoDataType>,
                                   mut algorithm_pass_fn:  AlgorithmPassFn)
                                  -> (PassResult, AlgoDataType) {

    let allocator_savepoint = features::ALLOC.save_point();
    let start = Instant::now();
    let algo_data = black_box(algorithm_pass_fn(algo_data).await);
    let duration = start.elapsed();
    let allocator_statistics = features::ALLOC.delta_statistics(&allocator_savepoint);

    (PassResult {
        time_measurements:  duration,
        space_measurements: BigOSpacePassMeasurements {
            used_memory_before: allocator_savepoint.metrics.current_used_memory,
            used_memory_after:  allocator_statistics.current_used_memory,
            min_used_memory:    allocator_statistics.min_used_memory,
            max_used_memory:    allocator_statistics.max_used_memory,
        },
    }, algo_data)
}

/// contains the measurements for a pass done in [run_sync_pass()]
#[derive(Clone,Copy)]
pub struct PassResult {
    pub time_measurements:  Duration,
    pub space_measurements: BigOSpacePassMeasurements,
}
impl Default for PassResult {
    fn default() -> Self {
        Self {
            time_measurements: Duration::default(),
            space_measurements: BigOSpacePassMeasurements {
                used_memory_before: 0,
                used_memory_after:  0,
                min_used_memory:    0,
                max_used_memory:    0,
            }
        }
    }
}

