//! Knows how to run & measure regular, non-iterator algorithms for the purpose of having their complexities analysed.\
// //! See [tests] and `tests/big-o-tests.rs` for examples.

use crate::{
    configs::{OUTPUT},
    low_level_analysis::{
        self,
        types::{
            BigOAlgorithmAnalysis,
            BigOAlgorithmComplexity,
            AlgorithmPassesInfo,
            AlgorithmMeasurements,
            BigOTimeMeasurements,
            BigOSpaceMeasurements,
            TimeUnit},
    },
    runners::common::*
};


/// TODO
pub fn test_constant_set_iterator_algorithm() {}

/// TODO
pub fn test_set_resizing_iterator_algorithm() {}

/// Runs [analyse_algorithm()], trying to match the given maximum time & space complexities to the ones observed in runtime when running the algorithm
/// -- retrying as much as `max_retry_attempts` to avoid flaky test results.\
/// /// In case of rejection, a detailed run log with measurements & analysis results is issued.
/// TODO follow the same design & features as in crud.rs -- specially the retrying & error handling
pub fn test_algorithm<_ScalarDuration: TryInto<u64> + Copy>
                     (test_name:                &str,                    max_retry_attempts:        u32,
                      mut reset_fn:             impl FnMut(),
                      pass1_set_size:           u32,                     pass1_algorithm:           impl FnMut() -> u32,
                      pass2_set_size:           u32,                     pass2_algorithm:           impl FnMut() -> u32,
                      expected_time_complexity: BigOAlgorithmComplexity, expected_space_complexity: BigOAlgorithmComplexity,
                      time_unit:                &TimeUnit<_ScalarDuration>,
                      ) {

    OUTPUT(&format!("Running '{}' algorithm:\n", test_name));
    let (_reset_pass_result,                   r0) = run_pass_verbosely("  Resetting: ", "", || {reset_fn(); 0}, time_unit, OUTPUT);
    let (pass1_result, r1) = run_pass_verbosely("; Pass 1: ", "", pass1_algorithm, time_unit, OUTPUT);
    let (pass2_result, r2) = run_pass_verbosely("; Pass 2: ", "", pass2_algorithm, time_unit, OUTPUT);
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
        panic!("{}", msg);
    }
    if observed_time_complexity as u32 > expected_time_complexity as u32 {
        let msg = format!("\n ** Aborted due to TIME complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", test_name, expected_time_complexity, observed_time_complexity);
        OUTPUT(&msg);
        panic!("{}", msg);
    }

    OUTPUT(&format!("r={}\n\n", r0 ^ r1 ^ r2));
}
