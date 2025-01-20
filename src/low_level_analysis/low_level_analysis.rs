//! See [super].

use super::{
    configs::PERCENT_TOLERANCE,
    types::{BigOAlgorithmComplexity},
};


/// Performs the Algorithm Complexity Analysis on the resource denoted by `u`, where `u1` & `u2` are the resource
/// utilization on passes 1 & 2 and, likewise, `n1` & `n2` represent the number of element, iterations or computations
/// -- in other words, represents the `n` in the Big-O notation... `O(n)`, `O(log(n))`, `O(n²)`, etc...
pub fn analyse_complexity(u1: f64, u2: f64, n1: f64, n2: f64) -> BigOAlgorithmComplexity {
    if (u2 / u1) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetterThanO1
    } else if ((u2 / u1) - 1.0).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::O1
    } else if ((u2 / u1) / ( n2.log2() / n1.log2() )) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenO1AndOLogN
    } else if ( ((u2 / u1) / ( n2.log2() / n1.log2() )) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::OLogN
    } else if ((u2 / u1) / (n2 / n1)) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenOLogNAndON
    } else if ( ((u2 / u1) / (n2 / n1)) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ON
    } else if ((u2 / u1) / ( (n2*n2.log2()) / (n1*n1.log2()) )) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenONAndONLogN
    } else if ( ((u2 / u1) / ( (n2*n2.log2()) / (n1*n1.log2()) )) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ONLogN
    } else if ((u2 / u1) / (n2 / n1).powi(2)) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenONLogNAndON2
    } else if ( ((u2 / u1) / (n2 / n1).powi(2)) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ON2
    } else if ((u2 / u1) / (n2 / n1).powi(3)) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenON2AndON3
    } else if ( ((u2 / u1) / (n2 / n1).powi(3)) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ON3
    } else if ((u2 / u1) / (n2 / n1).powi(4)) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenON3AndON4
    } else if ( ((u2 / u1) / (n2 / n1).powi(4)) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ON4
    } else if (u2 / u1.powf(n2/n1)) < 1.0 - PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenON4AndOkN
    } else if ( (u2 / u1.powf(n2/n1)) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::OkN
    } else {
        BigOAlgorithmComplexity::WorseThanExponential
    }
}

/// TODO 2022-06-30: fix the math here (and tests) to the same standards as the function above -- and also include the increased complexity levels
/// Performs the Algorithm Complexity Analysis on an iterator algorithm that alters the elements it operates on as it runs.\
///   - `u1` & `u2` are the resource utilization on passes 1 & 2
///   - `n` represent the number of element added or remove on each pass
pub fn analyse_set_resizing_iterator_complexity(u1: f64, u2: f64, n: f64) -> BigOAlgorithmComplexity {
    if ((u1 / u2) - 1.0) > PERCENT_TOLERANCE {
        // sanity check
        BigOAlgorithmComplexity::BetterThanO1
    } else if ((u2 / u1) - 1.0).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::O1
    } else if ((u2 / u1) / ( (n * 3.0).log2() / n.log2() )) < 1.0 {
        BigOAlgorithmComplexity::BetweenO1AndOLogN
    } else if ( ((u2 / u1) / ( (n * 3.0).log2() / n.log2() )) - 1.0 ).abs() < PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::OLogN
    } else if ( ((u2 / u1) / 3.0) - 1.0 ).abs() <= PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::ON
    } else if ( ((u2 / u1) / 3.0) - 1.0 ) > PERCENT_TOLERANCE {
        BigOAlgorithmComplexity::BetweenONAndONLogN
    } else {
        // by exclusion...
        BigOAlgorithmComplexity::BetweenOLogNAndON
    }
}


#[cfg(test)]
mod tests {

    //! Unit tests for [low_level_analysis](super) module -- using 'serial_test' crate in order to make time measurements more reliable.

    use super::*;
    use crate::{
        features::{OUTPUT},
        low_level_analysis::{
            types::{
                BigOIteratorAlgorithmType,
                BigOAlgorithmComplexity, BigOAlgorithmAnalysis,
                BigOTimeMeasurements, BigOSpaceMeasurements,
                ConstantSetIteratorAlgorithmPassesInfo, SetResizingIteratorAlgorithmPassesInfo,
                ConstantSetIteratorAlgorithmMeasurements, SetResizingIteratorAlgorithmMeasurements,
            },
            time_analysis::*,
            space_analysis::*
        },
        runners::common::{run_iterator_pass_verbosely},
    };
    use std::{
        time::{Duration},
    };
    use serial_test::serial;
    use crate::low_level_analysis::types::BigOPassMeasurements;

