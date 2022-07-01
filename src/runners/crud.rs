//! Knows how to run & measure CRUD algorithms in the right sequence for the purpose of having
//! their complexities analysed.\
//! See [tests] and `tests/big-o-tests.rs` for examples.

use crate::{
    configs::{OUTPUT},
    low_level_analysis::{
        self,
        types::{BigOIteratorAlgorithmType, TimeUnit, ConstantSetIteratorAlgorithmMeasurements, SetResizingIteratorAlgorithmMeasurements,
                BigOAlgorithmAnalysis, BigOTimeMeasurements, BigOSpaceMeasurements,
                SetResizingIteratorAlgorithmPassesInfo, ConstantSetIteratorAlgorithmPassesInfo, BigOAlgorithmComplexity},
    },
    runners::common::*,
};
use std::{
    convert::TryInto,
    ops::Range,
    time::{SystemTime},
    io::{self, Write},
    {error::Error, fmt},
    fmt::{Display, Formatter},
    collections::BTreeMap,
};


/// Runs [analyze_crud_algorithms()], trying to match the given maximum time & space complexities to the ones observed in runtime when running the algorithm
/// -- retrying as much as `max_retry_attempts` to avoid flaky test results.\
/// In case of rejection, a detailed run log with measurements & analysis results is issued.
pub fn test_crud_algorithms<'a,
    _ResetClosure:  Fn(u32) -> u32 + Sync,
    _CreateClosure: Fn(u32) -> u32 + Sync,
    _ReadClosure:   Fn(u32) -> u32 + Sync,
    _UpdateClosure: Fn(u32) -> u32 + Sync,
    _DeleteClosure: Fn(u32) -> u32 + Sync,
    T: TryInto<u64> > (crud_name: &'a str, max_retry_attempts: u32,
                       reset_fn:  _ResetClosure,
                       create_fn: _CreateClosure, expected_create_time_complexity: BigOAlgorithmComplexity, expected_create_space_complexity: BigOAlgorithmComplexity,
                       read_fn:   _ReadClosure,     expected_read_time_complexity: BigOAlgorithmComplexity,   expected_read_space_complexity: BigOAlgorithmComplexity,
                       update_fn: _UpdateClosure, expected_update_time_complexity: BigOAlgorithmComplexity, expected_update_space_complexity: BigOAlgorithmComplexity,
                       delete_fn: _DeleteClosure, expected_delete_time_complexity: BigOAlgorithmComplexity, expected_delete_space_complexity: BigOAlgorithmComplexity,
                       warmup_percentage: u32, create_iterations_per_pass: u32, read_iterations_per_pass: u32, update_iterations_per_pass: u32, delete_iterations_per_pass: u32,
                       create_threads: u32, read_threads: u32, update_threads: u32, delete_threads: u32,
                       time_unit: &'a TimeUnit<T>) where PassResult<'a, T>: Copy, T: Copy {

    // adapts the 'iterations_per_pass' to the 'attempt' number, so each retry uses slightly different values
    fn adapt(attempt: u32, iterations_per_pass: u32) -> u32 {
        let factor = 10-(((attempt % 15)/3)*2); // [10,8,6,4,2,10,8,6,4,2,...]
        match attempt {
            0 => iterations_per_pass,
            _ => match (attempt-1) % 3 {
                0 => iterations_per_pass / factor,
                1 => iterations_per_pass - (iterations_per_pass / factor),
                2 => iterations_per_pass + (iterations_per_pass / factor),
                _ => panic!("fix this match")
            }
        }

    }

    let mut collected_errors = Vec::<CRUDComplexityAnalysisError>::with_capacity(max_retry_attempts as usize);

    // in order to reduce false-negatives, retry up to 'max_retry_attempts' if time complexity don't match
    // the maximum acceptable create, read, update or delete 'expected_*_time_complexity'(ies)
    for attempt in 0..max_retry_attempts {

        let adapted_create_iterations_per_pass = adapt(attempt, create_iterations_per_pass);
        let   adapted_read_iterations_per_pass = adapt(attempt, read_iterations_per_pass);
        let adapted_update_iterations_per_pass = adapt(attempt, update_iterations_per_pass);
        let adapted_delete_iterations_per_pass = adapt(attempt, delete_iterations_per_pass);

        let crud_analysis = internal_analyse_crud_algorithms(crud_name, &reset_fn,
                                                             &create_fn,  expected_create_time_complexity, expected_create_space_complexity,
                                                             &read_fn,     expected_read_time_complexity, expected_read_space_complexity,
                                                             &update_fn, expected_update_time_complexity, expected_update_space_complexity,
                                                             &delete_fn, expected_delete_time_complexity, expected_delete_space_complexity,
                                                             warmup_percentage, adapted_create_iterations_per_pass, adapted_read_iterations_per_pass, adapted_update_iterations_per_pass, adapted_delete_iterations_per_pass,
                                                             create_threads, read_threads, update_threads, delete_threads,
                                                             time_unit);

        // In case of error, retry only if the complexity analysis failed to match the maximum requirement for Time,
        // which can be affected by run-time environment conditions (specially if the involved machines aren't fully idle
        // or on low RAM conditions, causing swap or page faults to kick in).
        // Space complexity analysis is always deterministic, regardless of the environment conditions.
        if crud_analysis.is_err() {
            let crud_analysis_error = crud_analysis.err().unwrap();
            if crud_analysis_error.failed_analysis == "Time" {
                if attempt < max_retry_attempts-1 {
                    collected_errors.push(crud_analysis_error);
                    OUTPUT(&format!("\nAttempt {} failed. Resetting before retrying", attempt+1));
                    reset_fn(100);  // 100% of the created elements
                    OUTPUT("...\n");
                    continue;
                } else {
                    let unique_failed_operations_count = collected_errors.iter()
                        .rfold(BTreeMap::<String, u32>::new(), |mut acc, collected_error| {
                            let key = format!("{} with {:?}", collected_error.failed_operation, collected_error.failed_complexity);
                            let op_count = acc.get_mut(&key);
                            match op_count {
                                Some(count) => *count += 1,
                                None => {
                                    acc.insert(key, 1);
                                },
                            };
                            acc
                        });
                    let previous_errors = unique_failed_operations_count.iter()
                        .rfold(String::new(), |mut acc, failed_operation_count_item| {
                            let operation = failed_operation_count_item.0;
                            let count = failed_operation_count_item.1;
                            acc.push_str(&format!(" - {} ({} time{})\n", operation, count, if *count == 1 {""} else {"s"}));
                            acc
                        });
                    panic!("After {} attempts, gave up retrying: {}.\n\
                            Previous attempts failed at:\n\
                            {}", max_retry_attempts, crud_analysis_error, previous_errors);
                }
            } else {
                // mismatched space complexity (if not on the first loop, reset_fn probably didn't deallocated)
                panic!("At attempt #{}, SPACE complexity mismatch: {}\n", attempt+1, crud_analysis_error);
            }
        }
        break;
    }
}

/// Runs time & space analysis for Create, Read, Update and Delete algorithms -- usually from a container or database.
/// Returns the Optional analysis for each operation + the full report, in textual form.
/// An analysis will be None if the provided '*_iterations_per_pass' or '*_threads' are 0.\
/// --> This function is not meant to be run in tests -- see [test_crud_algorithms()] instead.
///   - `reset_fn` -- a closure or function that will be called after warming up, to restore the empty
///                   state of the container and to deallocate any memory allocated during the warmup pass
///                   (which only runs if `warmup_percentage` > 0)
///   - `create_fn`, `read_fn`, `update_fn` & `delete_fn` -- closures or functions for each of the
///                                                          CRUD operations
///   - --> note for the functions above: they have the following signature 'fn (n: u32) -> u32', where
///         'n' is the number of the element to be operated on (for reset, the number of created
///         elements is given); all of them should return an 'u32' dependent on the execution of the
///         algorithm to avoid any 'call removal optimizations'
///   - `warmup_percentage` -- [0..100]: if > 0, causes an warmup pass to be executed before the first
///                            and second passes, to hot load caches, resolve page faults, establish
///                            network connections or do any other operations that might impact the
///                            time complexity analysis. Note, however, that the [reset_fn] must
///                            also deallocate any allocated memory so the space complexity analysis
///                            is not compromised.
///   - `create_iterations_per_pass`, `read_iterations_per_pass`, `update_iterations_per_pass` &
///     `delete_iterations_per_pass` -- number of times each CRUD algorithm should run, per pass -- not
///                                     too small (any involved IO/OS times should be negligible) nor too
///                                     big (so the analysis won't take up much time nor resources)
///   - `create_threads`, `read_threads`, `update_threads`, `delete_threads` -- specifies how many threads
///     should be recruited for each CRUD operation. Each thread is guaranteed to call their algorithm's
///     closures (see the '*_fn' parameters) within a continuous range
///   - `time_unit` -- specifies the time unit to use to measure & present time results. Notice the measured
///                    numbers are integers, so the unit should be at least one or two orders of magnitude
///                    broader than the measured values. Space measurements are always in bytes and their
///                    presentation unit (b, KiB, MiB or GiB) are automatically selected.
pub fn analyse_crud_algorithms<'a,
                               _ResetClosure:  Fn(u32) -> u32 + Sync,
                               _CreateClosure: Fn(u32) -> u32 + Sync,
                               _ReadClosure:   Fn(u32) -> u32 + Sync,
                               _UpdateClosure: Fn(u32) -> u32 + Sync,
                               _DeleteClosure: Fn(u32) -> u32 + Sync,
                               T: TryInto<u64> > (crud_name: &'a str,
                                                  reset_fn:  _ResetClosure,
                                                  create_fn: _CreateClosure,
                                                  read_fn:   _ReadClosure,
                                                  update_fn: _UpdateClosure,
                                                  delete_fn: _DeleteClosure,
                                                  warmup_percentage: u32, create_iterations_per_pass: u32, read_iterations_per_pass: u32, update_iterations_per_pass: u32, delete_iterations_per_pass: u32,
                                                  create_threads: u32, read_threads: u32, update_threads: u32, delete_threads: u32,
                                                  time_unit: &'a TimeUnit<T>)
        -> (Option< BigOAlgorithmAnalysis<SetResizingIteratorAlgorithmMeasurements<'a,T>> >,    // create analysis
            Option< BigOAlgorithmAnalysis<ConstantSetIteratorAlgorithmMeasurements<'a,T>> >,    // read analysis
            Option< BigOAlgorithmAnalysis<ConstantSetIteratorAlgorithmMeasurements<'a,T>> >,    // update analysis
            Option< BigOAlgorithmAnalysis<SetResizingIteratorAlgorithmMeasurements<'a,T>> >,    // delete analysis
            String) where PassResult<'a, T>: Copy, T: Copy {

    internal_analyse_crud_algorithms(crud_name, reset_fn,
                                     create_fn,  BigOAlgorithmComplexity::WorseThanExponential,  BigOAlgorithmComplexity::WorseThanExponential,
                                     read_fn,     BigOAlgorithmComplexity::WorseThanExponential,   BigOAlgorithmComplexity::WorseThanExponential,
                                     update_fn, BigOAlgorithmComplexity::WorseThanExponential,  BigOAlgorithmComplexity::WorseThanExponential,
                                     delete_fn,  BigOAlgorithmComplexity::WorseThanExponential,  BigOAlgorithmComplexity::WorseThanExponential,
                                     warmup_percentage, create_iterations_per_pass, read_iterations_per_pass, update_iterations_per_pass, delete_iterations_per_pass,
                                     create_threads, read_threads, update_threads, delete_threads,
                                     time_unit).unwrap()
}

#[derive(Debug)]
struct CRUDComplexityAnalysisError {
    pub failed_operation:     String,
    pub failed_analysis:      String,
    pub failed_complexity:    BigOAlgorithmComplexity,
    pub failed_assertion_msg: String,
    #[allow(dead_code)]
    pub partial_report:       String,
}
impl Display for CRUDComplexityAnalysisError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CRUD complexity analysis assertion failed: {}", self.failed_assertion_msg)
    }
}
impl Error for CRUDComplexityAnalysisError {}

