//! Contains the "low level" artifacts for analyzing an algorithm's time measurements, in Big-O notation:
//!   - [analyse_constant_set_algorithm] & [analyse_set_resizing_algorithm] -- functions performing the analysis;
//!   - [ConstantSetAlgorithmMeasurements] & [SetResizingAlgorithmMeasurements] --structs for holding the measurements;
//!   - [BigOAlgorithmComplexity] -- analysis result enum & pretty str methods.

pub mod types;
pub mod time_analysis;
pub mod space_analysis;

use crate::big_o_analysis::types::*;
use crate::big_o_analysis::time_analysis::*;
use crate::big_o_analysis::space_analysis::*;

#[cfg(test)]
mod tests {

    use super::*;

    use super::super::{conditionals,BigOAlgorithmType,run_pass,big_o_analysis::{TimeUnit,TimeUnits}};
    use crate::conditionals::{OUTPUT};

    use std::ops::Range;
    use std::convert::TryInto;

    use serial_test::serial;
    use crate::PassResult;

    const BUSY_LOOP_DELAY: u32 = 999*conditionals::LOOP_MULTIPLIER;

    #[test]
    #[serial(cpu)]
    fn serialization() {
        OUTPUT("BigOAlgorithmComplexity enum members, as strings:\n");
        let enum_members = [
            BigOAlgorithmComplexity::BetterThanO1,
            BigOAlgorithmComplexity::O1,
            BigOAlgorithmComplexity::OLogN,
            BigOAlgorithmComplexity::BetweenOLogNAndON,
            BigOAlgorithmComplexity::ON,
            BigOAlgorithmComplexity::WorseThanON,
        ];
        for enum_member in enum_members {
            OUTPUT(&format!("\t{:?} => '{}'\n", enum_member, enum_member.as_pretty_str()));
        }
        OUTPUT("\n");
    }

