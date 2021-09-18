//! Exports time & space algorithm complexity analysis functions, as well as the needed types to operate on them. See:
//!   - [time_analysis]
//!   - [space_analysis]
//!   - [types]
//!
//! ... and, most importantly, tests both analysis on real functions. See [tests].

pub mod types;
pub mod time_analysis;
pub mod space_analysis;

use crate::{
    conditionals::{self},
    big_o_analysis::{
        types::{TimeUnit, TimeUnits, BigOTimePassMeasurements, BigOSpacePassMeasurements},
    }
};

use std::convert::TryInto;
use std::ops::Range;
use std::time::{SystemTime, Duration};


/// Runs a pass on the given 'algorithm' callback function or closure,
/// measuring (and returning) the time it took to run all iterations specified in 'range'.
/// ```
///     /// Algorithm function under analysis -- receives the iteration number on each call
///     /// (for set changing algorithms) or the set size (for constant set algorithms).
///     /// Returns any computed number to avoid compiler call cancellation optimizations
///     fn algorithm(i: u32) -> u32 {0}
/// ```
/// returns: tuple with (elapsed_time: u64, computed_number: u32)
pub(crate) fn run_pass<'a, _AlgorithmClosure: Fn(u32) -> u32 + Sync, ScalarDuration: TryInto<u64> + Copy>
(algorithm: &_AlgorithmClosure, algorithm_type: &BigOAlgorithmType, range: Range<u32>, time_unit: &'a TimeUnit<ScalarDuration>, threads: u32)
 -> (PassResult<'a,ScalarDuration>, u32) {

    type ThreadLoopResult = (Duration, u32);

    fn thread_loop<_AlgorithmClosure: Fn(u32) -> u32 + Sync>
    (algorithm: &_AlgorithmClosure, algorithm_type: &BigOAlgorithmType, range: Range<u32>) -> ThreadLoopResult {
        let mut thread_r: u32 = range.end;

        let thread_start = SystemTime::now();

        // run 'algorithm()' allowing normal or reversed order
        match algorithm_type {
            BigOAlgorithmType::ConstantSet => {
                if range.end < range.start {
                    for e in (range.end..range.start).rev() {
                        thread_r ^= algorithm(e);
                    }
                } else {
                    for e in range {
                        thread_r ^= algorithm(e);
                    }
                }
            },
            BigOAlgorithmType::SetResizing => {
                if range.end < range.start {
                    for e in (range.end..range.start).rev() {
                        thread_r ^= algorithm(e);
                    }
                } else {
                    for e in range {
                        thread_r ^= algorithm(e);
                    }
                }
            },
        }

        let thread_end = SystemTime::now();
        let thread_duration = thread_end.duration_since(thread_start).unwrap();

        (thread_duration, thread_r)
    }

    // use crossbeam's scoped threads to avoid requiring a 'static lifetime for our algorithm closure
    crossbeam::scope(|scope| {

        // start all threads
        let i32_range = range.end as i32 .. range.start as i32;
        let chunk_size = (i32_range.end-i32_range.start)/threads as i32;
        let mut thread_handlers: Vec<crossbeam::thread::ScopedJoinHandle<ThreadLoopResult>> = Vec::with_capacity(threads as usize);
        let allocator_savepoint = conditionals::ALLOC.save_point();
        for n in 0..threads as i32 {
            let chunked_range = i32_range.start+chunk_size*n..i32_range.start+chunk_size*(n+1);
            thread_handlers.push( scope.spawn(move |_| thread_loop(algorithm, algorithm_type, chunked_range.start as u32 .. chunked_range.end as u32)) );
        }

        // wait for them all to finish
        let mut r = range.start+1;
        let mut elapsed_average = 0.0f64;
        for handler in thread_handlers {
            let joining_result = handler.join();
            if joining_result.is_err() {
                panic!("Panic! while running provided 'algorithm' closure: algo type: {:?}, range: {:?}: Error: {:?}", algorithm_type, range, joining_result.unwrap_err())
            }
            let (thread_duration, thread_r) = joining_result.unwrap();
            let thread_elapsed = (time_unit.duration_conversion_fn_ptr)(&thread_duration).try_into().unwrap_or_default();
            elapsed_average += thread_elapsed as f64 / threads as f64;
            r ^= thread_r;
        }

        let allocator_statistics = conditionals::ALLOC.delta_statistics(&allocator_savepoint);

        (PassResult {
            time_measurements:  BigOTimePassMeasurements {
                elapsed_time: elapsed_average.round() as u64,
                time_unit,
            },
            space_measurements: BigOSpacePassMeasurements {
                used_memory_before: allocator_savepoint.metrics.current_used_memory,
                used_memory_after:  allocator_statistics.current_used_memory,
                min_used_memory:    allocator_statistics.min_used_memory,
                max_used_memory:    allocator_statistics.max_used_memory,
            },
        }, r)

    }).unwrap()

}

/// contains the measurements for a pass done in [run_pass()]
#[derive(Clone,Copy)]
pub struct PassResult<'a,ScalarTimeUnit: Copy> {
    pub time_measurements:  BigOTimePassMeasurements<'a,ScalarTimeUnit>,
    pub space_measurements: BigOSpacePassMeasurements,
}
impl<ScalarTimeUnit: Copy> Default for PassResult<'_,ScalarTimeUnit> {
    fn default() -> Self {
        Self {
            time_measurements: BigOTimePassMeasurements {
                elapsed_time: 0,
                time_unit: &TimeUnits::get_const_default(),
            },
            space_measurements: BigOSpacePassMeasurements {
                used_memory_before: 0,
                used_memory_after:  0,
                min_used_memory:    0,
                max_used_memory:    0,
            }
        }
    }
}

