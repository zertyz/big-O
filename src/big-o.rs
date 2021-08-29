pub mod big_o_analysis;
mod conditionals;
mod metrics_allocator;
mod ring_buffer;

use crate::big_o_analysis::{SetResizingAlgorithmMeasurements, ConstantSetAlgorithmMeasurements, BigOAlgorithmAnalysis, BigOAlgorithmComplexity};
use crate::conditionals::OUTPUT;

use std::convert::TryInto;
use std::ops::Range;
use std::time::{SystemTime, Duration};
use std::io;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};

pub fn analyze_crud_algorithm<'a,
                              _ResetClosure:  Fn(u32) -> u32 + Sync,
                              _CreateClosure: Fn(u32) -> u32 + Sync,
                              _ReadClusure:   Fn(u32) -> u32 + Sync,
                              _UpdateClosure: Fn(u32) -> u32 + Sync,
                              _DeleteClosure: Fn(u32) -> u32 + Sync,
                              T: TryInto<u64> > (crud_name: &'a str,
                                                 reset_fn:  _ResetClosure,
                                                 create_fn: _CreateClosure,
                                                 read_fn:   _ReadClusure,
                                                 update_fn: _UpdateClosure,
                                                 delete_fn: _DeleteClosure,
                                                 warmup_percentage: u32, create_iterations_per_pass: u32, read_iterations_per_pass: u32, update_iterations_per_pass: u32, delete_iterations_per_pass: u32,
                                                 create_threads: u32, read_threads: u32, update_threads: u32, delete_threads: u32,
                                                 time_unit: &TimeUnit<T>)
        -> (BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a>>,    // create analysis
            BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a>>,    // read analysis
            BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a>>,    // update analysis
            BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a>>,    // delete analysis
            String) {                                                       // full report

    let mut full_report = String::new();

    // wrap around the original 'OUTPUT' function to capture the [full_report]
    let mut _output = |msg: &str| {
        full_report.push_str(msg);
        OUTPUT(msg);
    };

    /// wrap around the original 'run_pass' to output intermediate results
    fn run_pass_verbosely<_AlgorithmClosure: Fn(u32) -> u32 + Sync,
                 _OutputClosure:    FnMut(&str),
                 T: TryInto<u64>> (result_prefix: &str, result_suffix: &str,
                                   algorithm: &_AlgorithmClosure, algorithm_type: &BigOAlgorithmType, range: Range<u32>, unit: &TimeUnit<T>,
                                   threads: u32, mut _output: _OutputClosure)
                -> (PassResult, u32) {
        _output(&format!("{}", result_prefix));
        let (pass_result, r) = run_pass(algorithm, algorithm_type, range, unit, threads);
        let used_memory = pass_result.delta_bytes as f32;   // intermediate results will show the pass resulting used memory (instead of the min/max used mem, used for analysis)
        let sign = if used_memory > 0.0 {"+"} else {"-"};
        let memory_unit = if used_memory.abs() > (1<<30) as f32 {"G"}                          else if used_memory.abs() > (1<<20) as f32 {"M"}                                else if used_memory.abs() > (1<<10) as f32 {"K"} else {"b"};
        let memory_delta = if used_memory.abs() > (1<<30) as f32 { used_memory /(1<<30) as f32} else if used_memory.abs() > (1<<20) as f32 { used_memory.abs() /(1<<20) as f32} else if used_memory.abs() > (1<<10) as f32 { used_memory.abs() /(1<<10) as f32} else { used_memory.abs() };
        let memory_stats = format!("{}{:.2}{}", sign, memory_delta, memory_unit);
        _output(&format!("{}{}/{}{}", pass_result.elapsed_time, unit.unit_str, memory_stats, result_suffix));
        (pass_result, r)
    }

    const NUMBER_OF_PASSES: u32 = 2;

    let mut create_passes_results = [PassResult{elapsed_time: 0, delta_bytes: 0,  min_max_delta_bytes: 0}; NUMBER_OF_PASSES as usize];
    let mut   read_passes_results = [PassResult{elapsed_time: 0, delta_bytes: 0,  min_max_delta_bytes: 0}; NUMBER_OF_PASSES as usize];
    let mut update_passes_results = [PassResult{elapsed_time: 0, delta_bytes: 0,  min_max_delta_bytes: 0}; NUMBER_OF_PASSES as usize];
    let mut delete_passes_results = [PassResult{elapsed_time: 0, delta_bytes: 0,  min_max_delta_bytes: 0}; NUMBER_OF_PASSES as usize];

    // computed result to avoid any call cancellation optimizations when running in release mode
    let mut r: u32 = 0;

    _output(&format!("{} CRUD Algorithm Complexity Analysis:\n  ", crud_name));

    // range calculation
    fn calc_regular_CRU_range(iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * pass_number       .. iterations_per_pass * (pass_number + 1) }
    fn calc_regular_D_range  (iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * (pass_number + 1) .. iterations_per_pass * pass_number }
    let calc_warmup_CRU_range = |iterations_per_pass|  0 .. iterations_per_pass * warmup_percentage / 100;
    let calc_warmup_D_range   = |iterations_per_pass| iterations_per_pass * warmup_percentage / 100 .. 0;


    // now the real deal: run the passes & CRUD operations
    for pass in 0..NUMBER_OF_PASSES {

        // warmup (only on the first pass)
        if pass == 0 && warmup_percentage > 0 {

            let warmup_start = SystemTime::now();
            _output("warming up [");
            io::stdout().flush().unwrap();
            if create_iterations_per_pass > 0 {
                _output(&"C");
                let (_elapse, warmup_r) = run_pass(&create_fn, &BigOAlgorithmType::SetResizing, calc_warmup_CRU_range(create_iterations_per_pass), time_unit, create_threads);
                r ^= warmup_r;
            }
            if read_iterations_per_pass > 0 {
                _output(&"R");
                let (_elapse, warmup_r) = run_pass(&read_fn, &BigOAlgorithmType::ConstantSet, calc_warmup_CRU_range(read_iterations_per_pass), time_unit, read_threads);
                r ^= warmup_r;
            }
            if update_iterations_per_pass > 0 {
                _output(&"U");
                let (_elapse, warmup_r) = run_pass(&update_fn, &BigOAlgorithmType::ConstantSet, calc_warmup_CRU_range(update_iterations_per_pass), time_unit, update_threads);
                r ^= warmup_r;
            }
            if delete_iterations_per_pass > 0 {
                let delete_range = delete_iterations_per_pass * (pass + 1) .. delete_iterations_per_pass * pass;
                _output(&"D");
                let (_elapse, warmup_r) = run_pass(&delete_fn, &BigOAlgorithmType::SetResizing, calc_warmup_D_range(delete_iterations_per_pass), time_unit, delete_threads);
                r ^= warmup_r;
            }
            reset_fn(warmup_percentage);
            _output("] ");

            let warmup_end = SystemTime::now();
            let warmup_duration = warmup_end.duration_since(warmup_start).unwrap();
            let warmup_elapsed = (time_unit.duration_conversion_fn_ptr)(&warmup_duration).try_into().unwrap_or_default();
            _output(&format!("{}{}, ", warmup_elapsed, time_unit.unit_str));
        }

        // show pass number
        _output(&format!("{} Pass (", if pass == 0 {
            "First"
        } else {
            "); Second"
        }));

        // execute regular passes verbosely
        let (create_pass, cr) = run_pass_verbosely("create: ",   "", &create_fn, &BigOAlgorithmType::SetResizing, calc_regular_CRU_range(create_iterations_per_pass, pass), time_unit, create_threads, &mut _output);
        let (read_pass,   rr) = run_pass_verbosely("; read: ",   "", &read_fn,   &BigOAlgorithmType::ConstantSet, calc_regular_CRU_range(  read_iterations_per_pass, pass), time_unit,   read_threads, &mut _output);
        let (update_pass, ur) = run_pass_verbosely("; update: ", "", &update_fn, &BigOAlgorithmType::ConstantSet, calc_regular_CRU_range(update_iterations_per_pass, pass), time_unit, update_threads, &mut _output);

        create_passes_results[pass as usize] = create_pass;
          read_passes_results[pass as usize] = read_pass;
        update_passes_results[pass as usize] = update_pass;

        r += cr^rr^ur;
    }
    _output("):\n\n");

    // analyze & output "create", "read" and "update" reports
    let create_analysis = big_o_analysis::analyse_set_resizing_algorithm(SetResizingAlgorithmMeasurements {
        measurement_name: "Create",
        pass_1_total_time: create_passes_results[0].elapsed_time,
        pass_2_total_time: create_passes_results[1].elapsed_time,
        pass_1_max_mem:    create_passes_results[0].min_max_delta_bytes,
        pass_2_max_mem:    create_passes_results[1].min_max_delta_bytes,
        delta_set_size:    create_iterations_per_pass
    });
    let read_analysis = big_o_analysis::analyse_constant_set_algorithm(ConstantSetAlgorithmMeasurements {
        measurement_name: "Read",
        pass_1_total_time: read_passes_results[0].elapsed_time,
        pass_2_total_time: read_passes_results[1].elapsed_time,
        pass_1_max_mem:    read_passes_results[0].min_max_delta_bytes,
        pass_2_max_mem:    read_passes_results[1].min_max_delta_bytes,
        pass_1_set_size:   create_iterations_per_pass,
        pass_2_set_size:   create_iterations_per_pass * 2,
        repetitions:       read_iterations_per_pass,
    });
    let update_analysis = big_o_analysis::analyse_constant_set_algorithm(ConstantSetAlgorithmMeasurements {
        measurement_name: "Update",
        pass_1_total_time: update_passes_results[0].elapsed_time,
        pass_2_total_time: update_passes_results[1].elapsed_time,
        pass_1_max_mem:    update_passes_results[0].min_max_delta_bytes,
        pass_2_max_mem:    update_passes_results[1].min_max_delta_bytes,
        pass_1_set_size:   create_iterations_per_pass,
        pass_2_set_size:   create_iterations_per_pass * 2,
        repetitions: update_iterations_per_pass,
    });
    if create_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", create_analysis));
    }
    if read_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", read_analysis));
    }
    if update_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", update_analysis));
    }

    // delete passes (note that delete passes are applied in reverse order)
    if delete_iterations_per_pass > 0 {
        _output("Delete Passes (");
        for pass in (0..NUMBER_OF_PASSES).rev() {
            let msg = format!("{}: ", if pass == 0 {
                "; 1st"
            } else {
                "2nd"
            });
            let (delete_elapsed, dr) = run_pass_verbosely(&msg, "", &delete_fn, &BigOAlgorithmType::SetResizing, calc_regular_D_range(delete_iterations_per_pass, pass), time_unit, delete_threads, &mut _output);
            delete_passes_results[pass as usize] = delete_elapsed;
            r += dr;
        }
    }

    _output(&format!(") r={}:\n", r));

    // analyze & output "delete" report
    let delete_analysis = big_o_analysis::analyse_set_resizing_algorithm(SetResizingAlgorithmMeasurements {
        measurement_name: "Delete",
        pass_1_total_time: delete_passes_results[0].elapsed_time,
        pass_2_total_time: delete_passes_results[1].elapsed_time,
        pass_1_max_mem:    delete_passes_results[0].min_max_delta_bytes,
        pass_2_max_mem:    delete_passes_results[1].min_max_delta_bytes,
        delta_set_size:    delete_iterations_per_pass,
    });
    if delete_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", delete_analysis));
    }

    (create_analysis, read_analysis, update_analysis, delete_analysis, full_report)
}

