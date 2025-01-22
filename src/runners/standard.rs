//! Knows how to run & measure regular, non-iterator algorithms for the purpose of having their complexities analysed.\
// //! See [tests] and `tests/big-o-tests.rs` for examples.

use std::time::Duration;
use keen_retry::{loggable_retry_errors, ResolvedResult, RetryProducerResult, RetryResult};
use crate::{
    features::{OUTPUT},
    low_level_analysis::{
        self,
        types::{
            BigOAlgorithmAnalysis,
            BigOAlgorithmComplexity,
            AlgorithmPassesInfo,
            AlgorithmMeasurements,
            BigOTimeMeasurements,
            BigOSpaceMeasurements,
        },
    },
    runners::common::*
};
use crate::low_level_analysis::types::BigOPassMeasurements;

/// TODO
pub fn test_constant_set_iterator_algorithm() {}

/// TODO
pub fn test_set_resizing_iterator_algorithm() {}

/// Runs [analyse_algorithm()], trying to match the given maximum time & space complexities to the ones observed in runtime when running the algorithm
/// -- retrying as much as `max_retry_attempts` to avoid flaky test results.\
/// /// In case of rejection, a detailed run log with measurements & analysis results is issued.
pub fn test_algorithm(test_name:                 &str,
                      max_retry_attempts:        u32,
                      mut reset_fn:              impl FnMut(),
                      pass1_set_size:            u32,
                      mut pass1_algorithm:       impl FnMut() -> u32,
                      pass2_set_size:            u32,
                      mut pass2_algorithm:       impl FnMut() -> u32,
                      expected_time_complexity:  BigOAlgorithmComplexity,
                      expected_space_complexity: BigOAlgorithmComplexity) {
    let result = analyse_algorithm(test_name, &mut reset_fn, pass1_set_size, &mut pass1_algorithm, pass2_set_size, &mut pass2_algorithm, expected_time_complexity, expected_space_complexity)
        .retry_with(|_| analyse_algorithm(test_name, &mut reset_fn, pass1_set_size, &mut pass1_algorithm, pass2_set_size, &mut pass2_algorithm, expected_time_complexity, expected_space_complexity))
        .with_delays((0..max_retry_attempts).map(|_| Duration::from_secs(5)));
    let failure_msg = match result {
        ResolvedResult::Ok { .. } => None,
        ResolvedResult::Fatal { error, .. } => Some(error),
        ResolvedResult::Recovered { .. } => None,
        ResolvedResult::GivenUp { retry_errors, fatal_error, .. } => Some(format!("Given up with '{}' after {max_retry_attempts} attempts. Previous transient errors: {}", fatal_error, loggable_retry_errors(&retry_errors))),
        ResolvedResult::Unrecoverable { retry_errors, fatal_error, .. } => Some(format!("Stopped after retrying for {max_retry_attempts} attempts due to the fatal outcome '{}'. Previous transient errors: {}", fatal_error, loggable_retry_errors(&retry_errors))),
    };
    if let Some(failure_msg) = failure_msg {
        panic!("{}", failure_msg);
    }
}

/// Internal version of [test_algorithm()], allowing retries
fn analyse_algorithm(test_name:                 &str,
                     reset_fn:                  &mut impl FnMut(),
                     pass1_set_size:            u32,
                     pass1_algorithm:           &mut impl FnMut() -> u32,
                     pass2_set_size:            u32,
                     pass2_algorithm:           &mut impl FnMut() -> u32,
                     expected_time_complexity:  BigOAlgorithmComplexity,
                     expected_space_complexity: BigOAlgorithmComplexity)
                    -> RetryProducerResult<String, String> {

    OUTPUT(&format!("Running '{}' algorithm:\n", test_name));
    let (_reset_pass_result,                   r0) = run_sync_pass_verbosely("  Resetting: ", "", || {reset_fn(); 0}, OUTPUT);
    let (pass1_result, r1) = run_sync_pass_verbosely("; Pass 1: ", "", pass1_algorithm, OUTPUT);
    let (pass2_result, r2) = run_sync_pass_verbosely("; Pass 2: ", "", pass2_algorithm, OUTPUT);
    let measurements = AlgorithmMeasurements {
        measurement_name: test_name,
        passes_info: AlgorithmPassesInfo {
            pass1_n: pass1_set_size,
            pass2_n: pass2_set_size,
        },
        time_measurements: BigOTimeMeasurements {
            pass_1_measurements: pass1_result.time_measurements,
            pass_2_measurements: pass2_result.time_measurements,
        },
        space_measurements: BigOSpaceMeasurements {
            pass_1_measurements: pass1_result.space_measurements,
            pass_2_measurements: pass2_result.space_measurements,
        },
        pass1_measurements: BigOPassMeasurements {
            time_measurements: pass1_result.time_measurements,
            space_measurements: Default::default(),
            custom_measurements: vec![],
        },
        pass2_measurements: BigOPassMeasurements {
            time_measurements: Default::default(),
            space_measurements: Default::default(),
            custom_measurements: vec![],
        },
    };
    let observed_time_complexity  = low_level_analysis::time_analysis::analyse_time_complexity(&measurements.passes_info, &measurements.time_measurements);
    let observed_space_complexity = low_level_analysis::space_analysis::analyse_space_complexity(&measurements.passes_info, &measurements.space_measurements);
    let algorithm_analysis = BigOAlgorithmAnalysis {
        time_complexity: observed_time_complexity,
        space_complexity: observed_space_complexity,
        algorithm_measurements: measurements,
    };

    OUTPUT("\n\n");
    OUTPUT(&format!("{}\n", algorithm_analysis));


    if observed_space_complexity as u32 > expected_space_complexity as u32 {
        let msg = format!("\n ** Aborted due to SPACE complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", test_name, expected_space_complexity, observed_space_complexity);
        OUTPUT(&msg);
        RetryResult::Fatal { input: (), error: msg }
    } else if observed_time_complexity as u32 > expected_time_complexity as u32 {
        let msg = format!("\n ** TIME complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?} -- a reattempt may be performed...\n\n", test_name, expected_time_complexity, observed_time_complexity);
        OUTPUT(&msg);
        RetryResult::Transient { input: (), error: msg }
    } else {
        let msg = format!("r={}\n\n", r0 ^ r1 ^ r2);
        OUTPUT(&msg);
        RetryResult::Ok { reported_input: (), output: msg }
    }

}
