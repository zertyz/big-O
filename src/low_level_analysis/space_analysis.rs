//! Contains functions to perform Algorithm's Space Complexity Analysis.
//!
//! The complexity analysis is done on the `max_memory` used -- which includes any *auxiliary memory* used, where
//! *auxiliary memory* means allocations that were done, but then freed before the algorithm finishes,

use crate::low_level_analysis::{
    analyse_complexity,
    analyse_set_resizing_iterator_complexity,
    types::*,
};


/// Performs space complexity analysis for regular, non-iterator algorithms, such as `fib(n)`, `sort(n)`, `bsearch(e, n)`, ...
pub fn analyse_space_complexity(passes_info:  &AlgorithmPassesInfo,
                                measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    // max mem usage
    let s1 = (measurements.pass_1_measurements.max_used_memory - measurements.pass_1_measurements.used_memory_before) as f64;
    let s2 = (measurements.pass_2_measurements.max_used_memory - measurements.pass_2_measurements.used_memory_before) as f64;

    // set sizes
    let n1 = passes_info.pass1_n as f64;
    let n2 = passes_info.pass2_n as f64;

    analyse_complexity(s1, s2, n1, n2)
}

/// Perform space complexity analysis for iterator algorithms that do not alter the size of the set they operate on or for
/// non-iterator algorithms (even if they are growing/shrinking a data set from top to zero),
/// where iterator algorithms are the ones that operates on a single element (of a huge set) per call.\
/// Examples: fib(n), sort(n), bsearch, read, update
///   - O(1) for functions that do not use ram at all (or that use a const amount, regardless of the set size), like uncached reads / updates
///   - O(log(n)) for recursive binary searches and the like
///   - O(n) if caching takes place for reads / updates
/// See [analyse_space_complexity_for_set_resizing_iterator_algorithm()] for iterator algorithms that resize the data set they operate on.
pub fn analyse_space_complexity_for_constant_set_iterator_algorithm(passes_info:  &ConstantSetIteratorAlgorithmPassesInfo,
                                                                    measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    // max mem usage
    let s1 = (measurements.pass_1_measurements.max_used_memory - measurements.pass_1_measurements.used_memory_before) as f64;
    let s2 = (measurements.pass_2_measurements.max_used_memory - measurements.pass_2_measurements.used_memory_before) as f64;

    // set sizes
    let n1 = std::cmp::min(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;
    let n2 = std::cmp::max(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;

    analyse_complexity(s1, s2, n1, n2)
}

/// Perform space complexity analysis for iterator algorithms that alter the set size they operate on,
/// where iterator algorithms are the ones that adds/consumes one element (to/from a huge set) per call.\
/// Examples: insert/delete, enqueue/dequeue, push/pop
///   - O(1) will be returned for insertions that allocate a constant slot size for each element, regardless of the number of elements;
///   - If some filtering is done at insertion time (like in an UpSert or upon inserting only even elements), than Better than O(1) may be returned;
///   - O(log(n)), being worse than O(1), requires that an additional allocation to be done every now and then... like when building a sortable/searchable
///     binary tree for the vector elements -- to allow fast searches while keeping the traversal order (to match insertion order)
///   - O(n) for wired algorithms where each new inserted (n+1) element cause another (n) or even (n+1) elements to be insert as well.
///     Example: increment by 1 both chess board dimensions. board(n) will become board(n+1): 1+2*n elements are added. O(n) memory consumption.
/// See [analyse_space_complexity_for_constant_set_iterator_algorithm()] for non-iterator algorithms and for iterator algorithms that operate on a constant set.
pub fn analyse_space_complexity_for_set_resizing_iterator_algorithm(passes_info:  &SetResizingIteratorAlgorithmPassesInfo,
                                                                    measurements: &BigOSpaceMeasurements) -> BigOAlgorithmComplexity {

    let n = passes_info.delta_set_size as f64;

    // max mem used
    let s1 = (measurements.pass_1_measurements.max_used_memory - measurements.pass_1_measurements.used_memory_before) as f64;
    let s2 = (measurements.pass_2_measurements.max_used_memory - measurements.pass_2_measurements.used_memory_before) as f64;

    analyse_set_resizing_iterator_complexity(s1, s2, n)
}

#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [space_analysis](super) module

    use super::*;
    use serial_test::serial;


    /// test the space complexity analysis results based on some known-to-be-correct measurement sizes
    #[test]
    #[serial]
    fn analyse_constant_set_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: ConstantSetIteratorAlgorithmPassesInfo, space_measurements: BigOSpaceMeasurements| {
            let observed_time_complexity = analyse_space_complexity_for_constant_set_iterator_algorithm(&passes_info, &space_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
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
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
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
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
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
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2600, repetitions: 1000 },
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
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
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

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::ONLogN,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
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
                       max_used_memory: 2400,
                       min_used_memory: 0
                   },
               });

    }

    /// test the space complexity analysis results based on some known-to-be-correct measurement sizes
    #[test]
    #[serial]
    fn analyse_set_resizing_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: SetResizingIteratorAlgorithmPassesInfo, space_measurements: BigOSpaceMeasurements| {
            let observed_complexity = analyse_space_complexity_for_set_resizing_iterator_algorithm(&passes_info, &space_measurements);
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
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenONAndONLogN,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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

}