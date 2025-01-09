//! Contains functions to perform Algorithm's Time Complexity Analysis.

use crate::low_level_analysis::{
    analyse_complexity,
    analyse_set_resizing_iterator_complexity,
    types::*,
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

    analyse_complexity(t1, t2, n1, n2)
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

    analyse_complexity(t1, t2, n1, n2)
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

    analyse_set_resizing_iterator_complexity(t1, t2, n)
}

#[cfg(test)]
mod tests {

    //! Unit tests for [time_analysis](super) module

    use super::*;
    use crate::features::*;
    use serial_test::serial;


    /// tests the time complexity analysis results based on some known-to-be-correct measurement times
    #[test]
    #[serial]
    fn analyse_algorithm_theoretical_test() {
        let assert = |measurement_name, expected_complexity, passes_info: AlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_time_complexity = analyse_time_complexity(&passes_info, &time_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on regular, non-iterator algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) algorithm", BigOAlgorithmComplexity::BetterThanO1,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 89,  time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(1) algorithm", BigOAlgorithmComplexity::O1,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(log(n)) algorithm", BigOAlgorithmComplexity::OLogN,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 111, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical between O(log(n)) and O(n) algorithm", BigOAlgorithmComplexity::BetweenOLogNAndON,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 150, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(n) algorithm", BigOAlgorithmComplexity::ON,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 100, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 200, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(n.log(n)) algorithm", BigOAlgorithmComplexity::ONLogN,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1000, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 2220, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical between O(n.log(n)) and O(n²) algorithm", BigOAlgorithmComplexity::BetweenONLogNAndON2,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1000, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 3000, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(n²) algorithm", BigOAlgorithmComplexity::ON2,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1000, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 4000, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(n³) algorithm", BigOAlgorithmComplexity::ON3,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1000, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 8000, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(n^4) algorithm", BigOAlgorithmComplexity::ON4,
               AlgorithmPassesInfo { pass1_n: 1000, pass2_n: 2000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time:  1000, time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 16000, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Theoretical O(k^n) algorithm", BigOAlgorithmComplexity::OkN,
               AlgorithmPassesInfo { pass1_n: 10, pass2_n: 70 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1.0e1 as u64,                   time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: 1.0e7 as u64, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("O(k^n) algorithm (10% lower than the theoretical value)", BigOAlgorithmComplexity::OkN,
               AlgorithmPassesInfo { pass1_n: 10, pass2_n: 70 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1.0e1 as u64,           time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: (1.0e7 * 0.901) as u64, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("O(k^n) algorithm (10% greater than the theoretical value)", BigOAlgorithmComplexity::OkN,
               AlgorithmPassesInfo { pass1_n: 10, pass2_n: 70 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1.0e1 as u64,           time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: (1.0e7 * 1.099) as u64, time_unit: &TimeUnits::MICROSECOND }
               });

        assert("Worse than exponential algorithm", BigOAlgorithmComplexity::WorseThanExponential,
               AlgorithmPassesInfo { pass1_n: 10, pass2_n: 70 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { elapsed_time: 1.0e1 as u64,           time_unit: &TimeUnits::MICROSECOND },
                   pass_2_measurements: BigOTimePassMeasurements { elapsed_time: (1.0e7 * 1.101) as u64, time_unit: &TimeUnits::MICROSECOND }
               });

    }

    /// test the time complexity analysis results based on some known-to-be-correct measurement times
    #[test]
    #[serial]
    fn analyse_constant_set_iterator_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: ConstantSetIteratorAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_time_complexity = analyse_time_complexity_for_constant_set_iterator_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET iterator algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 - (PERCENT_TOLERANCE*100.0) as u64 - 1},
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
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 150 },
        });

        assert("Theoretical O(n) Update/Select", BigOAlgorithmComplexity::ON,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 200 },
        });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::ONLogN,
               ConstantSetIteratorAlgorithmPassesInfo { pass_1_set_size: 1000, pass_2_set_size: 2000, repetitions: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 226 },
        });

    }

    /// test the time complexity analysis results based on some known-to-be-correct measurement times
    #[test]
    #[serial]
    fn analyse_set_resizing_iterator_algorithm_theoretical_test() {

        let assert = |measurement_name, expected_complexity, passes_info: SetResizingIteratorAlgorithmPassesInfo, time_measurements: BigOTimeMeasurements<_>| {
            let observed_complexity = analyse_time_complexity_for_set_resizing_iterator_algorithm(&passes_info, &time_measurements);
            assert_eq!(observed_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING iterator algorithm for '{}' check failed!", measurement_name);
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
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 122 },
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

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenONAndONLogN,
               SetResizingIteratorAlgorithmPassesInfo { delta_set_size: 1000 },
               BigOTimeMeasurements {
                   pass_1_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 100 },
                   pass_2_measurements: BigOTimePassMeasurements { time_unit: &TimeUnits::MICROSECOND, elapsed_time: 333 },
        });
    }

}