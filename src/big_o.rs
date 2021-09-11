use crate::{
    conditionals::{self,OUTPUT},
    big_o_analysis::{
        self, run_pass, PassResult, BigOAlgorithmType,
        types::{BigOAlgorithmAnalysis, TimeUnit, TimeUnits, ConstantSetAlgorithmMeasurements, SetResizingAlgorithmMeasurements,
                BigOTimeMeasurements, BigOSpaceMeasurements, BigOSpacePassMeasurements, BigOTimePassMeasurements,
                SetResizingAlgorithmPassesInfo, ConstantSetAlgorithmPassesInfo},
    }
};

use std::convert::TryInto;
use std::ops::Range;
use std::time::{SystemTime, Duration};
use std::io;
use std::io::Write;
use crate::big_o_analysis::types::BigOAlgorithmComplexity;
use crate::big_o_analysis::types::BigOAlgorithmComplexity::BetterThanO1;


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
                                                 time_unit: &'a TimeUnit<T>)
        -> (Option< BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a,T>> >,    // create analysis
            Option< BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a,T>> >,    // read analysis
            Option< BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a,T>> >,    // update analysis
            Option< BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a,T>> >,    // delete analysis
            String) where PassResult<'a, T>: Copy, T: Copy {                            // full report

    test_crud_algorithm(crud_name, reset_fn,
                        create_fn,  BigOAlgorithmComplexity::WorseThanON, BigOAlgorithmComplexity::WorseThanON,
                        read_fn,     BigOAlgorithmComplexity::WorseThanON,  BigOAlgorithmComplexity::WorseThanON,
                        update_fn, BigOAlgorithmComplexity::WorseThanON,BigOAlgorithmComplexity::WorseThanON,
                        delete_fn, BigOAlgorithmComplexity::WorseThanON,BigOAlgorithmComplexity::WorseThanON,
                        warmup_percentage, create_iterations_per_pass, read_iterations_per_pass, update_iterations_per_pass, delete_iterations_per_pass,
                        create_threads, read_threads, update_threads, delete_threads,
                        time_unit)
}