#[derive(Debug)]
/// Specifies if the algorithm under analysis alters the data set it works on or if it has no side-effects on it
/// Different math applies on each case, as well as different parameters to the 'algorithm(u32) -> u32' function.
pub enum BigOAlgorithmType {
    /// the algorithm under analysis change the data set size it operates on. Examples: insert/delete, enqueue/dequeue, ...
    SetResizing,
    /// the algorithm under analysis doesn't change the data set size it operates on. Examples: queries, sort, fib, ...
    ConstantSet,
}


#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [low_level_analysis](super) module -- using 'serial_test' crate in order to make time measurements more reliable.

    use super::*;

    use crate::big_o_analysis::types::*;
    use crate::big_o_analysis::time_analysis::*;
    use crate::big_o_analysis::space_analysis::*;

    use crate::{
        conditionals::{self,OUTPUT},
        big_o_analysis::types::{TimeUnit,TimeUnits}
    };

    use std::ops::Range;
    use std::convert::TryInto;

    use serial_test::serial;

    const BUSY_LOOP_DELAY: u32 = 999*conditionals::LOOP_MULTIPLIER;

    /// assures serializations & implementors of *Display* from [types] work as they should
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial(cpu)]
    fn serialization() {
        OUTPUT("BigOAlgorithmComplexity enum members, as strings:\n");
        let enum_members = [
            BigOAlgorithmComplexity::BetterThanO1,
            BigOAlgorithmComplexity::O1,
            BigOAlgorithmComplexity::OLogN,
            BigOAlgorithmComplexity::BetweenOLogNAndON,
            BigOAlgorithmComplexity::ON,
            BigOAlgorithmComplexity::WorseThanON,
        ];
        for enum_member in enum_members {
            OUTPUT(&format!("\t{:?} => '{}'\n", enum_member, enum_member.as_pretty_str()));
        }
        OUTPUT("\n");
    }

    /// tests time & space complexity analysis on real constant set algorithms
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial(cpu)]
    fn analyse_constant_set_algorithm_real_test() {

        const REPETITIONS: u32 = 3072;
        const PASS_1_SET_SIZE: u32 = REPETITIONS;
        const PASS_2_SET_SIZE: u32 = REPETITIONS * 3;
        const TIME_UNIT: &TimeUnit<u128> = &TimeUnits::MICROSECOND;

        fn o_1_select(mut _n: u32) -> u32 {
            // single element allocation & busy_loop time processing
            let vec = vec![busy_loop(BUSY_LOOP_DELAY*5)];
            vec.iter().sum()
        }

        fn o_log_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 1;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut len = 0;
            while n > 0 {
                r += busy_loop(BUSY_LOOP_DELAY/2);
                n /= 2;
                len += 1;
            }
            let vec = Vec::<u32>::with_capacity(len*400);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        fn o_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 2;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut len = 0;
            while n > 0 {
                r += busy_loop(BUSY_LOOP_DELAY/200);
                n -= 1;
                len += 1;
            }
            let vec = Vec::<u32>::with_capacity(len);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        let analyze = |measurement_name, select_function: fn(u32) -> u32| {
            OUTPUT(&format!("Real '{}', fetching {} elements on each pass ", measurement_name, REPETITIONS));

            let (_warmup_result               , r1) = _run_pass("(warmup: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. REPETITIONS / 10,                            TIME_UNIT);
            let (pass_1_result, r2) = _run_pass("; pass1: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. PASS_1_SET_SIZE,                             TIME_UNIT);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", select_function, &BigOAlgorithmType::ConstantSet, PASS_2_SET_SIZE - REPETITIONS .. PASS_2_SET_SIZE, TIME_UNIT);

            let constant_set_passes_info = ConstantSetAlgorithmPassesInfo {
                pass_1_set_size: PASS_1_SET_SIZE,
                pass_2_set_size: PASS_2_SET_SIZE,
                repetitions: REPETITIONS,
            };

            let time_measurements = BigOTimeMeasurements {
                pass_1_measurements: pass_1_result.time_measurements,
                pass_2_measurements: pass_2_result.time_measurements,
            };
            
            let space_measurements = BigOSpaceMeasurements {
                pass_1_measurements: pass_1_result.space_measurements,
                pass_2_measurements: pass_2_result.space_measurements,
            };

            let time_complexity  = analyse_time_complexity_for_constant_set_algorithm(&constant_set_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_constant_set_algorithm(&constant_set_passes_info, &space_measurements);

            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: ConstantSetAlgorithmMeasurements {
                    measurement_name,
                    passes_info: constant_set_passes_info,
                    time_measurements,
                    space_measurements
                },
            };

            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1+r2+r3));
            algorithm_analysis
        };

        let assert_with_retry = |max_retries, measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            for attempt in 1..max_retries+1 {
                let algorithm_analysis = analyze(measurement_name, insert_function);
                assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
                if algorithm_analysis.time_complexity != expected_complexity && attempt < max_retries {
                    OUTPUT("\n==> Time measurement mismatch. Retrying...\n\n");
                    continue;
                }
                assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
                break;
            }
        };

        assert_with_retry(15, "O1_select() function",    o_1_select,     BigOAlgorithmComplexity::O1);
        assert_with_retry(15, "OLogN_select() function", o_log_n_select, BigOAlgorithmComplexity::OLogN);
        assert_with_retry(15, "ON_select() function",    o_n_select,     BigOAlgorithmComplexity::ON);

    }

    /// tests time & space complexity analysis on real set resizing algorithms
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_real_test() {

        const DELTA_SET_SIZE: u32 = 3072;

        fn o_1_insert(mut _n: u32) -> u32 {
            // single element allocation & busy_loop time processing
            let vec = vec![busy_loop(BUSY_LOOP_DELAY*2)];
            vec.iter().sum()
        }

        fn o_log_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let mut len = if n==DELTA_SET_SIZE-1 {DELTA_SET_SIZE*2/3} else {0};
            while n > 0 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/2);
                n = n/2;
                len += n;
            }
            let vec = Vec::<u32>::with_capacity(len as usize);
            r ^ (len + vec.iter().sum::<u32>())
        }

        /// this would be an O(n/2) function -- the average case for a naive sorted insert... but still O(n). Change n = n-2 to n = n-1 and the analysis will be the same.
        fn o_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let len = if n<=DELTA_SET_SIZE {(n/20)*(n/20)} else {(n/20)*(n/20)-(n/40)*(n/40)};
            while n > 1 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/50);
                n = n-2;
            }
            let vec = Vec::<u32>::with_capacity(len as usize * 400);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        let analyze = |measurement_name, insert_function: fn(u32) -> u32| {
            OUTPUT(&format!("Real '{}' with {} elements on each pass ", measurement_name, DELTA_SET_SIZE));

            /* warmup pass -- container / database should be reset before and after this */
            let (_warmup_result, r1) = _run_pass("(warmup: ", "", insert_function, &BigOAlgorithmType::SetResizing, 0 .. DELTA_SET_SIZE / 10, &TimeUnits::MICROSECOND);
            /* if we were operating on real data, we would reset the container / database after the warmup, before running pass 1 */
            let (pass_1_result, r2) = _run_pass("; pass1: ", "", insert_function, &BigOAlgorithmType::SetResizing, 0 ..DELTA_SET_SIZE, &TimeUnits::MICROSECOND);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", insert_function, &BigOAlgorithmType::SetResizing, DELTA_SET_SIZE.. DELTA_SET_SIZE * 2, &TimeUnits::MICROSECOND);

            let set_resizing_passes_info = SetResizingAlgorithmPassesInfo { delta_set_size: DELTA_SET_SIZE };

            let time_measurements = BigOTimeMeasurements {
                pass_1_measurements: pass_1_result.time_measurements,
                pass_2_measurements: pass_2_result.time_measurements,
            };

            let space_measurements = BigOSpaceMeasurements {
                pass_1_measurements: pass_1_result.space_measurements,
                pass_2_measurements: pass_2_result.space_measurements,
            };

            let time_complexity  = analyse_time_complexity_for_set_resizing_algorithm(&set_resizing_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_set_resizing_algorithm(&set_resizing_passes_info, &space_measurements);
            
            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: SetResizingAlgorithmMeasurements {
                    measurement_name,
                    passes_info: set_resizing_passes_info,
                    time_measurements,
                    space_measurements,
                },
            };
            
            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1^r2^r3));
            algorithm_analysis
        };

        let assert_with_retry = |max_retries, measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            for attempt in 1..max_retries+1 {
                let algorithm_analysis = analyze(measurement_name, insert_function);
                assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
                if algorithm_analysis.time_complexity != expected_complexity && attempt < max_retries {
                    OUTPUT("\n==> Time measurement mismatch. Retrying...\n\n");
                    continue;
                }
                assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
                break;
            }
        };

        assert_with_retry(15, "O1_insert() function",    o_1_insert,     BigOAlgorithmComplexity::O1);
        assert_with_retry(15, "OLogN_insert() function", o_log_n_insert, BigOAlgorithmComplexity::OLogN);
        assert_with_retry(15, "ON_insert() function",    o_n_insert,     BigOAlgorithmComplexity::ON);
    }

   #[inline]
    fn busy_loop(iterations: u32) -> u32 {
        let mut r: u32 = iterations;
        for i in 0..iterations {
            r ^= i;
        }
        r
    }

    /// wrap around the original 'run_pass' to output intermediate results
    fn _run_pass<'a,
                 _AlgorithmClosure: Fn(u32) -> u32 + Sync,
                 T: TryInto<u64> + Copy > (result_prefix: &str,
                                           result_suffix: &str,
                                           algorithm: _AlgorithmClosure,
                                           algorithm_type: &BigOAlgorithmType,
                                           range: Range<u32>,
                                           unit: &'a TimeUnit<T>) -> (PassResult<'a,T>, u32) {
        let (pass_result, r) = run_pass(&algorithm, algorithm_type, range, unit, 1);
        OUTPUT(&format!("{}{}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
        (pass_result, r)
    }

}