/// Returns the analysed complexities + the full report, as a string in the form (create, read, update, delete, report).
/// If one of the measured complexities don't match the maximum expected, None is returned for that analysis, provided it's *_number_of_iterations_per_pass is > 0.
fn internal_analyse_crud_algorithms<'a,
                              _ResetClosure:  Fn(u32) -> u32 + Sync,
                              _CreateClosure: Fn(u32) -> u32 + Sync,
                              _ReadClosure:   Fn(u32) -> u32 + Sync,
                              _UpdateClosure: Fn(u32) -> u32 + Sync,
                              _DeleteClosure: Fn(u32) -> u32 + Sync,
                              T: TryInto<u64> > (crud_name: &'a str,
                                                 reset_fn:  _ResetClosure,
                                                 create_fn: _CreateClosure, expected_create_time_complexity: BigOAlgorithmComplexity, expected_create_space_complexity: BigOAlgorithmComplexity,
                                                 read_fn:   _ReadClosure,     expected_read_time_complexity: BigOAlgorithmComplexity,   expected_read_space_complexity: BigOAlgorithmComplexity,
                                                 update_fn: _UpdateClosure, expected_update_time_complexity: BigOAlgorithmComplexity, expected_update_space_complexity: BigOAlgorithmComplexity,
                                                 delete_fn: _DeleteClosure, expected_delete_time_complexity: BigOAlgorithmComplexity, expected_delete_space_complexity: BigOAlgorithmComplexity,
                                                 warmup_percentage: u32, create_iterations_per_pass: u32, read_iterations_per_pass: u32, update_iterations_per_pass: u32, delete_iterations_per_pass: u32,
                                                 create_threads: u32, read_threads: u32, update_threads: u32, delete_threads: u32,
                                                 time_unit: &'a TimeUnit<T>)
        -> Result<(Option< BigOAlgorithmAnalysis<SetResizingIteratorAlgorithmMeasurements<'a,T>> >,       // create analysis
                   Option< BigOAlgorithmAnalysis<ConstantSetIteratorAlgorithmMeasurements<'a,T>> >,       // read analysis
                   Option< BigOAlgorithmAnalysis<ConstantSetIteratorAlgorithmMeasurements<'a,T>> >,       // update analysis
                   Option< BigOAlgorithmAnalysis<SetResizingIteratorAlgorithmMeasurements<'a,T>> >,       // delete analysis
                   String), CRUDComplexityAnalysisError> where PassResult<'a, T>: Copy, T: Copy { // full report

    let mut full_report = String::with_capacity(2048);

    // wrap around the original 'OUTPUT' function to capture the [full_report]
    let mut _output = |msg: &str| {
        full_report.push_str(msg);
        OUTPUT(msg);
    };

    let mut create_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut   read_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut update_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];
    let mut delete_passes_results = [PassResult::<T>::default(); NUMBER_OF_PASSES as usize];

    const NUMBER_OF_PASSES: u32 = 2;

    // accumulation of computed results from [create_fn], [read_fn], [update_fn] and [delete_fn]
    // to avoid any call cancellation optimizations when running in release mode
    let mut r: u32 = 0;

    // range calculation
    fn calc_regular_cru_range(iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * pass_number       .. iterations_per_pass * (pass_number + 1) }
    fn calc_regular_d_range(iterations_per_pass: u32, pass_number: u32) -> Range<u32> { iterations_per_pass * (pass_number + 1) .. iterations_per_pass * pass_number }

    /// Contains factored out code to measure & analyse READ or UPDATE operations, checking the expected maximum time & space complexities
    ///   - [pass_number] -- u32 in the range [0..NUMBER_OF_PASSES]: specifies the number of the pass being run
    ///   - [operation_name] -- &str: either "Read" or "Update"
    ///   - [suffix] -- &str: ", " or "" -- used to correctly separate intermediate results
    ///   - [passes_results] -- either [read_passes_results] or [update_passes_results]: the array to put [PassResults] on
    ///   - [algorithm_closure] -- the algorithm closure to run -- either [read_fn] or [update_fn]
    ///   - [expected_time_complexity] & [expected_space_complexity] -- the maximum expected complexities (cause the method
    ///     to return in error if the expectations are not met)
    ///   - [number_of_iterations_per_pass] -- u32: either [read_iterations_per_pass] or [update_iterations_per_pass]
    ///   - [number_of_threads] -- u32: either [read_threads] or [update_threads]
    macro_rules! run_constant_set_pass {
        ($pass_number: expr, $operation_name: literal, $suffix: expr, $passes_results: ident,
         $algorithm_closure: ident, $expected_time_complexity: ident, $expected_space_complexity: ident,
         $number_of_iterations_per_pass: expr, $number_of_threads: ident) => {
            if $number_of_iterations_per_pass > 0 {
                let (pass_result, pass_r) = run_iterator_pass_verbosely(&format!("{}: ", $operation_name.to_ascii_lowercase()), $suffix,
                                                                        &$algorithm_closure, &BigOIteratorAlgorithmType::SetResizing,
                                                                        calc_regular_cru_range($number_of_iterations_per_pass, $pass_number),
                                                                        time_unit, $number_of_threads, &mut _output);
                $passes_results[$pass_number as usize] = pass_result;
                r ^= pass_r;
                if $pass_number == NUMBER_OF_PASSES-1 {
                    let measurements = ConstantSetIteratorAlgorithmMeasurements {
                        measurement_name: $operation_name,
                        passes_info: ConstantSetIteratorAlgorithmPassesInfo {
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
                    let  time_complexity = low_level_analysis::time_analysis::  analyse_time_complexity_for_constant_set_iterator_algorithm(&measurements.passes_info, &measurements.time_measurements);
                    let space_complexity = low_level_analysis::space_analysis::analyse_space_complexity_for_constant_set_iterator_algorithm(&measurements.passes_info, &measurements.space_measurements);
                    yield_analysis_or_return_with_error!($operation_name, measurements, $expected_time_complexity, $expected_space_complexity, time_complexity, space_complexity)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// Contains factored out code to measure & analyse CREATE or DELETE operations, checking the expected maximum time & space complexities
    ///   - [pass_number] -- u32 in the range [0..NUMBER_OF_PASSES]: specifies the number of the pass being run
    ///   - [operation_name] -- &str: either "Read" or "Update"
    ///   - [suffix] -- &str: ", " or "" -- used to correctly separate intermediate results
    ///   - [result_prefix_closure] -- fn (pass_number, operation_name) -> String: the prefix for [run_pass_verbosely] to show
    ///     the intermediate measurements -- should return "operation_name: " for create; "2nd", "1st; " for delete;
    ///   - [passes_results] -- either [create_passes_results] or [delete_passes_results]: the array to put [PassResults] on
    ///   - [algorithm_closure] -- the algorithm closure to run -- either [create_fn] or [delete_fn]
    ///   - [expected_time_complexity] & [expected_space_complexity] -- the maximum expected complexities (cause the method
    ///     to return in error if the expectations are not met)
    ///   - [number_of_iterations_per_pass] -- u32: either [create_iterations_per_pass] or [delete_iterations_per_pass]
    ///   - [number_of_threads] -- u32: either [create_threads] or [delete_threads]
    macro_rules! run_set_resizing_pass {
        ($pass_number: expr, $operation_name: literal, $suffix: ident, $result_prefix_closure: expr,
         $passes_results: ident, $range_fn: ident, $last_pass_number: expr,
         $algorithm_closure: ident, $expected_time_complexity: ident, $expected_space_complexity: ident,
         $number_of_iterations_per_pass: expr, $number_of_threads: ident) => {
            if $number_of_iterations_per_pass > 0 {
                let (pass_result, pass_r) = run_iterator_pass_verbosely(&$result_prefix_closure($pass_number, $operation_name), $suffix,
                                                                        &$algorithm_closure, &BigOIteratorAlgorithmType::SetResizing,
                                                                        $range_fn($number_of_iterations_per_pass, $pass_number),
                                                                        time_unit, $number_of_threads, &mut _output);
                $passes_results[$pass_number as usize] = pass_result;
                r ^= pass_r;
                if $pass_number == $last_pass_number {
                    let measurements = SetResizingIteratorAlgorithmMeasurements {
                        measurement_name: $operation_name,
                        passes_info: SetResizingIteratorAlgorithmPassesInfo {
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
                    let  time_complexity = low_level_analysis::time_analysis::  analyse_time_complexity_for_set_resizing_iterator_algorithm(&measurements.passes_info, &measurements.time_measurements);
                    let space_complexity = low_level_analysis::space_analysis::analyse_space_complexity_for_set_resizing_iterator_algorithm(&measurements.passes_info, &measurements.space_measurements);
                    yield_analysis_or_return_with_error!($operation_name, measurements, $expected_time_complexity, $expected_space_complexity, time_complexity, space_complexity)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// factored out code from [run_constant_set_pass!()] and [run_set_resizing_pass!()] --
    /// returns the [BigOAlgorithmAnalysis] or return the method with the error message
    macro_rules! yield_analysis_or_return_with_error {
        ($operation_name: literal, $measurements: ident,
         $expected_time_complexity: ident, $expected_space_complexity: ident,
         $observed_time_complexity: ident, $observed_space_complexity: ident) => {
            if $observed_time_complexity as u32 > $expected_time_complexity as u32 {
                _output(&format!(" ** Aborted due to TIME complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", $operation_name, $expected_time_complexity, $observed_time_complexity));
                return Err(CRUDComplexityAnalysisError {
                    failed_operation:     $operation_name.to_string(),
                    failed_analysis:      "Time".to_owned(),
                    failed_complexity:    $observed_time_complexity,
                    failed_assertion_msg: format!("'{}' algorithm was expected to match a maximum TIME complexity of '{:?}', but '{:?}' was measured", $operation_name, $expected_time_complexity, $observed_time_complexity),
                    partial_report:       full_report,
                });
            } else if $observed_space_complexity as u32 > $expected_space_complexity as u32 {
                _output(&format!(" ** Aborted due to SPACE complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", $operation_name, $expected_space_complexity, $observed_space_complexity));
                return Err(CRUDComplexityAnalysisError {
                    failed_operation:     $operation_name.to_string(),
                    failed_analysis:      "Space".to_owned(),
                    failed_complexity:    $observed_space_complexity,
                    failed_assertion_msg: format!("'{}' algorithm was expected to match a maximum SPACE complexity of '{:?}', but '{:?}' was measured", $operation_name, $expected_space_complexity, $observed_space_complexity),
                    partial_report:       full_report,
                });
            } else {
                Some(BigOAlgorithmAnalysis {
                    algorithm_measurements: $measurements,
                    $observed_time_complexity,
                    $observed_space_complexity,
                })
            }
        }
    }

    macro_rules! run_create_pass {
        ($pass_number: expr) => {{
            let suffix = if read_iterations_per_pass > 0 || update_iterations_per_pass > 0 {", "} else {""};
            run_set_resizing_pass!($pass_number, "Create", suffix, |_pass_number: u32, pass_name: &str| format!("{}: ", pass_name.to_ascii_lowercase()),
                                   create_passes_results, calc_regular_cru_range, NUMBER_OF_PASSES-1,
                                   create_fn, expected_create_time_complexity, expected_create_space_complexity,
                                   create_iterations_per_pass, create_threads)
        }}
    }
    macro_rules! run_read_pass {
        ($pass_number: expr) => {{
            let suffix = if update_iterations_per_pass > 0 {", "} else {""};
            run_constant_set_pass!($pass_number, "Read", suffix, read_passes_results, read_fn, expected_read_time_complexity, expected_read_space_complexity, read_iterations_per_pass, read_threads)
        }}
    }
    macro_rules! run_update_pass {
        ($pass_number: expr) => {{
            let suffix = "";
            run_constant_set_pass!($pass_number, "Update", suffix, update_passes_results, update_fn, expected_update_time_complexity, expected_update_space_complexity, update_iterations_per_pass, update_threads)
        }}
    }
    macro_rules! run_delete_pass {
        ($pass_number: expr) => {{
            let suffix = "";
            run_set_resizing_pass!($pass_number, "Delete", suffix,
                                   |pass_number: u32, _pass_name: &str|
                                       if pass_number == NUMBER_OF_PASSES-1 {
                                         "2nd: "
                                       } else {
                                         "; 1st: "
                                       },
                                   delete_passes_results, calc_regular_d_range, 0,
                                   delete_fn, expected_delete_time_complexity, expected_delete_space_complexity,
                                   delete_iterations_per_pass, delete_threads)
        }}
    }


    _output(&format!("{} CRUD Algorithm Complexity Analysis:\n  ", crud_name));

    // warmup
    if warmup_percentage > 0 {

        // warmup ranges
        let calc_warmup_cru_range = |iterations_per_pass|  0 .. iterations_per_pass * warmup_percentage / 100;
        let calc_warmup_d_range = |iterations_per_pass| iterations_per_pass * warmup_percentage / 100 .. 0;

        let warmup_start = SystemTime::now();
        _output("warming up [");
        io::stdout().flush().unwrap();
        if create_iterations_per_pass > 0 {
            _output(&"C");
            let (_elapse, warmup_r) = run_iterator_pass(&create_fn, &BigOIteratorAlgorithmType::SetResizing, calc_warmup_cru_range(create_iterations_per_pass), time_unit, create_threads);
            r ^= warmup_r;
        }
        if read_iterations_per_pass > 0 {
            _output(&"R");
            let (_elapse, warmup_r) = run_iterator_pass(&read_fn, &BigOIteratorAlgorithmType::ConstantSet, calc_warmup_cru_range(read_iterations_per_pass), time_unit, read_threads);
            r ^= warmup_r;
        }
        if update_iterations_per_pass > 0 {
            _output(&"U");
            let (_elapse, warmup_r) = run_iterator_pass(&update_fn, &BigOIteratorAlgorithmType::ConstantSet, calc_warmup_cru_range(update_iterations_per_pass), time_unit, update_threads);
            r ^= warmup_r;
        }
        if delete_iterations_per_pass > 0 {
            _output(&"D");
            let (_elapse, warmup_r) = run_iterator_pass(&delete_fn, &BigOIteratorAlgorithmType::SetResizing, calc_warmup_d_range(delete_iterations_per_pass), time_unit, delete_threads);
            r ^= warmup_r;
        }
        _output("] ");
        reset_fn(create_iterations_per_pass * warmup_percentage / 100);

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

    _output("):\n\n");

    // output analysis reports
    if create_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", create_analysis.as_ref().unwrap()));
    }
    if read_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", read_analysis.as_ref().unwrap()));
    }
    if update_iterations_per_pass > 0 {
        _output(&format!("{}\n\n", update_analysis.as_ref().unwrap()));
    }

    // delete passes (passes are applied in reverse order)
    let delete_analysis;
    if delete_iterations_per_pass > 0 {
        _output("Delete Passes (");
        run_delete_pass!(1);
        delete_analysis = run_delete_pass!(0);

        _output(&format!(") r={}:\n", r));

        // output the analysis report
        _output(&format!("{}\n\n", delete_analysis.as_ref().unwrap()));
    } else {
        delete_analysis = None;
    }

    Ok( (create_analysis, read_analysis, update_analysis, delete_analysis, full_report) )
}


#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [crud_analysis](super) module -- using 'serial_test' crate in order to make time measurements more reliable.

    use super::*;
    use crate:: {
        low_level_analysis::types::{TimeUnits, BigOAlgorithmMeasurements},
    };
    use std::{
        collections::HashMap,
        sync::atomic::{Ordering, AtomicU32},
    };
    use serial_test::serial;

    /// Attests that the right report structures are produced for all possible CRUD tests:
    ///   - progress is reported per pass, per operation (operation = create, read, update or delete)
    ///   - sub-reports are only created when 'iterations_per_pass' for the operation is > 0
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial]                                  // needed since considerable RAM is used (which may interfere with 'crud_analysis.rs' tests)
    fn analyse_crud_algorithm_output_check() {
        let iterations_per_pass = 100000;

        // high level asserting functions
        /////////////////////////////////

        fn assert_contains_algorithm_report<T: BigOAlgorithmMeasurements>(report: &str, algorithm_analysis: Option<BigOAlgorithmAnalysis<T>>, algorithm_name: &str) {
            assert!(report.contains(&algorithm_analysis.unwrap().to_string()), "couldn't find '{}' report analysis on the full report", algorithm_name);
        }
        fn assert_passes_progress(report: &str, warmup: bool, create: bool, read: bool, update: bool, delete: bool) {
            if warmup {
                let warmup_announcement = format!("warming up [{}{}{}{}] ",
                                                  if create {"C"} else {""},
                                                  if read   {"R"} else {""},
                                                  if update {"U"} else {""},
                                                  if delete {"D"} else {""});
                assert!(report.contains(&warmup_announcement), "'Warmup' announcement was not properly issued -- no '{}' announcement was found on the full report", warmup_announcement);
                assert!(report.contains(", First Pass ("),     "'warming up' & 'First Pass' announcements seem not to be in sync -- they used to be separated by a comma when the former one is present");
            } else {
                assert!(!report.contains("warming up "), "'Warmup' announcement was present on the full report, even when it wasn't requested");
            }
            if create {
                let first_pass_announcement = "First Pass (create: ";
                let second_pass_announcement = "Second Pass (create: ";
                assert!(report.contains(first_pass_announcement), "'First Pass' announcement of the 'create' step seems wrong");
                assert!(report.contains(second_pass_announcement), "'Second Pass' announcement of the 'create' step seems wrong");
            } else {
                assert!(!report.contains("create: "), "'create' step announcement was present on the full report, even when that step wasn't requested");
            }
            if read {
                let first_pass_announcement  = if create {"b, read: "} else {"First Pass (read: "};
                let second_pass_announcement = if create {"b, read: "} else {"Second Pass (read: "};
                assert!(report.contains(first_pass_announcement), "'First Pass' announcement of the 'read' step seems wrong when 'create' is {}present", if create {""} else {"not "});
                assert!(report.contains(second_pass_announcement), "'Second Pass' announcement of the 'read' step seems wrong when 'create' is {}present", if create {""} else {"not "});
            } else {
                assert!(!report.contains("read: "), "'read' step announcement was present on the full report, even when that step wasn't requested");
            }
            if update {
                let first_pass_announcement  = if create || read {"b, update: "} else {"First Pass (update: "};
                let second_pass_announcement = if create || read {"b, update: "} else {"Second Pass (update: "};
                assert!(report.contains(first_pass_announcement), "'First Pass' announcement of the 'update' step seems wrong when 'create'/'read' are {}present", if create || read {""} else {"not "});
                assert!(report.contains(second_pass_announcement), "'Second Pass' announcement of the 'update' step seems wrong when 'create'/'read' are {}present", if create || read {""} else {"not "});
            } else {
                assert!(!report.contains("update: "), "'update' step announcement was present on the full report, even when that step wasn't requested");
            }
            let delete_passes_announcement = "Delete Passes (";
            if delete {
                assert!(report.contains(delete_passes_announcement), "'Delete' announcement was not properly issued -- no '{}' announcement was found on the full report", delete_passes_announcement);
            } else {
                assert!(!report.contains(delete_passes_announcement), "'Delete' announcement was present on the full report, even when it wasn't requested");
            }
            assert!(!report.contains(",); Second Pass ("), "comma handling at the end of the 'First Pass' announcement is wrong");
            assert!(!report.contains(" ); Second Pass ("), "space handling at end end of the 'First Pass' announcement is wrong");
            assert!(report.contains("b):\n"), "comma / space handling at the end of the 'Second Pass' announcement seems wrong");
        }

        // checks
        /////////

        // fully fledged output
        let (create_analysis,
             read_analysis,
             update_analysis,
             delete_analysis,
             report) = analyse_crud_algorithms("MyContainer",
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
        assert_passes_progress(&report, true, true, true, true, true);
        assert_contains_algorithm_report(&report, create_analysis, "Create");
        assert_contains_algorithm_report(&report, read_analysis, "Read");
        assert_contains_algorithm_report(&report, update_analysis, "Update");
        assert_contains_algorithm_report(&report, delete_analysis, "Delete");

        // no warmup
        let (_create_analysis,
             _read_analysis,
             _update_analysis,
             _delete_analysis,
             report) = analyse_crud_algorithms("MyContainer",
                                               |_n| panic!("'reset_fn' should not be called if there is no warmup taking place"),
                                               |n| (n+1)/(n+1),
                                               |n| (n+1)/(n+1),
                                               |n| (n+1)/(n+1),
                                               |n| (n+1)/(n+1),
                                               0/* no warmup */, iterations_per_pass, iterations_per_pass, iterations_per_pass, iterations_per_pass,
                                               1, 1, 1, 1,
                                               &TimeUnits::NANOSECOND);
        assert_passes_progress(&report, false, true, true, true, true);

        // no delete as well
        let (_create_analysis,
            _read_analysis,
            _update_analysis,
            delete_analysis,
            report) = analyse_crud_algorithms("MyContainer",
                                              |_n| panic!("'reset_fn' should not be called if there is no warmup taking place"),
                                              |n| (n+1)/(n+1),
                                              &|n| (n+1)/(n+1),
                                              |n| (n+1)/(n+1),
                                              |_n| panic!("'delete_fn' should not be called if there is no warmup taking place"),
                                              0/*no warmup*/, iterations_per_pass, iterations_per_pass, iterations_per_pass, 0,
                                              1, 1, 1, 0,
                                              &TimeUnits::NANOSECOND);
        assert_passes_progress(&report, false, true, true, true, false);
        assert!(delete_analysis.is_none(), "No Delete Complexity Analysis should have been made");

        // just create
        let (_create_analysis,
            _read_analysis,
            _update_analysis,
            _delete_analysis,
            report) = analyse_crud_algorithms("MyContainer",
                                              |_n| panic!("'reset_fn' should not be called if there is no warmup taking place"),
                                              |n| (n+1)/(n+1),
                                              &|_n| panic!("'read_fn' should not be called if there is no warmup taking place"),
                                              |_n| panic!("'update_fn' should not be called if there is no warmup taking place"),
                                              |_n| panic!("'delete_fn' should not be called if there is no warmup taking place"),
                                              0/*no warmup*/, iterations_per_pass, 0, 0, 0,
                                              1, 1, 1, 1,
                                              &TimeUnits::NANOSECOND);
        assert_passes_progress(&report, false, true, false, false, false);
    }

    /// Attests the same number of iterations are produced regardless of the number of threads:
    ///   - 'iterations_per_pass must' be a multiple of 'n_threads'
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial]
    fn thread_chunk_division() {
        let iterations_per_pass = 1000;
        for n_threads in [1,2,4,5,10] {
            let map_locker = parking_lot::RwLock::new(HashMap::<u32, u32>::with_capacity(2 * iterations_per_pass as usize));
            let max_length = AtomicU32::new(0);
            analyse_crud_algorithms("thread_chunk_division",
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
                   assert_eq!(map.remove(&n), Some(n), "missing element #{} when deleting for n_threads {}", n, n_threads);
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
}
