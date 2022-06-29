use crate::{
    configs::OUTPUT,
    low_level_analysis::{
        self,
        types::{
            BigOAlgorithmComplexity,
            ConstantSetAlgorithmPassesInfo,
            ConstantSetAlgorithmMeasurements,
            BigOTimeMeasurements,
            BigOSpaceMeasurements,
            TimeUnit},
    },
    runners::common::*
};
use crate::low_level_analysis::types::BigOAlgorithmAnalysis;


pub fn test_constant_set_algorithm<_ScalarDuration: TryInto<u64> + Copy>
                                  (test_name:                &str,                    max_retry_attempts:        u32,
                                   pass1_set_size:           u32,                     pass1_algorithm:           impl FnMut() -> u32,
                                   pass2_set_size:           u32,                     pass2_algorithm:           impl FnMut() -> u32,
                                   expected_time_complexity: BigOAlgorithmComplexity, expected_space_complexity: BigOAlgorithmComplexity,
                                   time_unit:                &TimeUnit<_ScalarDuration>,
                                   reset_fn:                 impl FnMut()) {

    OUTPUT(&format!("Running '{}' algorithm:\n", test_name));
    let (pass1_result, r1) = run_pass_verbosely("  Pass 1: ", "", pass1_algorithm, time_unit, OUTPUT);
    let (pass2_result, r2) = run_pass_verbosely(", Pass 2: ", "", pass2_algorithm, time_unit, OUTPUT);
    let measurements = ConstantSetAlgorithmMeasurements {
        measurement_name: test_name,
        passes_info: ConstantSetAlgorithmPassesInfo {
            pass_1_set_size: pass1_set_size,
            pass_2_set_size: pass2_set_size,
            repetitions: 1,
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
    let observed_time_complexity  = low_level_analysis::time_analysis::  analyse_time_complexity_for_constant_set_algorithm(&measurements.passes_info, &measurements.time_measurements);
    let observed_space_complexity = low_level_analysis::space_analysis::analyse_space_complexity_for_constant_set_algorithm(&measurements.passes_info, &measurements.space_measurements);
    let algorithm_analysis = BigOAlgorithmAnalysis {
        time_complexity: observed_time_complexity,
        space_complexity: observed_space_complexity,
        algorithm_measurements: measurements,
    };

    OUTPUT("\n\n");
    OUTPUT(&format!("{}\n", algorithm_analysis));


    if observed_space_complexity as u32 > expected_space_complexity as u32 {
        OUTPUT(&format!("\n ** Aborted due to SPACE complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", test_name, expected_space_complexity, observed_space_complexity));
    }
    if observed_time_complexity as u32 > expected_time_complexity as u32 {
        OUTPUT(&format!("\n ** Aborted due to TIME complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n", test_name, expected_time_complexity, observed_time_complexity));
    }
    OUTPUT(&format!("r={}\n\n", r1 | r2));
}