/// experimental/rudimentary assertion macro to let an 'observed_complexity' better than 'expected_complexity' to pass,
/// in the hope to reduce false-negative test failures
#[macro_export]
macro_rules! assert_complexity {
    ($observed_complexity:expr, $expected_complexity:expr $(,)?) => ({
        match (&$observed_complexity, &$expected_complexity) {
            (observed_complexity_val, expected_complexity_val) => {
                if !(*observed_complexity_val as u32 <= *expected_complexity_val as u32) {
                    assert_eq!(observed_complexity_val, expected_complexity_val, "expected enum value: {}; observed: {} -- which is not equal or less than the expected", expected_complexity_val, observed_complexity_val);
                }
            }
        }
    });
    ($observed_complexity:expr, $expected_complexity:expr, $($arg:tt)+) => ({
        match (&$observed_complexity, &$expected_complexity) {
            (observed_complexity_val, expected_complexity_val) => {
                if !(*observed_complexity_val as u32 <= *expected_complexity_val as u32) {
                    assert_eq!(observed_complexity_val, expected_complexity_val, $($arg)+);
                }
            }
        }
    });
}

#[derive(Clone,Copy)]
struct PassResult {
    /// unit-less time took to run the pass
    elapsed_time: u64,
    /// delta between mem used at the end of the pass and mem used at the beginning of the pass --
    /// positive if pass mainly allocates, negative if it mainly frees memory.
    delta_bytes: i32,
    /// always positive delta between min mem used & max mem used during the pass execution.
    min_max_delta_bytes: u32,
}

