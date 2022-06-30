//! Contains functions to perform Algorithm's Time Complexity Analysis.

use crate::low_level_analysis::{
    analyze_complexity,
    analyze_set_resizing_iterator_complexity,
    types::*,
    configs::*,
};


/// Performs time complexity analysis for regular, non-iterator algorithms, such as `fib(n)`, `sort(n)`, `bsearch(e, n)`, ...
pub fn analyse_time_complexity<ScalarTimeUnit: Copy>(passes_info:  &AlgorithmPassesInfo,
                                                     measurements: &BigOTimeMeasurements<ScalarTimeUnit>) -> BigOAlgorithmComplexity {

    // time variation
    let t1 = measurements.pass_1_measurements.elapsed_time as f64;
    let t2 = measurements.pass_2_measurements.elapsed_time as f64;

    // set sizes
    let n1 = passes_info.pass1_n as f64;
    let n2 = passes_info.pass2_n as f64;

    analyze_complexity(t1, t2, n1, n2)
}

/// Performs time complexity analysis (based on the 2 passes & measurements given), for an iterator algorithm that does not alter
/// the size of the set they operate on or for non-iterator algorithms (even if they are growing/shrinking a data set from top to
/// zero), where iterator algorithms are the ones that operates on a single element (of a huge set) per call.\
/// Examples: select/update, get, sort, fib...
///
/// To perform the analysis, two passes are required on different set sizes -- see [passes_info]. The number of repetitions & set size
/// must be carefully chosen in order to generate elapsed times (on each pass) high enough to make OS, IO and network latencies
/// negligible -- if the operation is CPU bounded, the machine should be idle.
///
/// The returned algorithm complexity -- in big-O notation -- is an asymptotic indication of the time needed to execute the algorithm
/// on one element, in proportion to a set size of 'n' elements. See [BigOAlgorithmComplexity].
pub fn analyse_time_complexity_for_constant_set_iterator_algorithm<ScalarTimeUnit: Copy>(passes_info:  &ConstantSetIteratorAlgorithmPassesInfo,
                                                                                         measurements: &BigOTimeMeasurements<ScalarTimeUnit>) -> BigOAlgorithmComplexity {

    // time variation
    let t1 = measurements.pass_1_measurements.elapsed_time as f64;
    let t2 = measurements.pass_2_measurements.elapsed_time as f64;

    // set sizes
    let n1 = std::cmp::min(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;
    let n2 = std::cmp::max(passes_info.pass_1_set_size, passes_info.pass_2_set_size) as f64;

    analyze_complexity(t1, t2, n1, n2)
}

/// Performs time complexity analysis (based on the 2 passes & measurements given), for iterator algorithms
/// that alter the size of they operate on, where iterator algorithms are the ones that adds/consumes
/// one element (to/from a huge set) per call.\
/// Examples: insert/delete, push/pop, enqueue/dequeue, add/remove and so on.
///
/// To perform the analysis, two passes are required with the same delta in elements count on each -- see [passes_info].
/// The number of executions must be carefully chosen in order to generate elapsed times (on each pass) high enough to
/// make OS, IO and network latencies negligible -- if the operation is CPU bounded, the machine should be idle.
///
/// The returned algorithm complexity -- in big-O notation -- is an asymptotic indication of the time needed to execute the algorithm
/// on one element, in proportion to a set size of 'n' elements. See [BigOAlgorithmComplexity].
pub fn analyse_time_complexity_for_set_resizing_iterator_algorithm<ScalarTimeUnit: Copy>(passes_info:  &SetResizingIteratorAlgorithmPassesInfo,
                                                                                         measurements: &BigOTimeMeasurements<ScalarTimeUnit>) -> BigOAlgorithmComplexity {

    let n = passes_info.delta_set_size as f64;

    // time variation
    let t1 = measurements.pass_1_measurements.elapsed_time as f64;
    let t2 = measurements.pass_2_measurements.elapsed_time as f64;

    analyze_set_resizing_iterator_complexity(t1, t2, n)
}

#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [time_analysis](super) module

    use super::*;
    use serial_test::serial;


    /// test the time complexity analysis results based on some known-to-be-correct measurement times
    #[test]
    #[serial]
    fn analyse_constant_set_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: ConstantSetIteratorAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_time_complexity = analyse_time_complexity_for_constant_set_iterator_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 - (PERCENT_TOLERANCE*100.0) as u64 },
        });

        assert("Theoretical O(1) Update/Select", BigOAlgorithmComplexity::O1,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
        });

        assert("Theoretical O(log(n)) Update/Select", BigOAlgorithmComplexity::OLogN,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 111 },
        });

        assert("Theoretical between O(log(n)) and O(n) Update/Select", BigOAlgorithmComplexity::BetweenOLogNAndON,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2500, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical O(n) Update/Select", BigOAlgorithmComplexity::ON,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::WorseThanON,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 222 },
        });

    }

    /// test the time complexity analysis results based on some known-to-be-correct measurement times
    #[test]
    #[serial]
    fn analyse_set_resizing_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: SetResizingIteratorAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_complexity = analyse_time_complexity_for_set_resizing_iterator_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Insert/Delete", BigOAlgorithmComplexity::BetterThanO1,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 - (PERCENT_TOLERANCE*100.0) as u64 },
        });

        assert("Theoretical O(1) Insert/Delete", BigOAlgorithmComplexity::O1,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
        });

        assert("Theoretical O(log(n)) Insert/Delete", BigOAlgorithmComplexity::OLogN,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 111 },
        });

        assert("Theoretical between O(log(n)) and O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenOLogNAndON,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical O(n) Insert/Delete", BigOAlgorithmComplexity::ON,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 300 },
        });

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::WorseThanON,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 333 },
        });
    }

    /// test the time complexity analysis results progression when measurements increase
    #[test]
    #[serial]
    fn smooth_transitions() {

        // constant_set
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_total_time in 0..500 {
            let current_complexity = analyse_time_complexity_for_constant_set_iterator_algorithm(
                &ConstantSetIteratorAlgorithmPassesInfo {
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
            let current_complexity = analyse_time_complexity_for_set_resizing_iterator_algorithm(
                &SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
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