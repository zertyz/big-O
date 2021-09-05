//! Contains functions to perform Algorithm's Space Complexity Analysis.
//!
//! Here we analyze two types of algorithms: the ones that alter the size of the set they operate on (insert/delete) and
//! the ones that don't, so the correct Big-O notation can be given.
//!
//! The complexity analysis is done on the max_memory used -- which includes any auxiliary memory used, but then freed,
//! on an insertion, for example. Since the algorithm is supposed to run several times, the auxiliary memory tends to
//! cause a negligible effect on the analysis, when compared to the non-auxiliary allocations (provided you algorithm is ok).
//!
use crate::big_o_analysis::types::*;

/// Perform space complexity analysis for algorithms that do not alter the set size they operate on.
/// Examples: fib(n), sort(n), bsearch, read, update
///   - O(1) for functions that do not use ram at all (or that use a const amount, regardles of the set size), like uncached reads / updates
///   - O(log(n)) for recursive binary searches and the like
///   - O(n) if caching takes place for reads / updates
pub fn analyse_space_complexity_for_constant_set_algorithm(passes_info:  &ConstantSetAlgorithmPassesInfo,
                                                           measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    // max mem used
    let s1 = (measurements.pass_1_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64);
    let s2 = (measurements.pass_2_measurements.max_used_memory as f64 - measurements.pass_2_measurements.used_memory_before as f64);

    // set size variation
    let n1 = std::cmp::min(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;
    let n2 = std::cmp::max(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;

    let time_complexity: BigOAlgorithmComplexity;

    if ((s1 / s2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        time_complexity = BigOAlgorithmComplexity::BetterThanO1;
    } else if ((s2 / s1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        time_complexity = BigOAlgorithmComplexity::O1;
    } else if ( ((s2 / s1) / ( n2.log2() / n1.log2() )) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n2)/log(n1)) ~= 1
        time_complexity = BigOAlgorithmComplexity::OLogN;
    } else if ( ((s2 / s1) / (n2 / n1)) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / (n2/n1) ~= 1
        time_complexity = BigOAlgorithmComplexity::ON;
    } else if ( ((s2 / s1) / (n2 / n1)) - 1.0_f64 ) > PERCENT_TOLERANCE {
        // check for worse than O(n)
        time_complexity = BigOAlgorithmComplexity::WorseThanON;
    } else {
        // by exclusion...
        time_complexity = BigOAlgorithmComplexity::BetweenOLogNAndON;
    }

    time_complexity
}

/// Perform space complexity analysis for algorithms that alter the set size they operate on.
/// Examples: insert/delete, enqueue/dequeue, push/pop
///   - O(1) will be returned for insertions that allocate 1 constant slot size for each element;
///   - If some filtering is done at insertion time (like in an UpSert or upon inserting only even elements), than Better than O(1) may be returned;
///   - O(log(n)), being worse than O(1), requires that an additional allocation to be done every now and then... like when building a sortable/searchable
///     binary tree for the vector elements -- to allow fast searches while keeping the traversal order (to match insertion order)
///   - O(n) for wired algorithms where each new inserted (n+1) element cause another (n) or even (n+1) elements to be insert as well.
///     Example: increment by 1 both chess board dimensions. board(n) will become board(n+1): 1+2*n elements are added. O(n) memory consumption.
pub fn analyse_space_complexity_for_set_resizing_algorithm(passes_info:  &SetResizingAlgorithmPassesInfo,
                                                           measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    let n = passes_info.delta_set_size as f64;

    // max mem used
    let s1 = (measurements.pass_1_measurements.max_used_memory as f64 - measurements.pass_1_measurements.used_memory_before as f64) / n;
    let s2 = (measurements.pass_2_measurements.max_used_memory as f64 - measurements.pass_2_measurements.used_memory_before as f64) / n;

    let time_complexity: BigOAlgorithmComplexity;

    if ((s1 / s2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        time_complexity = BigOAlgorithmComplexity::BetterThanO1;
    } else if ((s2 / s1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        time_complexity = BigOAlgorithmComplexity::O1;
    } else if ( ((s2 / s1) / ( (n * 3.0_f64).log2() / n.log2() )) - 1.0_f64 ).abs() < PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n*3)/log(n)) ~= 1
        time_complexity = BigOAlgorithmComplexity::OLogN;
    } else if ( ((s2 / s1) / 3.0_f64) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / 3 ~= 1
        time_complexity = BigOAlgorithmComplexity::ON;
    } else if ( ((s2 / s1) / 3.0_f64) - 1.0_f64 ) > PERCENT_TOLERANCE {
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
                       max_used_memory: 1000,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1000,
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
                       max_used_memory: 1000,
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 2000,
                       min_used_memory: 0
                   },
               });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::WorseThanON,
               ConstantSetAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 1000,
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

        // ∑(int)log2(1)..log2((2^n)-1) -- deduced from Gauss's arithmetic progression sum
        let log_sum_bit_values = |n_bits| (1..n_bits).into_iter().fold(0.0, |sum: f32, bit: usize| {
            let bit_value_start: usize = 1<<bit;
            let bit_value_finish = (1<<(bit+1))-1;
            let bit_sum = (bit_value_start+bit_value_finish) * (bit_value_start/2);
            sum + (bit_value_start/2) as f32 * ((bit_sum as f32).log2())
        }).round() as usize;

        assert("Theoretical better than O(1) Insert/Delete", BigOAlgorithmComplexity::BetterThanO1,
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
                       max_used_memory: log_sum_bit_values(10),
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: log_sum_bit_values(11) - log_sum_bit_values(10),
                       min_used_memory: 0
                   },
               });

        assert("Theoretical between O(log(n)) and O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenOLogNAndON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 10000 + log_sum_bit_values(10),
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: 20000 + log_sum_bit_values(11) - log_sum_bit_values(10),
                       min_used_memory: 0
                   },
               });

        assert("Theoretical O(n) Insert/Delete", BigOAlgorithmComplexity::ON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (1 + 1000) * (1000 / 2),
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (1000 + 2000) * (1000 / 2),
                       min_used_memory: 0
                   },
               });

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::WorseThanON,
               SetResizingAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOSpaceMeasurements {
                   pass_1_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (1 + 1000) * (1000 / 2),
                       min_used_memory: 0
                   },
                   pass_2_measurements: BigOSpacePassMeasurements {
                       used_memory_before: 0,
                       used_memory_after: 0,
                       max_used_memory: (2000 + 3000) * (1000 / 2),
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
                        used_memory_before: std::cmp::min(100, pass_2_used_memory),
                        used_memory_after: pass_2_used_memory,
                        max_used_memory: pass_2_used_memory,
                        min_used_memory: std::cmp::min(100, pass_2_used_memory)
                    },
                });
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "Space analysis 'analyse_space_complexity_for_constant_set_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_used_memory of {}", last_complexity, current_complexity, pass_2_used_memory);
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
                        used_memory_before: std::cmp::min(100, pass_2_used_memory),
                        used_memory_after: pass_2_used_memory,
                        max_used_memory: pass_2_used_memory,
                        min_used_memory: std::cmp::min(100, pass_2_used_memory)
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