/// Runs a pass on the given 'algorithm' callback function (see [AlgorithmFnPtr]),
/// measuring (and returning) the time it took to run all iterations specified in 'range'.
/// ````
///     /// Algorithm function under analysis -- receives the iteration number on each call
///     /// (for set changing algorithms) or the set size (for constant set algorithms).
///     /// Returns any computed number to avoid compiler call cancellation optimizations
///     fn algorithm(i: u32) -> u32 {0}
/// ````
/// returns: tuple with (elapsed_time: u64, computed_number: u32)
fn run_pass<_AlgorithmClosure: Fn(u32) -> u32 + Sync, ScalarDuration: TryInto<u64>>
           (algorithm: &_AlgorithmClosure, algorithm_type: &BigOAlgorithmType, range: Range<u32>, unit: &TimeUnit<ScalarDuration>, threads: u32)
           -> (PassResult, u32) {

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
            let thread_elapsed = (unit.duration_conversion_fn_ptr)(&thread_duration).try_into().unwrap_or_default();
            elapsed_average += thread_elapsed as f64 / threads as f64;
            r ^= thread_r;
        }

        let allocator_statistics = conditionals::ALLOC.delta_statistics(&allocator_savepoint);

        (PassResult {
            elapsed_time: elapsed_average as u64,
            delta_bytes: ( (allocator_statistics.allocated_bytes + allocator_statistics.reallocated_news_bytes + allocator_statistics.zeroed_allocated_bytes) as isize -
                           (allocator_statistics.deallocated_bytes + allocator_statistics.reallocated_originals_bytes) as isize) as i32,
            min_max_delta_bytes: allocator_statistics.max_used_memory as u32 - allocator_statistics.min_used_memory as u32,

        }, r)

    }).unwrap()

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

