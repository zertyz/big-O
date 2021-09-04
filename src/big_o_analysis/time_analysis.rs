use crate::big_o_analysis::types::*;

/// Performs the algorithm analysis for a reasonably large select/update operation (on a database or not).
/// To perform the analysis, two passes of selects/updates of r elements must be done.
/// On the first pass, the data set must have 'n1' elements and, on the second pass, 'n2' elements -- 'n2' must be (at least) twice 'n1'.
/// 'r' should be reasonably large so that end-start can be accurately measured and account for OS, IO and network latencies.
/// 'start's 1 & 2 and 'end's 1 & 2 are measurement times, regardless of the measurement unit -- milliseconds or microseconds.
/// The returned algorithm complexity is an indication of the time taken to select/update one element on a data set containing
/// 'n' elements, where 'O' is the constant of proportionality -- the average time to select/update 1 element.\
/// Returns: [1] -- the algorithm complexity;\
///          [2] -- a string with the algorithm analysis report.\
pub fn analyse_time_complexity_for_constant_set_algorithm<ScalarTimeUnit: Copy>(passes_info:  &ConstantSetAlgorithmPassesInfo,
                                                                                measurements: &BigOTimeMeasurements<ScalarTimeUnit>) -> BigOAlgorithmComplexity {

    // time variation
    let t1 = measurements.pass_1_measurements.elapsed_time as f64 / passes_info.repetitions as f64;
    let t2 = measurements.pass_2_measurements.elapsed_time as f64 / passes_info.repetitions as f64;

    // set size variation
    let n1 = std::cmp::min(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;
    let n2 = std::cmp::max(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;

    let time_complexity: BigOAlgorithmComplexity;

    if ((t1/t2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        time_complexity = BigOAlgorithmComplexity::BetterThanO1;
    } else if ((t2/t1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        time_complexity = BigOAlgorithmComplexity::O1;
    } else if ( ((t2/t1) / ( n2.log2() / n1.log2() )) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n2)/log(n1)) ~= 1
        time_complexity = BigOAlgorithmComplexity::OLogN;
    } else if ( ((t2/t1) / (n2 / n1)) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / (n2/n1) ~= 1
        time_complexity = BigOAlgorithmComplexity::ON;
    } else if ( ((t2/t1) / (n2 / n1)) - 1.0_f64 ) > PERCENT_TOLERANCE {
        // check for worse than O(n)
        time_complexity = BigOAlgorithmComplexity::WorseThanON;
    } else {
        // by exclusion...
        time_complexity = BigOAlgorithmComplexity::BetweenOLogNAndON;
    }

    time_complexity
}

pub fn analyse_time_complexity_for_set_resizing_algorithm<ScalarTimeUnit: Copy>(passes_info:  &SetResizingAlgorithmPassesInfo,
                                                                                measurements: &BigOTimeMeasurements<ScalarTimeUnit>) -> BigOAlgorithmComplexity {

    let n = passes_info.delta_set_size as f64;

    // time variation
    let t1 = measurements.pass_1_measurements.elapsed_time as f64 / n;
    let t2 = measurements.pass_2_measurements.elapsed_time as f64 / n;

    let time_complexity: BigOAlgorithmComplexity;

    if ((t1/t2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        time_complexity = BigOAlgorithmComplexity::BetterThanO1;
    } else if ((t2/t1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        time_complexity = BigOAlgorithmComplexity::O1;
    } else if ( ((t2/t1) / ( (n * 3.0_f64).log2() / n.log2() )) - 1.0_f64 ).abs() < PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n*3)/log(n)) ~= 1
        time_complexity = BigOAlgorithmComplexity::OLogN;
    } else if ( ((t2/t1) / 3.0_f64) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / 3 ~= 1
        time_complexity = BigOAlgorithmComplexity::ON;
    } else if ( ((t2/t1) / 3.0_f64) - 1.0_f64 ) > PERCENT_TOLERANCE {
        // check for worse than O(n)
        time_complexity = BigOAlgorithmComplexity::WorseThanON;
    } else {
        // by exclusion...
        time_complexity = BigOAlgorithmComplexity::BetweenOLogNAndON;
    }

    time_complexity
}

#[cfg(test)]
mod tests {

    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(cpu)]
    fn analyse_constant_set_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: ConstantSetAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_time_complexity = analyse_time_complexity_for_constant_set_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 - (PERCENT_TOLERANCE*100.0) as u64 },
        });

        assert("Theoretical O(1) Update/Select", BigOAlgorithmComplexity::O1,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
        });

        assert("Theoretical O(log(n)) Update/Select", BigOAlgorithmComplexity::OLogN,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 111 },
        });

        assert("Theoretical between O(log(n)) and O(n) Update/Select", BigOAlgorithmComplexity::BetweenOLogNAndON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2500, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical O(n) Update/Select", BigOAlgorithmComplexity::ON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::WorseThanON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 222 },
        });

    }

    #[test]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: SetResizingAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_complexity = analyse_time_complexity_for_set_resizing_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Insert/Delete", BigOAlgorithmComplexity::BetterThanO1,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 - (PERCENT_TOLERANCE*100.0) as u64 },
        });

        assert("Theoretical O(1) Insert/Delete", BigOAlgorithmComplexity::O1,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
        });

        assert("Theoretical O(log(n)) Insert/Delete", BigOAlgorithmComplexity::OLogN,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 111 },
        });

        assert("Theoretical between O(log(n)) and O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenOLogNAndON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical O(n) Insert/Delete", BigOAlgorithmComplexity::ON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 300 },
        });

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::WorseThanON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 333 },
        });
    }

    #[test]
    #[serial(cpu)]
    fn smooth_transitions() {

        // constant_set
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_total_time in 0..500 {
            let current_complexity = analyse_time_complexity_for_constant_set_algorithm(
                &ConstantSetAlgorithmPassesInfo {
                    pass_1_set_size: 1000,
                    pass_2_set_size: 2000,
                    repetitions: 1000
                },
                &BigOTimeMeasurements {
                    pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                    pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: pass_2_total_time },
            });
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "Time analysis 'analyse_time_complexity_for_constant_set_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_total_time of {}", last_complexity, current_complexity, pass_2_total_time);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_time_complexity_for_constant_set_algorithm(...)' transitioned to {:?} at {}", current_complexity, pass_2_total_time);
            }
        }

        // set_resizing
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_total_time in 0..500 {
            let current_complexity = analyse_time_complexity_for_set_resizing_algorithm(
                &SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
                &BigOTimeMeasurements {
                    pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                    pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: pass_2_total_time },
            });
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "Time analysis 'analyse_time_complexity_for_set_resizing_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_total_time of {}", last_complexity, current_complexity, pass_2_total_time);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_time_complexity_for_set_resizing_algorithm(...)' transitioned to {:?} at {}", current_complexity, pass_2_total_time);
            }
        }
    }

}