/// Returns the analyzed complexities + the full report, as a string in the form (create, read, update, delete, report).
/// If one of the measured complexities don't match the minimum expected, None is returned for that analysis, provided it's *_number_of_iterations_per_pass is > 0.
pub fn test_crud_algorithm<'a,
                              _ResetClosure:  Fn(u32) -> u32 + Sync,
                              _CreateClosure: Fn(u32) -> u32 + Sync,
                              _ReadClusure:   Fn(u32) -> u32 + Sync,
                              _UpdateClosure: Fn(u32) -> u32 + Sync,
                              _DeleteClosure: Fn(u32) -> u32 + Sync,
                              T: TryInto<u64> > (crud_name: &'a str,
                                                 reset_fn:  _ResetClosure,
                                                 create_fn: _CreateClosure, expected_create_time_complexity: BigOAlgorithmComplexity, expected_create_space_complexity: BigOAlgorithmComplexity,
                                                 read_fn:   _ReadClusure,     expected_read_time_complexity: BigOAlgorithmComplexity,   expected_read_space_complexity: BigOAlgorithmComplexity,
                                                 update_fn: _UpdateClosure, expected_update_time_complexity: BigOAlgorithmComplexity, expected_update_space_complexity: BigOAlgorithmComplexity,
                                                 delete_fn: _DeleteClosure, expected_delete_time_complexity: BigOAlgorithmComplexity, expected_delete_space_complexity: BigOAlgorithmComplexity,
                                                 warmup_percentage: u32, create_iterations_per_pass: u32, read_iterations_per_pass: u32, update_iterations_per_pass: u32, delete_iterations_per_pass: u32,
                                                 create_threads: u32, read_threads: u32, update_threads: u32, delete_threads: u32,
                                                 time_unit: &'a TimeUnit<T>)
        -> (Option< BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a,T>> >,    // create analysis
            Option< BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a,T>> >,    // read analysis
            Option< BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements<'a,T>> >,    // update analysis
            Option< BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements<'a,T>> >,    // delete analysis
            String) where PassResult<'a, T>: Copy, T: Copy {                            // full report

    let mut full_report = String::new();

    // wrap around the original 'OUTPUT' function to capture the [full_report]
    let mut _output = |msg: &str| {
        full_report.push_str(msg);
        OUTPUT(msg);
    };

    /// wrap around the original 'run_pass' to output intermediate results
    fn run_pass_verbosely<'a,
                          _AlgorithmClosure: Fn(u32) -> u32 + Sync,
                          _OutputClosure:    FnMut(&str),
                          T: TryInto<u64> + Copy> (result_prefix: &str, result_suffix: &str,
                                                   algorithm: &_AlgorithmClosure, algorithm_type: &BigOAlgorithmType, range: Range<u32>, time_unit: &'a TimeUnit<T>,
                                                   threads: u32, mut _output: _OutputClosure)
                                                   -> (PassResult<'a,T>, u32) {
        let (pass_result, r) = run_pass(algorithm, algorithm_type, range, time_unit, threads);
        _output(&format!("{}{}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
        (pass_result, r)
    }

    let mut create_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut   read_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut update_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut delete_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];

    const NUMBER_OF_PASSES: u32 = 2;

    // computed results to avoid any call cancellation optimizations when running in release mode
    let mut r: u32 = 0;
    let mut cr: u32 = 0;
    let mut rr: u32 = 0;
    let mut ur: u32 = 0;
    let mut dr: u32 = 0;

    // range calculation
    fn calc_regular_cru_range(iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * pass_number       .. iterations_per_pass * (pass_number + 1) }
    fn calc_regular_d_range(iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * (pass_number + 1) .. iterations_per_pass * pass_number }

    macro_rules! run_constant_set_pass {
        ($pass_number: expr, $pass_name: literal, $suffix: literal, $passes_results: ident,
         $algorithm_closure: ident, $expected_time_complexity: ident, $expected_space_complexity: ident,
         $number_of_iterations_per_pass: expr, $number_of_threads: ident) => {
            if $number_of_iterations_per_pass > 0 {
                let (pass_result, pass_r) = run_pass_verbosely(&format!("{}: ", $pass_name.to_ascii_lowercase()), $suffix,
                                                               &$algorithm_closure, &BigOAlgorithmType::SetResizing,
                                                               calc_regular_cru_range($number_of_iterations_per_pass, $pass_number),
                                                               time_unit, $number_of_threads, &mut _output);
                $passes_results[$pass_number as usize] = pass_result;
                r ^= pass_r;
                if $pass_number == NUMBER_OF_PASSES-1 {
                    let measurements = ConstantSetAlgorithmMeasurements {
                        measurement_name: $pass_name,
                        passes_info: ConstantSetAlgorithmPassesInfo {
                            pass_1_set_size: create_iterations_per_pass,
                            pass_2_set_size: create_iterations_per_pass * 2,
                            repetitions: $number_of_iterations_per_pass,
                        },
                        time_measurements: BigOTimeMeasurements {
                            pass_1_measurements: $passes_results[0].time_measurements,
                            pass_2_measurements: $passes_results[1].time_measurements,
                        },
                        space_measurements: BigOSpaceMeasurements {
                            pass_1_measurements: $passes_results[0].space_measurements,
                            pass_2_measurements: $passes_results[1].space_measurements,
                        },
                    };
                    let  time_complexity = big_o_analysis::time_analysis::  analyse_time_complexity_for_constant_set_algorithm(&measurements.passes_info, &measurements.time_measurements);
                    let space_complexity = big_o_analysis::space_analysis::analyse_space_complexity_for_constant_set_algorithm(&measurements.passes_info, &measurements.space_measurements);
                    if time_complexity as u32 > $expected_time_complexity as u32 {
                        panic!("'{}' algorithm was expected to match a minimum TIME complexity of '{:?}', but '{:?}' was measured", $pass_name, $expected_time_complexity, time_complexity)
                    } else if space_complexity as u32 > $expected_space_complexity as u32 {
                        panic!("'{}' algorithm was expected to match a minimum SPACE complexity of '{:?}', but '{:?}' was measured", $pass_name, $expected_space_complexity, space_complexity)
                    } else {
                        Some(BigOAlgorithmAnalysis {
                            algorithm_measurements: measurements,
                            time_complexity,
                            space_complexity,
                        })
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    macro_rules! run_set_resizing_pass {
        ($pass_number: expr, $pass_name: literal, $suffix: literal, $result_prefix_closure: expr,
         $passes_results: ident, $range_fn: ident, $last_pass_number: expr,
         $algorithm_closure: ident, $expected_time_complexity: ident, $expected_space_complexity: ident,
         $number_of_iterations_per_pass: expr, $number_of_threads: ident) => {
            if $number_of_iterations_per_pass > 0 {
                let (pass_result, pass_r) = run_pass_verbosely(&$result_prefix_closure($pass_number, $pass_name), $suffix,
                                                               &$algorithm_closure, &BigOAlgorithmType::SetResizing,
                                                               $range_fn($number_of_iterations_per_pass, $pass_number),
                                                               time_unit, $number_of_threads, &mut _output);
                $passes_results[$pass_number as usize] = pass_result;
                r ^= pass_r;
                if $pass_number == $last_pass_number {
                    let measurements = SetResizingAlgorithmMeasurements {
                        measurement_name: $pass_name,
                        passes_info: SetResizingAlgorithmPassesInfo {
                            delta_set_size: $number_of_iterations_per_pass,
                        },
                        time_measurements: BigOTimeMeasurements {
                            pass_1_measurements: $passes_results[0].time_measurements,
                            pass_2_measurements: $passes_results[1].time_measurements,
                        },
                        space_measurements: BigOSpaceMeasurements {
                            pass_1_measurements: $passes_results[0].space_measurements,
                            pass_2_measurements: $passes_results[1].space_measurements,
                        },
                    };
                    let  time_complexity = big_o_analysis::time_analysis::  analyse_time_complexity_for_set_resizing_algorithm(&measurements.passes_info, &measurements.time_measurements);
                    let space_complexity = big_o_analysis::space_analysis::analyse_space_complexity_for_set_resizing_algorithm(&measurements.passes_info, &measurements.space_measurements);
                    if time_complexity as u32 > $expected_time_complexity as u32 {
                        panic!("'{}' algorithm was expected to match a minimum TIME complexity of '{:?}', but '{:?}' was measured", $pass_name, $expected_time_complexity, time_complexity)
                    } else if space_complexity as u32 > $expected_space_complexity as u32 {
                        panic!("'{}' algorithm was expected to match a minimum SPACE complexity of '{:?}', but '{:?}' was measured", $pass_name, $expected_space_complexity, space_complexity)
                    } else {
                        Some(BigOAlgorithmAnalysis {
                            algorithm_measurements: measurements,
                            time_complexity,
                            space_complexity,
                        })
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    macro_rules! run_create_pass {
        ($pass_number: expr) => {
            run_set_resizing_pass!($pass_number, "Create", ", ", |pass_number: u32, pass_name: &str| format!("{}: ", pass_name.to_ascii_lowercase()),
                                   create_passes_results, calc_regular_cru_range, NUMBER_OF_PASSES-1,
                                   create_fn, expected_create_time_complexity, expected_create_space_complexity,
                                   create_iterations_per_pass, create_threads)
        }
    }
    macro_rules! run_read_pass {
        ($pass_number: expr) => {
            run_constant_set_pass!($pass_number, "Read", ", ", read_passes_results, read_fn, expected_read_time_complexity, expected_read_space_complexity, read_iterations_per_pass, read_threads)
        }
    }
    macro_rules! run_update_pass {
        ($pass_number: expr) => {
            run_constant_set_pass!($pass_number, "Update", "", update_passes_results, update_fn, expected_update_time_complexity, expected_update_space_complexity, update_iterations_per_pass, update_threads)
        }
    }
    macro_rules! run_delete_pass {
        ($pass_number: expr) => {
            run_set_resizing_pass!($pass_number, "Delete", "",
                                   |pass_number: u32, pass_name: &str|
                                       if pass_number == NUMBER_OF_PASSES-1 {
                                         "2nd: "
                                       } else {
                                         "; 1st: "
                                       },
                                   delete_passes_results, calc_regular_d_range, 0,
                                   delete_fn, expected_delete_time_complexity, expected_delete_space_complexity,
                                   delete_iterations_per_pass, delete_threads)
        }
    }


    _output(&format!("{} CRUD Algorithm Complexity Analysis:\n  ", crud_name));

    // warmup
    if warmup_percentage > 0 {

        let calc_warmup_cru_range = |iterations_per_pass|  0 .. iterations_per_pass * warmup_percentage / 100;
        let calc_warmup_d_range = |iterations_per_pass| iterations_per_pass * warmup_percentage / 100 .. 0;

        let warmup_start = SystemTime::now();
        _output("warming up [");
        io::stdout().flush().unwrap();
        if create_iterations_per_pass > 0 {
            _output(&"C");
            let (_elapse, warmup_r) = run_pass(&create_fn, &BigOAlgorithmType::SetResizing, calc_warmup_cru_range(create_iterations_per_pass), time_unit, create_threads);
            r ^= warmup_r;
        }
        if read_iterations_per_pass > 0 {
            _output(&"R");
            let (_elapse, warmup_r) = run_pass(&read_fn, &BigOAlgorithmType::ConstantSet, calc_warmup_cru_range(read_iterations_per_pass), time_unit, read_threads);
            r ^= warmup_r;
        }
        if update_iterations_per_pass > 0 {
            _output(&"U");
            let (_elapse, warmup_r) = run_pass(&update_fn, &BigOAlgorithmType::ConstantSet, calc_warmup_cru_range(update_iterations_per_pass), time_unit, update_threads);
            r ^= warmup_r;
        }
        if delete_iterations_per_pass > 0 {
            _output(&"D");
            let (_elapse, warmup_r) = run_pass(&delete_fn, &BigOAlgorithmType::SetResizing, calc_warmup_d_range(delete_iterations_per_pass), time_unit, delete_threads);
            r ^= warmup_r;
        }
        _output("] ");
        reset_fn(warmup_percentage);

        let warmup_end = SystemTime::now();
        let warmup_duration = warmup_end.duration_since(warmup_start).unwrap();
        let warmup_elapsed = (time_unit.duration_conversion_fn_ptr)(&warmup_duration).try_into().unwrap_or_default();
        _output(&format!("{}{}, ", warmup_elapsed, time_unit.unit_str));
    }

    _output("First Pass (");
    run_create_pass!(0);
    run_read_pass!(0);
    run_update_pass!(0);

    _output("); Second Pass (");
    let create_analysis = run_create_pass!(1);
    let read_analysis = run_read_pass!(1);
    let update_analysis = run_update_pass!(1);

    // closes the intermediate pass results report
    _output("):\n\n");

    // output "create", "read" and "update" reports
    if create_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", create_analysis.as_ref().unwrap()));
    }
    if read_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", read_analysis.as_ref().unwrap()));
    }
    if update_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", update_analysis.as_ref().unwrap()));
    }

    // delete passes (applied in reverse order)
    let delete_analysis;
    if (delete_iterations_per_pass > 0) {
        _output("Delete Passes (");
        run_delete_pass!(1);
        delete_analysis = run_delete_pass!(0);

        _output(&format!(") r={}:\n", r));

        // output the "delete" report
        _output(&format!("{}\n\n", delete_analysis.as_ref().unwrap()));
    } else {
        delete_analysis = None;
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


#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;
    use std::sync::Arc;
    use std::collections::HashMap;
    use crate::big_o_analysis::types::{BigOAlgorithmMeasurements, BigOAlgorithmComplexity};
    use crate::conditionals::ALLOC;
    use std::sync::atomic::{Ordering, AtomicU32};

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
        fn assert_contains_algorithm_report<T: BigOAlgorithmMeasurements>(report: &str, algorithm_analysis: Option<BigOAlgorithmAnalysis<T>>, algorithm_name: &str) {
            assert!(report.contains(&algorithm_analysis.unwrap().to_string()), "couldn't find '{}' report analysis on the full report", algorithm_name);
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
        assert!(delete_analysis.is_none(), "No Delete Complexity Analysis should have been made");
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
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "DELETE complexity mismatch");
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
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::ON, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::ON, "DELETE complexity mismatch");

    }

    /// Attests O(1) performance characteristics for HashMaps
    #[test]
    #[serial(cpu)]
    fn hashmap_algorithm_analysis() {
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
        let (create_analysis, read_analysis, update_analysis, delete_analysis, _full_report) = crud_analysis;
        assert_complexity!(create_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "CREATE complexity mismatch");
        assert_complexity!(  read_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1,   "READ complexity mismatch");
        assert_complexity!(update_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "UPDATE complexity mismatch");
        assert_complexity!(delete_analysis.unwrap().time_complexity, BigOAlgorithmComplexity::O1, "DELETE complexity mismatch");
    }
}