/// Specifies a time unit for the 'big-O' crate when measuring / reporting results.
/// Please use one of the prebuilt 'TimeUnits' constants:
/// TimeUnits::NANOSECOND, TimeUnits::MICROSECOND,TimeUnits::MILLISECOND,  TimeUnits::SECOND
pub struct TimeUnit<T> {
    /// printable unit
    unit_str: &'static str,
    /// on std::time::Duration 'as_micros', 'as_seconds', ... function to convert a Duration object into a scalar
    duration_conversion_fn_ptr: fn(&std::time::Duration) -> T,
}
/// prebuilt 'TimeUnit' constants
struct TimeUnits {}
impl TimeUnits {
    pub const NANOSECOND:  TimeUnit<u128> = TimeUnit { unit_str: "ns", duration_conversion_fn_ptr: std::time::Duration::as_nanos};
    pub const MICROSECOND: TimeUnit<u128> = TimeUnit { unit_str: "µs", duration_conversion_fn_ptr: std::time::Duration::as_micros};
    pub const MILLISECOND: TimeUnit<u128> = TimeUnit { unit_str: "ms", duration_conversion_fn_ptr: std::time::Duration::as_millis};
    pub const SECOND:      TimeUnit<u64>  = TimeUnit { unit_str: "s",  duration_conversion_fn_ptr: std::time::Duration::as_secs};
}