    /// test algorithm complexity analysis progression when resource utilization increase for regular, non-iterator algorithms
    /// and for constant set iterator algorithms
    #[test]
    #[serial]
    fn smooth_transitions() {
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for u2 in 0..11_000_001 {
            let current_complexity = analyse_complexity(10.0, u2 as f64, 2.0, 14.0);
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "'analyse_complexity(..., {}, ..., ...)' suddenly went from {:?} to {:?} when `u2` when from {} to {}", u2, last_complexity, current_complexity, u2-1, u2);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_complexity(...)' transitioned to {:?} when `u2`={}", current_complexity, u2);
            }
        }
        assert_eq!(last_complexity, BigOAlgorithmComplexity::WorseThanExponential, "Please update this test to cycle through all variants of `BigOAlgorithmComplexity`");
    }

    /// test algorithm complexity analysis progression when resource utilization increase for set resizing iterator algorithms
    #[test]
    #[serial]
    fn smooth_transitions_for_set_resizing_iterator_algorithm_() {
        let mut last_complexity = BigOAlgorithmComplexity::BetterThanO1;
        for u2 in 0..500 {
            let current_complexity = analyse_set_resizing_iterator_complexity(100.0, u2 as f64, 1000.0);
            let delta = current_complexity as i32 - last_complexity as i32;
            assert!(delta == 0 || delta == 1, "'analyse_set_resizing_iterator_complexity(..., {}, ...)' suddenly went from {:?} to {:?} when `u2` went from {} to {}", u2, last_complexity, current_complexity, u2-1, u2);
            if delta == 1 {
                last_complexity = current_complexity;
                eprintln!("'analyse_set_resizing_iterator_complexity(...)' transitioned to {:?} when `u2`={}", current_complexity, u2);
            }
        }
        assert_eq!(last_complexity, BigOAlgorithmComplexity::/*WorseThanExponential*/BetweenONAndONLogN, "Please update this test to cycle through all variantes of `BigOAlgorithmComplexity`");
    }


    /// tests time & space complexity analysis on real constant set algorithms
    #[test]
    #[serial]
    fn analyse_constant_set_algorithm_real_test() {

        const REPETITIONS: u32 = 1024;
        const PASS_1_SET_SIZE: u32 = REPETITIONS;
        const PASS_2_SET_SIZE: u32 = REPETITIONS * 4;

        fn o_1_select(mut _n: u32) -> u32 {
            // constant element allocation & single operation processing
            let mut vec = Vec::with_capacity(1024);
            vec.push(operation_simulator());
            vec.iter().sum::<u32>()
        }

        fn o_log_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 1;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut len = 0;
            while n > 0 {
                r ^= operation_simulator();
                n /= 2;
                len += 1;
            }
            let vec = Vec::<u32>::with_capacity(len*4096);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        fn o_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 2;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut len = 0;
            while n > 0 {
                r ^= operation_simulator();
                n -= 1;
                len += 1;
            }
            let vec = Vec::<u32>::with_capacity(len*4096);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        let analyze = |measurement_name, select_function: fn(u32) -> u32| {
            OUTPUT(&format!("Real '{}', fetching {} elements on each pass ", measurement_name, REPETITIONS));

            let (_warmup_result               , r1) = run_iterator_pass_verbosely("(warmup: ", "",    &select_function, &BigOIteratorAlgorithmType::ConstantSet, 0 .. REPETITIONS, 1, OUTPUT);
            let (pass_1_result, r2) = run_iterator_pass_verbosely("; pass1: ", "",    &select_function, &BigOIteratorAlgorithmType::ConstantSet, 0 .. PASS_1_SET_SIZE, 1, OUTPUT);
            let (pass_2_result, r3) = run_iterator_pass_verbosely("; pass2: ", "): ", &select_function, &BigOIteratorAlgorithmType::ConstantSet, PASS_2_SET_SIZE - REPETITIONS .. PASS_2_SET_SIZE, 1, OUTPUT);

            let constant_set_passes_info = ConstantSetIteratorAlgorithmPassesInfo {
                pass_1_set_size: PASS_1_SET_SIZE,
                pass_2_set_size: PASS_2_SET_SIZE,
                repetitions: REPETITIONS,
            };

            let time_measurements = BigOTimeMeasurements {
                pass_1_measurements: pass_1_result.time_measurements,
                pass_2_measurements: pass_2_result.time_measurements,
            };

            let space_measurements = BigOSpaceMeasurements {
                pass_1_measurements: pass_1_result.space_measurements,
                pass_2_measurements: pass_2_result.space_measurements,
            };

            let time_complexity  = analyse_time_complexity_for_constant_set_iterator_algorithm(&constant_set_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_constant_set_iterator_algorithm(&constant_set_passes_info, &space_measurements);

            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: ConstantSetIteratorAlgorithmMeasurements {
                    measurement_name,
                    passes_info: constant_set_passes_info,
                    pass1_measurements: BigOPassMeasurements {
                        time_measurements: time_measurements.pass_1_measurements,
                        space_measurements: space_measurements.pass_1_measurements,
                        custom_measurements: vec![],
                    },
                    pass2_measurements: BigOPassMeasurements {
                        time_measurements: time_measurements.pass_2_measurements,
                        space_measurements: space_measurements.pass_2_measurements,
                        custom_measurements: vec![],
                    },
                    time_measurements,
                    space_measurements,
                },
            };

            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1^r2^r3));
            algorithm_analysis
        };

        let assert_with_retry = |max_retries, measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            for attempt in 1..max_retries+1 {
                let algorithm_analysis = analyze(measurement_name, insert_function);
                assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
                if algorithm_analysis.time_complexity != expected_complexity && attempt < max_retries {
                    OUTPUT("\n==> Time measurement mismatch. Retrying...\n\n");
                    continue;
                }
                assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
                break;
            }
        };

        assert_with_retry(15, "O1_select() function",    o_1_select,     BigOAlgorithmComplexity::O1);
        assert_with_retry(15, "OLogN_select() function", o_log_n_select, BigOAlgorithmComplexity::OLogN);
        assert_with_retry(15, "ON_select() function",    o_n_select,     BigOAlgorithmComplexity::ON);

    }

    /// tests time & space complexity analysis on real set resizing algorithms
    #[test]
    #[serial]
    fn analyse_set_resizing_algorithm_real_test() {

        const DELTA_SET_SIZE: u32 = 1024;

        fn o_1_insert(mut _n: u32) -> u32 {
            // constant element allocation & single operation processing
            let mut vec = Vec::with_capacity(1024);
            vec.push(operation_simulator());
            vec.iter().sum::<u32>()
        }

        fn o_log_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let mut len = if n==DELTA_SET_SIZE-1 {DELTA_SET_SIZE*2/3} else {0};
            while n > 0 {
                r ^= operation_simulator();
                n = n/2;
                len += n;
            }
            let vec = Vec::<u32>::with_capacity(len as usize);
            r ^ (len + vec.iter().sum::<u32>())
        }

        /// this would be an O(n/2) function -- the average case for a naive sorted insert... but still O(n). Change n = n-2 to n = n-1 and the analysis will be the same.
        fn o_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let len = if n<=DELTA_SET_SIZE {(n/20)*(n/20)} else {(n/20)*(n/20)-(n/40)*(n/40)};
            while n > 1 {
                r ^= operation_simulator();
                n = n-2;
            }
            let vec = Vec::<u32>::with_capacity(len as usize * 400);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        let analyze = |measurement_name, insert_function: fn(u32) -> u32| {
            OUTPUT(&format!("Real '{}' with {} elements on each pass ", measurement_name, DELTA_SET_SIZE));

            /* warmup pass -- container / database should be reset before and after this */
            let (_warmup_result,                r1) = run_iterator_pass_verbosely("(warmup: ", "", &insert_function, &BigOIteratorAlgorithmType::SetResizing, 0 .. DELTA_SET_SIZE, 1, OUTPUT);
            /* if we were operating on real data, we would reset the container / database after the warmup, before running pass 1 */
            let (pass_1_result, r2) = run_iterator_pass_verbosely("; pass1: ", "", &insert_function, &BigOIteratorAlgorithmType::SetResizing, 0 ..DELTA_SET_SIZE, 1, OUTPUT);
            let (pass_2_result, r3) = run_iterator_pass_verbosely("; pass2: ", "): ", &insert_function, &BigOIteratorAlgorithmType::SetResizing, DELTA_SET_SIZE.. DELTA_SET_SIZE * 2, 1, OUTPUT);

            let set_resizing_passes_info = SetResizingIteratorAlgorithmPassesInfo { delta_set_size: DELTA_SET_SIZE };

            let time_measurements = BigOTimeMeasurements {
                pass_1_measurements: pass_1_result.time_measurements,
                pass_2_measurements: pass_2_result.time_measurements,
            };

            let space_measurements = BigOSpaceMeasurements {
                pass_1_measurements: pass_1_result.space_measurements,
                pass_2_measurements: pass_2_result.space_measurements,
            };

            let time_complexity  = analyse_time_complexity_for_set_resizing_iterator_algorithm(&set_resizing_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_set_resizing_iterator_algorithm(&set_resizing_passes_info, &space_measurements);

            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: SetResizingIteratorAlgorithmMeasurements {
                    measurement_name,
                    passes_info: set_resizing_passes_info,
                    time_measurements,
                    space_measurements,
                },
            };

            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1^r2^r3));
            algorithm_analysis
        };

        let assert_with_retry = |max_retries, measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            for attempt in 1..max_retries+1 {
                let algorithm_analysis = analyze(measurement_name, insert_function);
                assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
                if algorithm_analysis.time_complexity != expected_complexity && attempt < max_retries {
                    OUTPUT("\n==> Time measurement mismatch. Retrying...\n\n");
                    continue;
                }
                assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
                break;
            }
        };

        assert_with_retry(15, "O1_insert() function",    o_1_insert,     BigOAlgorithmComplexity::O1);
        assert_with_retry(15, "OLogN_insert() function", o_log_n_insert, BigOAlgorithmComplexity::OLogN);
        assert_with_retry(15, "ON_insert() function",    o_n_insert,     BigOAlgorithmComplexity::ON);
    }

   #[inline]
   /// simulates a cpu bound operation using precise sleeping --
   /// a random number is returned to avoid any call cancellation optimizations
    fn operation_simulator() -> u32 {
       const BUSY_LOOP_DELAY: u64 = 1;
       spin_sleep::sleep(Duration::from_nanos(BUSY_LOOP_DELAY));
       rand::random()
    }
}
