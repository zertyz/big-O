use crate::big_o_analysis::types::*;

pub fn analyse_space_complexity_for_constant_set_algorithm(passes_info:  &ConstantSetAlgorithmPassesInfo,
                                                           measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    // max mem used
    let t1 = (measurements.pass_1_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64) / passes_info.repetitions as f64;
    let t2 = (measurements.pass_2_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64) / passes_info.repetitions as f64;

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

pub fn analyse_space_complexity_for_set_resizing_algorithm(passes_info:  &SetResizingAlgorithmPassesInfo,
                                                           measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    let n = passes_info.delta_set_size as f64;

    // time variation
    let t1 = (measurements.pass_1_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64) / n;
    let t2 = (measurements.pass_2_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64) / n;

    let time_complexity: BigOAlgorithmComplexity;

    if ((t1/t2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        time_complexity = BigOAlgorithmComplexity::BetterThanO1;
    } else if ((t2/t1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        time_complexity = BigOAlgorithmComplexity::O1;
    } else if ( ((t2/t1) / ( (n*2.0).log2() / n.log2() )) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n2)/log(n1)) ~= 1
        time_complexity = BigOAlgorithmComplexity::OLogN;
    } else if ( ((t2/t1) / ((n*2.0) / n)) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / (n2/n1) ~= 1
        time_complexity = BigOAlgorithmComplexity::ON;
    } else if ( ((t2/t1) / ((n*2.0) / n)) - 1.0_f64 ) > PERCENT_TOLERANCE {
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

        let assert = |measurement_name, expected_complexity, passes_info: ConstantSetAlgorithmPassesInfo, space_measurements: BigOSpaceMeasurements| {
            let observed_time_complexity = analyse_space_complexity_for_constant_set_algorithm(&passes_info, &space_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 1024,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 0,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(1) Update/Select", BigOAlgorithmComplexity::O1,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 100,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 100,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(log(n)) Update/Select", BigOAlgorithmComplexity::OLogN,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (1000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (2000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical between O(log(n)) and O(n) Update/Select", BigOAlgorithmComplexity::BetweenOLogNAndON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2500, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1000 / (1000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 2000 / (2000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(n) Update/Select", BigOAlgorithmComplexity::ON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 2048,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::WorseThanON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 3800,
                       min_used_memory: 0
                   },
               });

    }

    #[test]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: SetResizingAlgorithmPassesInfo, space_measurements: BigOSpaceMeasurements| {
            let observed_complexity = analyse_space_complexity_for_set_resizing_algorithm(&passes_info, &space_measurements);
            assert_eq!(observed_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Insert/Delete", BigOAlgorithmComplexity::BetterThanO1,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 1024,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 0,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(1) Insert/Delete", BigOAlgorithmComplexity::O1,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(log(n)) Insert/Delete", BigOAlgorithmComplexity::OLogN,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (1000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (2000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical between O(log(n)) and O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenOLogNAndON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1000 / (1000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 2000 / (2000 as f32).ln() as usize,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(n) Insert/Delete", BigOAlgorithmComplexity::ON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 2048,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::WorseThanON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1024,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 3800,
                       min_used_memory: 0
                   },
               });
    }

    #[test]
    #[serial(cpu)]
    fn smooth_transitions() {

        // constant_set
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_used_memory in 0..500 {
            let current_complexity = analyse_space_complexity_for_constant_set_algorithm(
                &ConstantSetAlgorithmPassesInfo {
                    pass_1_set_size: 1000,
                    pass_2_set_size: 2000,
                    repetitions: 1000
                },
                &BigOSpaceMeasurements {
                    pass_1_measurements: BigOSpacePassMeasurements {
                        used_memory_before: 0,
                        used_memory_after: 100,
                        max_used_memory: 100,
                        min_used_memory: 0
                    },
                    pass_2_measurements: BigOSpacePassMeasurements {
                        used_memory_before: 100,
                        used_memory_after: pass_2_used_memory,
                        max_used_memory: pass_2_used_memory,
                        min_used_memory: 100
                    },
                });
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "Space analysis 'analyse_space_complexity_for_constant_set_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_total_time of {}", last_complexity, current_complexity, pass_2_used_memory);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_space_complexity_for_constant_set_algorithm(...)' transitioned to {:?} at {}", current_complexity, pass_2_used_memory);
            }
        }

        // set_resizing
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_used_memory in 0..500 {
            let current_complexity = analyse_space_complexity_for_set_resizing_algorithm(
                &SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
                &BigOSpaceMeasurements {
                    pass_1_measurements: BigOSpacePassMeasurements {
                        used_memory_before: 0,
                        used_memory_after: 100,
                        max_used_memory: 100,
                        min_used_memory: 0
                    },
                    pass_2_measurements: BigOSpacePassMeasurements {
                        used_memory_before: 100,
                        used_memory_after: pass_2_used_memory,
                        max_used_memory: pass_2_used_memory,
                        min_used_memory: 100
                    },
                });
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "Space analysis 'analyse_space_complexity_for_set_resizing_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_used_memory of {}", last_complexity, current_complexity, pass_2_used_memory);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_space_complexity_for_set_resizing_algorithm(...)' transitioned to {:?} at {}", current_complexity, pass_2_used_memory);
            }
        }
    }

}