fn main() {
    println!("Welcome to the big-O notation spikes!");
    println!();
    //tests::lowLevelExperiments();
}


#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;
    use std::sync::Arc;
    use std::collections::HashMap;
    use crate::big_o_analysis::{BigOTimeMeasurements, BigOAlgorithmComplexity};
    use crate::conditionals::ALLOC;

    /// Attests that the right report structures are produced for all possible CRUD tests:
    ///   - progress is reported per pass, per operation (operation = create, read, update or delete)
    ///   - sub-reports are only created when 'iterations_per_pass' for the operation is > 0
    #[test]
    #[serial(cpu)]
    fn analyze_crud_algorithm_output_check() {
        let iterations_per_pass = 100000;

        // high level asserting functions
        /////////////////////////////////

        fn assert_contains_status(report: &str, excerpt: &str) {
            assert!(report.contains(excerpt), "no '{}' status was found on the full report", excerpt);
        }
        fn assert_does_not_contain_status(report: &str, excerpt: &str) {
            assert!(!report.contains(excerpt), "found '{}' status on the full report, where it shouldn't be", excerpt);
        }
        fn assert_contains_algorithm_report<T: BigOTimeMeasurements>(report: &str, algorithm_analysis: BigOAlgorithmAnalysis<T>, algorithm_name: &str) {
            assert!(report.contains(&algorithm_analysis.to_string()), "couldn't find '{}' report analysis on the full report", algorithm_name);
        }
        fn assert_does_not_contain_algorithm_report<T: BigOTimeMeasurements>(report: &str, algorithm_analysis: BigOAlgorithmAnalysis<T>, algorithm_name: &str) {
            assert!(!report.contains(&algorithm_analysis.to_string()), "found a '{}' report analysis that shouldn't be on the full report", algorithm_name);
        }

        // checks
        /////////

        // fully fledged output
        let (create_analysis,
             read_analysis,
             update_analysis,
             delete_analysis,
             report) = analyze_crud_algorithm("MyContainer",
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              iterations_per_pass /100,
                                              iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
                                              1, 1, 1, 1,
                                              &TimeUnits::NANOSECOND);
        assert!(report.contains("MyContainer"), "CRUD name not present on the full report");
        assert_contains_status(&report, "warming up [CRUD] ");
        assert_contains_status(&report, "create:");
        assert_contains_status(&report, "read:");
        assert_contains_status(&report, "update:");
        assert_contains_status(&report, "Delete Passes");
        assert_contains_algorithm_report(&report, create_analysis, "Create");
        assert_contains_algorithm_report(&report, read_analysis, "Read");
        assert_contains_algorithm_report(&report, update_analysis, "Update");
        assert_contains_algorithm_report(&report, delete_analysis, "Delete");

        // no warmup
        let (_create_analysis,
             _read_analysis,
             _update_analysis,
             _delete_analysis,
             report) = analyze_crud_algorithm("MyContainer",
                                              |_n| panic!("'reset_fn' should not be called if there is no warmup taking place"),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              0/* no warmup */, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
                                              1, 1, 1, 1,
                                              &TimeUnits::NANOSECOND);
        assert_does_not_contain_status(&report, "warmup:");

        // no delete as well
        let (_create_analysis,
            _read_analysis,
            _update_analysis,
            delete_analysis,
            report) = analyze_crud_algorithm("MyContainer",
                                             |_n| panic!("'reset_fn' should not be called if there is no warmup taking place"),
                                             |n| (n+1)/(n+1),
                                             &|n| (n+1)/(n+1),
                                             |n| (n+1)/(n+1),
                                             |_n| panic!("'delete_fn' should not be called if there is no warmup taking place"),
                                             0/*no warmup*/, iterations_per_pass, iterations_per_pass, iterations_per_pass, 0/*no delete*/,
                                             1, 1, 1, 0,
                                             &TimeUnits::NANOSECOND);
        assert_does_not_contain_status(&report, "Delete Passes");
        assert_does_not_contain_algorithm_report(&report, delete_analysis, "Delete");
    }

    /// Attests the same number of iterations are produced regardless of the number of threads:
    ///   - 'iterations_per_pass must' be a multiple of 'n_threads'
    #[test]
    #[serial(cpu)]
    fn thread_chunk_division() {
        let iterations_per_pass = 1000;
        for n_threads in [1,2,4,5,10] {
            let map_locker = parking_lot::RwLock::new(HashMap::<u32, u32>::with_capacity(2 * iterations_per_pass as usize));
            let max_length = AtomicU32::new(0);
            analyze_crud_algorithm("Push & Pop (best case) Vec with ParkingLot",
                                   |_n| {0},
                                   |n| {
                                       let mut map = map_locker.write();
                                       map.insert(n, n);
                                       if map.len() as u32 > max_length.load(Ordering::Relaxed) {
                                           max_length.store(map.len() as u32, Ordering::Relaxed);
                                       }
                                       max_length.load(Ordering::Relaxed)
                                   },
                                   |_n| {0},
                                   |_n| {0},
                                   |n| {
                                       let mut map = map_locker.write();
                                       assert_eq!(map.remove(&n), Some(n), "missing element #{} when inserting for n_threads {}", n, n_threads);
                                       map.len() as u32
                                   },
                                   0, iterations_per_pass, 0, 0, iterations_per_pass,
                                   n_threads, n_threads, n_threads, n_threads,
                                   &TimeUnits::MICROSECOND);
            let map = map_locker.read();
            assert_eq!(iterations_per_pass *2, max_length.load(Ordering::Relaxed), "failed to insert records when testing for n_threads {}", n_threads);
            assert_eq!(0, map.len(), "failed to delete records when testing for n_threads {}", n_threads);
        }
    }

    /// Attests the best case CRUD for vectors -- Create, Read, Update and Delete... all O(1):
    ///   - inserts at the end (push)
    ///   - deletes at the end (pop)
    #[test]
    #[serial(cpu)]
    fn vec_best_case_algorithm_analysis() {
let mem_save_point = ALLOC.save_point();
        let iterations_per_pass: u32 = 400_000*conditionals::LOOP_MULTIPLIER;
        let n_threads = 1;
        let vec_locker = parking_lot::RwLock::new(Vec::<u32>::with_capacity(0));
        let crud_analysis = analyze_crud_algorithm("Push & Pop (best case) Vec with ParkingLot",
                |_n| {
                            let mut vec = vec_locker.write();
                            vec.clear();
                            vec.len() as u32
                        },
                |n| {
                            let mut vec = vec_locker.write();
                            vec.push(n);
                            vec.len() as u32
                        },
                |n| {
                            let vec = vec_locker.read();
                            vec[n as usize]
                        },
                |n| {
                            let mut vec = vec_locker.write();
                            vec[n as usize] = n+1;
                            vec.len() as u32
                        },
                |_n| {
                            let mut vec = vec_locker.write();
                            vec.pop().unwrap()
                        },
                25, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
                n_threads, n_threads, n_threads, n_threads,
                &TimeUnits::MICROSECOND);
eprintln!("ALLOCATION STATS: {}", ALLOC.delta_statistics(&mem_save_point));
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.time_complexity, BigOAlgorithmComplexity::O1, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.time_complexity, BigOAlgorithmComplexity::O1, "DELETE complexity mismatch");
    }

    /// Attests the worst case CRUD for vectors:
    ///   - Create always at the beginning -- O(n)
    ///   - Delete always at the beginning -- O(n)
    ///   - Reads and updates as the usual O(1)
    #[test]
    #[serial(cpu)]
    fn vec_worst_case_algorithm_analysis() {
let mem_save_point = ALLOC.save_point();
        let iterations_per_pass: u32 = 25_000/* *conditionals::LOOP_MULTIPLIER*/;
        let n_threads = 1;
        let vec_locker = parking_lot::RwLock::new(Vec::<u32>::with_capacity(0));
        let crud_analysis = analyze_crud_algorithm("Insert & Remove (worst case) Vec with ParkingLot",
               |_n| {
                   let mut vec = vec_locker.write();
                   vec.clear();
                   //hashmap.shrink_to_fit();
                   vec.len() as u32
               },
               |n| {
                   let val = (iterations_per_pass as u32)*2 - n;
                   let mut vec = vec_locker.write();
                   vec.insert(0, val);
                   val
               },
               |n| {
                   let vec = vec_locker.read();
                   vec[(n % iterations_per_pass) as usize]
               },
               |n| {
                   let mut vec = vec_locker.write();
                   vec[(n % iterations_per_pass) as usize] = n;
                   vec.len() as u32
               },
               |_n| {
                   let mut vec = vec_locker.write();
                   vec.remove(0)
               },
               0, iterations_per_pass, iterations_per_pass*10, iterations_per_pass*10, iterations_per_pass,
               n_threads, n_threads, n_threads, n_threads,
               &TimeUnits::MICROSECOND);
eprintln!("ALLOCATION STATS: {}", ALLOC.delta_statistics(&mem_save_point));
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.time_complexity, BigOAlgorithmComplexity::ON, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.time_complexity, BigOAlgorithmComplexity::ON, "DELETE complexity mismatch");

    }

    /// Attests O(1) performance characteristics for HashMaps
    #[test]
    #[serial(cpu)]
    fn hashmap_algorithm_analysis() {
let mem_save_point = ALLOC.save_point();
        let iterations_per_pass = 40_000*conditionals::LOOP_MULTIPLIER;
        let n_threads = 1;
        let map_locker = Arc::new(parking_lot::RwLock::new(HashMap::<String, u32>::with_capacity(2 * iterations_per_pass as usize)));
        let crud_analysis = analyze_crud_algorithm("Hashmap<String, u32> with ParkingLot",
               |_n| {
                   let mut hashmap = map_locker.write();
                   hashmap.clear();
                   //hashmap.shrink_to_fit();
                   hashmap.len() as u32
               },
               |n| {
                   let key = format!("key for {}", n);
                   let mut hashmap = map_locker.write();
                   hashmap.insert(key, n);
                   hashmap.len() as u32
               },
               |n| {
                   let key = format!("key for {}", n);
                   let hashmap = map_locker.read();
                   hashmap[&key]
               },
               |n| {
                   let key = format!("key for {}", n);
                   let mut hashmap = map_locker.write();
                   hashmap.insert(key, n+1);
                   hashmap.len() as u32
               },
               |n| {
                   let key = format!("key for {}", n);
                   let mut hashmap = map_locker.write();
                   hashmap.remove(&key).unwrap_or_default()
               },
               20, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
               n_threads, n_threads, n_threads, n_threads,
               &TimeUnits::MICROSECOND);
eprintln!("ALLOCATION STATS: {}", ALLOC.delta_statistics(&mem_save_point));
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.time_complexity, BigOAlgorithmComplexity::O1, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.time_complexity, BigOAlgorithmComplexity::O1, "DELETE complexity mismatch");
    }
}