    #[test]
    #[serial(cpu)]
    fn analyse_constant_set_algorithm_real_test() {

        const REPETITIONS: u32 = 4000;
        const PASS_1_SET_SIZE: u32 = REPETITIONS;
        const PASS_2_SET_SIZE: u32 = REPETITIONS * 3;
        const TIME_UNIT: &TimeUnit<u128> = &TimeUnits::MICROSECOND;

        fn o_1_select(mut _n: u32) -> u32 {
            // single element allocation & busy_loop time processing
            let vec = vec![busy_loop(BUSY_LOOP_DELAY*5)];
            vec.iter().sum()
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
                r += busy_loop(BUSY_LOOP_DELAY/2);
                n /= 2;
                len += 1;
            }
            let mut vec = Vec::<u32>::with_capacity(len*400);
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
                r += busy_loop(BUSY_LOOP_DELAY/500);
                n -= 1;
                len += 1;
            }
            let mut vec = Vec::<u32>::with_capacity(len);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        }

        let assert = |measurement_name, select_function: fn(u32) -> u32, expected_complexity| {
            OUTPUT(&format!("Real '{}' adding {} elements on each pass ", measurement_name, REPETITIONS));

            let (warmup_result, r1) = _run_pass("(warmup: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. REPETITIONS / 10,                            TIME_UNIT);
            let (pass_1_result, r2) = _run_pass("; pass1: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. PASS_1_SET_SIZE,                             TIME_UNIT);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", select_function, &BigOAlgorithmType::ConstantSet, PASS_2_SET_SIZE - REPETITIONS .. PASS_2_SET_SIZE, TIME_UNIT);

            let constant_set_passes_info = ConstantSetAlgorithmPassesInfo {
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

            let time_complexity  = analyse_time_complexity_for_constant_set_algorithm(&constant_set_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_constant_set_algorithm(&constant_set_passes_info, &space_measurements);

            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: ConstantSetAlgorithmMeasurements {
                    measurement_name,
                    passes_info: constant_set_passes_info,
                    time_measurements,
                    space_measurements
                },
            };

            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1+r2+r3));
            assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
            assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);

        };

        assert("O1_select() function",    o_1_select,     BigOAlgorithmComplexity::O1);
        assert("OLogN_select() function", o_log_n_select, BigOAlgorithmComplexity::OLogN);
        assert("ON_select() function",    o_n_select,     BigOAlgorithmComplexity::ON);

    }

    #[test]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_real_test() {

        const DELTA_SET_SIZE: u32 = 3000;

        fn o_1_insert(mut _n: u32) -> u32 {
            // single element allocation & busy_loop time processing
            let vec = vec![busy_loop(BUSY_LOOP_DELAY*2)];
            vec.iter().sum()
        }

        fn o_log_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let mut len = if n==DELTA_SET_SIZE-1 {DELTA_SET_SIZE*2/3} else {0};
            while n > 0 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/2);
                n = n/2;
                len += n;
            }
            let vec = Vec::<u32>::with_capacity(len as usize);
            r ^ (len + vec.iter().sum::<u32>())
        };

        /// this would be an O(n/2) function -- the average case for a naive sorted insert... but still O(n). Change n = n-2 to n = n-1 and the analysis will be the same.
        fn o_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let len = if n<=DELTA_SET_SIZE {(n/20)*(n/20)} else {(n/20)*(n/20)-(n/40)*(n/40)};
            while n > 1 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/50);
                n = n-2;
            }
            let vec = Vec::<u32>::with_capacity(len as usize * 400);
            r ^ (len as u32 + vec.iter().sum::<u32>())
        };

        let assert = |measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            OUTPUT(&format!("Real '{}' with {} elements on each pass ", measurement_name, DELTA_SET_SIZE));

            /* warmup pass -- container / database should be reset before and after this */
            let (warmup_result, r1) = _run_pass("(warmup: ", "", insert_function, &BigOAlgorithmType::SetResizing, 0 .. DELTA_SET_SIZE / 10, &TimeUnits::MICROSECOND);
            /* if we were operating on real data, we would reset the container / database after the warmup, before running pass 1 */
            let (pass_1_result, r2) = _run_pass("; pass1: ", "", insert_function, &BigOAlgorithmType::SetResizing, 0 ..DELTA_SET_SIZE, &TimeUnits::MICROSECOND);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", insert_function, &BigOAlgorithmType::SetResizing, DELTA_SET_SIZE.. DELTA_SET_SIZE * 2, &TimeUnits::MICROSECOND);

            let set_resizing_passes_info = SetResizingAlgorithmPassesInfo { delta_set_size: DELTA_SET_SIZE };

            let time_measurements = BigOTimeMeasurements {
                pass_1_measurements: pass_1_result.time_measurements,
                pass_2_measurements: pass_2_result.time_measurements,
            };

            let space_measurements = BigOSpaceMeasurements {
                pass_1_measurements: pass_1_result.space_measurements,
                pass_2_measurements: pass_2_result.space_measurements,
            };

            let time_complexity  = analyse_time_complexity_for_set_resizing_algorithm(&set_resizing_passes_info, &time_measurements);
            let space_complexity = analyse_space_complexity_for_set_resizing_algorithm(&set_resizing_passes_info, &space_measurements);
            
            let algorithm_analysis = BigOAlgorithmAnalysis {
                time_complexity,
                space_complexity,
                algorithm_measurements: SetResizingAlgorithmMeasurements {
                    measurement_name,
                    passes_info: set_resizing_passes_info,
                    time_measurements,
                    space_measurements,
                },
            };
            
            OUTPUT(&format!("\n{} (r={})\n", algorithm_analysis, r1^r2^r3));
            assert_eq!(algorithm_analysis.space_complexity, expected_complexity, "Algorithm SPACE Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
            assert_eq!(algorithm_analysis.time_complexity,  expected_complexity, "Algorithm TIME  Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
        };

        assert("O1_insert() function",    o_1_insert,     BigOAlgorithmComplexity::O1);
        assert("OLogN_insert() function", o_log_n_insert, BigOAlgorithmComplexity::OLogN);
        assert("ON_insert() function",    o_n_insert,     BigOAlgorithmComplexity::ON);
    }

   #[inline]
    fn busy_loop(iterations: u32) -> u32 {
        let mut r: u32 = iterations;
        for i in 0..iterations {
            r ^= i;
        }
        r
    }

    /// wrap around the original 'run_pass' to output intermediate results
    fn _run_pass<'a,
                 _AlgorithmClosure: Fn(u32) -> u32 + Sync,
                 T: TryInto<u64> + Copy > (result_prefix: &str,
                                           result_suffix: &str,
                                           algorithm: _AlgorithmClosure,
                                           algorithm_type: &BigOAlgorithmType,
                                           range: Range<u32>,
                                           unit: &'a TimeUnit<T>) -> (PassResult<'a,T>, u32) {
        let (pass_result, r) = run_pass(&algorithm, algorithm_type, range, unit, 1);
        OUTPUT(&format!("{}{}/{}{}", result_prefix, pass_result.time_measurements, pass_result.space_measurements, result_suffix));
        (pass_result, r)
    }

}