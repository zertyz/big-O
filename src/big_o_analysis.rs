//! Contains the "low level" artifacts for analyzing an algorithm's time measurements, in Big-O notation:
//!   - [analyse_constant_set_algorithm] & [analyse_set_resizing_algorithm] -- functions performing the analysis;
//!   - [ConstantSetAlgorithmMeasurements] & [SetResizingAlgorithmMeasurements] --structs for holding the measurements;
//!   - [BigOAlgorithmComplexity] -- analysis result enum & pretty str methods.

use std::fmt::{Display,Formatter};

/// acceptable variance (or errors) when measuring times
const PERCENT_TOLERANCE: f64 = 0.10;

/// return result for this module's main functions [analyse_constant_set_algorithm] & [analyse_set_resizing_algorithm]
pub struct BigOAlgorithmAnalysis<T: BigOTimeMeasurements/*,
                                 S: BigSpaceMeasurements*/> {
    pub time_complexity:    BigOAlgorithmComplexity,
    pub time_measurements:  T,
/*    /// related to the maximum used memory (even if it is freed afterwards)
    pub space_complexity:   BigOAlgorithmComplexity,
    pub space_measurements: S,*/
}
impl<T: BigOTimeMeasurements> Display for BigOAlgorithmAnalysis<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = || write!(f, "{}--> Algorithm Analysis: {}", self.time_measurements, self.time_complexity.as_pretty_str());
        match self.time_complexity {
            BigOAlgorithmComplexity::BetterThanO1      => fmt(),
            BigOAlgorithmComplexity::O1                => fmt(),
            BigOAlgorithmComplexity::OLogN             => fmt(),
            BigOAlgorithmComplexity::BetweenOLogNAndON => fmt(),
            BigOAlgorithmComplexity::ON                => fmt(),
            BigOAlgorithmComplexity::WorseThanON       => fmt(),
        }
    }
}

/// base trait for [SetResizingAlgorithmMeasurements] & [ConstantSetAlgorithmMeasurements], made public
/// to attend to rustc's rules. Most probably this trait is of no use outside it's own module.
pub trait BigOTimeMeasurements: Display {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BigOAlgorithmComplexity {
    BetterThanO1,
    O1,
    OLogN,
    BetweenOLogNAndON,
    ON,
    WorseThanON,
}
impl BigOAlgorithmComplexity {
    /// verbose description for each enum element
    pub fn as_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1) -- aren't the machines idle? too many threads? too little RAM?",
            Self::O1                => "O(1)",
            Self::OLogN             => "O(log(n))",
            Self::BetweenOLogNAndON => "Worse than O(log(n)) but better than O(n)",
            Self::ON                => "O(n)",
            Self::WorseThanON       => "Worse than O(n)",
        }
    }
}

/// Represents the measurements made on Algorithms that don't alter the set size of the data they operate on
/// (Selects / Updates / Sort / Fib...). This struct keeps average times (instead of best times) and it should
/// be used on latency variation conditions (network, several threads, ...) and also for searches (naive
/// implementations might find the first element immediately)
pub struct ConstantSetAlgorithmMeasurements<'a> {
    /// a name for this measurement, for presentation purposes
    pub measurement_name: &'a str,
    /// the absolute time (same unit as 'pass_2_total_time') it took to run "pass 1" (an operation applied
    /// 'repetitions' times, on an universe of 'pass_1_set_size' elements)
    pub pass_1_total_time: u64,
    /// the absolute time (same unit as 'pass_1_total_time') it took to run "pass 2" (an operation applied
    /// 'repetitions' times, on an universe of 'pass_2_set_size' elements)
    pub pass_2_total_time: u64,
    /// the maximum used memory during warm up (if enabled) + pass 1 execution, in bytes
    pub pass_1_max_mem: u32,
    /// the maximum used memory during pass 2 execution, in bytes
    pub pass_2_max_mem: u32,
    /// set size when running "pass 1" (an operation repeated 'repetitions' times)
    pub pass_1_set_size: u32,
    /// set size when running "pass 2" (an operation repeated 'repetitions' times)
    pub pass_2_set_size: u32,
    /// number of times the algorithm ran on each pass;
    /// each algorithm iteration should behave as executing on the same element without leaving side-effects
    pub repetitions: u32,
}
impl<'a> BigOTimeMeasurements for ConstantSetAlgorithmMeasurements<'a> {}
impl<'a> Display for ConstantSetAlgorithmMeasurements<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' constant set algorithm measurements:\n\
                     pass         Δt            Δs            Σn            ⊆r            t⁻\n\
                     1) {:>12}  {:>12}  {:>12}  {:>12}  {:>12.3}\n\
                     2) {:>12}  {:>12}  {:>12}  {:>12}  {:>12.3}\n",
               self.measurement_name,
               self.pass_1_total_time, self.pass_1_max_mem, self.pass_1_set_size, self.repetitions, self.pass_1_total_time as f64 / self.repetitions as f64,
               self.pass_2_total_time, self.pass_2_max_mem, self.pass_2_set_size, self.repetitions, self.pass_2_total_time as f64 / self.repetitions as f64)
    }
}

/// Performs the algorithm analysis for a reasonably large select/update operation (on a database or not).
/// To perform the analysis, two passes of selects/updates of r elements must be done.
/// On the first pass, the data set must have 'n1' elements and, on the second pass, 'n2' elements -- 'n2' must be (at least) twice 'n1'.
/// 'r' should be reasonably large so that end-start can be accurately measured and account for OS, IO and network latencies.
/// 'start's 1 & 2 and 'end's 1 & 2 are measurement times, regardless of the measurement unit -- milliseconds or microseconds.
/// The returned algorithm complexity is an indication of the time taken to select/update one element on a data set containing
/// 'n' elements, where 'O' is the constant of proportionality -- the average time to select/update 1 element.\
/// Returns: [1] -- the algorithm complexity;\
///          [2] -- a string with the algorithm analysis report.\
pub fn analyse_constant_set_algorithm(measurements: ConstantSetAlgorithmMeasurements) -> BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements> {

    // time variation
    let t1 = measurements.pass_1_total_time as f64 / measurements.repetitions as f64;
    let t2 = measurements.pass_2_total_time as f64 / measurements.repetitions as f64;

    // set size variation
    let n1 = std::cmp::min(measurements.pass_1_set_size, measurements.pass_2_set_size) as f64;
    let n2 = std::cmp::max(measurements.pass_1_set_size, measurements.pass_2_set_size) as f64;

    let computed_complexity: BigOAlgorithmAnalysis<ConstantSetAlgorithmMeasurements>;

    if ((t1/t2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::BetterThanO1, time_measurements: measurements };
    } else if ((t2/t1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::O1, time_measurements: measurements };
    } else if ( ((t2/t1) / ( n2.log2() / n1.log2() )) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n2)/log(n1)) ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::OLogN, time_measurements: measurements };
    } else if ( ((t2/t1) / (n2 / n1)) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / (n2/n1) ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::ON, time_measurements: measurements };
    } else if ( ((t2/t1) / (n2 / n1)) - 1.0_f64 ) > PERCENT_TOLERANCE {
        // check for worse than O(n)
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::WorseThanON, time_measurements: measurements };
    } else {
        // by exclusion...
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::BetweenOLogNAndON, time_measurements: measurements };
    }

    computed_complexity
}

/// Represents the measurements made on Algorithms that alters the set size of the data they operate on
/// (Inserts / Deletes / Pushes / Pops / Enqueues / Dequeues...)
pub struct SetResizingAlgorithmMeasurements<'a> {
    /// a name for this measurement, for presentation purposes
    pub measurement_name:   &'a str,
    /// the absolute time (same unit as 'pass_2_total_time') it took to run "pass 1" (an operation repeated
    /// 'processing_subset_size' times, leaving 'pass_1_end_set_size' elements at the end)
    pub pass_1_total_time:      u64,
    /// the absolute time (same unit as 'pass_1_total_time') it took to run "pass 2" (an operation repeated
    /// 'processing_subset_size' times, leaving 'pass_2_end_set_size' elements at the end)
    pub pass_2_total_time:      u64,
    /// the maximum used memory during warm up (if enabled) + pass 1 execution, in bytes
    pub pass_1_max_mem: u32,
    /// the maximum used memory during pass 2 execution, in bytes
    pub pass_2_max_mem: u32,
    /// number of elements added / removed on each pass;
    /// each algorithm iteration should either add or remove a single element
    /// and the test set must start or end with 0 elements
    pub delta_set_size: u32,
}
impl<'a> BigOTimeMeasurements for SetResizingAlgorithmMeasurements<'a> {}
impl<'a> Display for SetResizingAlgorithmMeasurements<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' set resizing algorithm measurements:\n\
                   pass         Δt            Σn            t⁻\n\
                   1) {:>12}  {:>12}  {:>12.3}\n\
                   2) {:>12}  {:>12}  {:>12.3}\n",
                self.measurement_name,
                self.pass_1_total_time, self.delta_set_size,   self.pass_1_total_time as f64 / self.delta_set_size as f64,
                self.pass_2_total_time, self.delta_set_size*2, self.pass_2_total_time as f64 / self.delta_set_size as f64)
    }
}

pub fn analyse_set_resizing_algorithm(measurements: SetResizingAlgorithmMeasurements) -> BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements> {

    let n = measurements.delta_set_size as f64;

    // time variation
    let t1 = measurements.pass_1_total_time as f64 / n;
    let t2 = measurements.pass_2_total_time as f64 / n;


    let computed_complexity: BigOAlgorithmAnalysis<SetResizingAlgorithmMeasurements>;

    if ((t1/t2) - 1.0_f64) > PERCENT_TOLERANCE {
        // sanity check
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::BetterThanO1, time_measurements: measurements };
    } else if ((t2/t1) - 1.0_f64).abs() <= PERCENT_TOLERANCE {
        // check for O(1) -- t2/t1 ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::O1, time_measurements: measurements };
    } else if ( ((t2/t1) / ( (n * 3.0_f64).log2() / n.log2() )) - 1.0_f64 ).abs() < PERCENT_TOLERANCE {
        // check for O(log(n)) -- (t2/t1) / (log(n*3)/log(n)) ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::OLogN, time_measurements: measurements };
    } else if ( ((t2/t1) / 3.0_f64) - 1.0_f64 ).abs() <= PERCENT_TOLERANCE {
        // check for O(n) -- (t2/t1) / 3 ~= 1
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::ON, time_measurements: measurements };
    } else if ( ((t2/t1) / 3.0_f64) - 1.0_f64 ) > PERCENT_TOLERANCE {
        // check for worse than O(n)
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::WorseThanON, time_measurements: measurements };
    } else {
        // by exclusion...
        computed_complexity = BigOAlgorithmAnalysis { time_complexity: BigOAlgorithmComplexity::BetweenOLogNAndON, time_measurements: measurements };
    }

    computed_complexity
}

#[cfg(test)]
mod tests {

    use super::super::{conditionals,BigOAlgorithmType,TimeUnit,TimeUnits,run_pass};
    use crate::conditionals::{OUTPUT};

    use super::*;
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
    fn analyse_constant_set_algorithm_theoretical_test() {

        let measurement_name = "analyse_constant_set_algorithm_theoretical_test";

        let assert = |measurement_name, expected_complexity, mut measurements: ConstantSetAlgorithmMeasurements| {
            measurements.measurement_name = measurement_name;
            let observed_analysis = analyse_constant_set_algorithm(measurements);
            OUTPUT(&format!("{}\n", observed_analysis));
            assert_eq!(observed_analysis.time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Update/Select", BigOAlgorithmComplexity::BetterThanO1,ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 100 - (PERCENT_TOLERANCE*100.0) as u64,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    0,
            pass_1_set_size:   1000,
            pass_2_set_size:   2000,
            repetitions:       1000
        });

        assert("Theoretical O(1) Update/Select", BigOAlgorithmComplexity::O1, ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 100,
            pass_1_max_mem:    0,
            pass_2_max_mem:    0,
            pass_1_set_size:   1000,
            pass_2_set_size:   2000,
            repetitions:       1000
        });

        assert("Theoretical O(log(n)) Update/Select", BigOAlgorithmComplexity::OLogN, ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 111,
            pass_1_max_mem:    (1000 as f32).ln() as u32,
            pass_2_max_mem:    (1000 as f32).ln() as u32,
            pass_1_set_size:   1000,
            pass_2_set_size:   2000,
            repetitions:       1000
        });

        assert("Theoretical between O(log(n)) and O(n) Update/Select", BigOAlgorithmComplexity::BetweenOLogNAndON, ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 200,
            pass_1_max_mem:    1000 / (1000 as f32).ln() as u32,
            pass_2_max_mem:    1000 / (1000 as f32).ln() as u32,
            pass_1_set_size:   1000,
            pass_2_set_size:   2500,
            repetitions:       1000
        });

        assert("Theoretical O(n) Update/Select", BigOAlgorithmComplexity::ON, ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 200,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    1024,
            pass_1_set_size:   1000,
            pass_2_set_size:   2000,
            repetitions:       1000
        });

        assert("Theoretical worse than O(n) Update/Select", BigOAlgorithmComplexity::WorseThanON, ConstantSetAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 222,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    2048,
            pass_1_set_size:   1000,
            pass_2_set_size:   2000,
            repetitions:       1000
        });

    }

    #[test]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_theoretical_test() {

        let measurement_name = "analyse_set_resizing_algorithm_theoretical_test";

        let assert = |measurement_name, expected_complexity, mut measurements: SetResizingAlgorithmMeasurements| {
            measurements.measurement_name = measurement_name;
            let observed_analysis = analyse_set_resizing_algorithm(measurements);
            OUTPUT(&format!("{}\n", observed_analysis));
            assert_eq!(observed_analysis.time_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);
        };

        assert("Theoretical better than O(1) Insert/Delete", BigOAlgorithmComplexity::BetterThanO1, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 100 - (PERCENT_TOLERANCE*100.0) as u64,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    0,
            delta_set_size:    1000
        });

        assert("Theoretical O(1) Insert/Delete", BigOAlgorithmComplexity::O1, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 100,
            pass_1_max_mem:    0,
            pass_2_max_mem:    0,
            delta_set_size:    1000
        });

        assert("Theoretical O(log(n)) Insert/Delete", BigOAlgorithmComplexity::OLogN, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 111,
            pass_1_max_mem:    (1000 as f32).ln() as u32,
            pass_2_max_mem:    (1000 as f32).ln() as u32,
            delta_set_size:    1000
        });

        assert("Theoretical between O(log(n)) and O(n) Insert/Delete", BigOAlgorithmComplexity::BetweenOLogNAndON, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 200,
            pass_1_max_mem:    1000 / (1000 as f32).ln() as u32,
            pass_2_max_mem:    1000 / (1000 as f32).ln() as u32,
            delta_set_size:    1000
        });

        assert("Theoretical O(n) Insert/Delete", BigOAlgorithmComplexity::ON, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 300,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    1024,
            delta_set_size:    1000
        });

        assert("Theoretical worse than O(n) Insert/Delete", BigOAlgorithmComplexity::WorseThanON, SetResizingAlgorithmMeasurements {
            measurement_name,
            pass_1_total_time: 100,
            pass_2_total_time: 333,
            pass_1_max_mem:    1024,
            pass_2_max_mem:    2048,
            delta_set_size:    1000
        });
    }

    #[test]
    #[serial(cpu)]
    fn time_analysis_smooth_transitions() {
        // constant_set
        let mut last_time_analysis = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_total_time in 0..500 {
            let current_analysis = analyse_constant_set_algorithm(ConstantSetAlgorithmMeasurements {
                measurement_name: "Smooth Transitions",
                pass_1_total_time: 100,
                pass_2_total_time,
                pass_1_max_mem: 100,
                pass_2_max_mem: pass_2_total_time as u32,   // TODO review if smooth analysis should be done all in one shot or if it should be split for time & space
                pass_1_set_size: 1000,
                pass_2_set_size: 2000,
                repetitions: 1000
            });
            let delta = current_analysis.time_complexity as i32 - last_time_analysis as i32;
            assert!(delta == 0 || delta == 1, "Time analysis 'analyse_constant_set_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_total_time of {}", last_time_analysis, current_analysis.time_complexity, pass_2_total_time);
            if delta == 1 {
                last_time_analysis = current_analysis.time_complexity;
                eprintln!("'analyse_constant_set_algorithm(...)' transitioned to {:?} at {}", current_analysis.time_complexity, pass_2_total_time);
            }
        }

        // set_resizing
        let mut last_time_analysis = BigOAlgorithmComplexity::BetterThanO1;
        for pass_2_total_time in 0..500 {
            let current_analysis = analyse_set_resizing_algorithm(SetResizingAlgorithmMeasurements {
                measurement_name: "Smooth Transitions",
                pass_1_total_time: 100,
                pass_2_total_time,
                pass_1_max_mem: 100,
                pass_2_max_mem: pass_2_total_time as u32,   // TODO see above
                delta_set_size: 1000,
            });
            let delta = current_analysis.time_complexity as i32 - last_time_analysis as i32;
            assert!(delta == 0 || delta == 1, "Time analysis 'analyse_set_resizing_algorithm(...)' suddenly went from {:?} to {:?} at pass_2_total_time of {}", last_time_analysis, current_analysis.time_complexity, pass_2_total_time);
            if delta == 1 {
                last_time_analysis = current_analysis.time_complexity;
                eprintln!("'analyse_set_resizing_algorithm(...)' transitioned to {:?} at {}", current_analysis.time_complexity, pass_2_total_time);
            }
        }
    }

    #[test]
    #[serial(cpu)]
    fn space_analysis_smooth_transitions() {
        todo!("do this one");
    }

    #[test]
    #[serial(cpu)]
    fn analyse_constant_set_algorithm_real_test() {

        const REPETITIONS: u32 = 4000;
        const PASS_1_SET_SIZE: u32 = REPETITIONS;
        const PASS_2_SET_SIZE: u32 = REPETITIONS * 3;

        fn o_1_select(mut _n: u32) -> u32 {
            // allocations: zero used memory
            busy_loop(BUSY_LOOP_DELAY*5)
        }

        fn o_log_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 1;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut vec = Vec::<u32>::with_capacity(0);
            while n > 0 {
                r += busy_loop(BUSY_LOOP_DELAY/2);
                n /= 2;
                vec.push(n);
            }
            r ^ vec.iter().sum::<u32>()
        }

        fn o_n_select(mut n: u32) -> u32 {
            let mut r: u32 = 2;
            if n < PASS_1_SET_SIZE {
                n = PASS_1_SET_SIZE;
            } else {
                n = PASS_2_SET_SIZE;
            }
            let mut vec = Vec::<u32>::with_capacity(0);
            while n > 0 {
                r += busy_loop(BUSY_LOOP_DELAY/500);
                n -= 1;
                vec.push(n);
            }
            r ^ vec.iter().sum::<u32>()
        }

        let assert = |measurement_name, select_function: fn(u32) -> u32, expected_complexity| {
            OUTPUT(&format!("Real '{}' adding {} elements on each pass ", measurement_name, REPETITIONS));

            let (warmup_result, r1) = _run_pass("(warmup: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. REPETITIONS / 10,                            &TimeUnits::MICROSECOND);
            let (pass_1_result, r2) = _run_pass("; pass1: ", "",    select_function, &BigOAlgorithmType::ConstantSet, 0 .. PASS_1_SET_SIZE,                             &TimeUnits::MICROSECOND);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", select_function, &BigOAlgorithmType::ConstantSet, PASS_2_SET_SIZE - REPETITIONS .. PASS_2_SET_SIZE, &TimeUnits::MICROSECOND);

            let observed_analysis = analyse_constant_set_algorithm(ConstantSetAlgorithmMeasurements {
                measurement_name,
                pass_1_total_time: pass_1_result.elapsed_time,
                pass_2_total_time: pass_2_result.elapsed_time,
                pass_1_max_mem:    pass_1_result.min_max_delta_bytes,
                pass_2_max_mem:    pass_2_result.min_max_delta_bytes,
                pass_1_set_size:   PASS_1_SET_SIZE,
                pass_2_set_size:   PASS_2_SET_SIZE,
                repetitions:       REPETITIONS,
            });
            OUTPUT(&format!("\n{} (r={})\n", observed_analysis, r1+r2+r3));
            assert_eq!(observed_analysis.time_complexity, expected_complexity, "Algorithm Analysis on CONSTANT SET algorithm for '{}' check failed!", measurement_name);

        };

        assert("O1_select() function",    o_1_select,     BigOAlgorithmComplexity::O1);
        assert("OLogN_select() function", o_log_n_select, BigOAlgorithmComplexity::OLogN);
        assert("ON_select() function",    o_n_select,     BigOAlgorithmComplexity::ON);

    }

    #[test]
    #[serial(cpu)]
    fn analyse_set_resizing_algorithm_real_test() {

        fn o_1_insert(mut _n: u32) -> u32 {
            // single element allocation & busy_loop time processing
            let mut vec = vec![busy_loop(BUSY_LOOP_DELAY*2)];
            vec.iter().sum()
        }

        fn o_log_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let mut vec = Vec::<u32>::with_capacity(0);
            while n > 0 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/2);
                n = n/2;
                vec.push(n);
            }
            r ^ vec.iter().sum::<u32>()
        }

        /// this would be an O(n/2) function -- the average case for a naive sorted insert... but still O(n). Change n = n-2 to n = n-1 and the analysis will be the same.
        fn o_n_insert(mut n: u32) -> u32 {
            let mut r: u32 = 0;
            let mut vec = Vec::<u32>::with_capacity(0);
            while n > 1 {
                r = r ^ busy_loop(BUSY_LOOP_DELAY/50);
                n = n-2;
                vec.push(n);
            }
            r ^ vec.iter().sum::<u32>()
        }

        let assert = |measurement_name, insert_function: fn(u32) -> u32, expected_complexity| {
            let delta_set_size = 3000;
            OUTPUT(&format!("Real '{}' with {} elements on each pass ", measurement_name, delta_set_size));

            /* warmup pass -- container / database should be reset before and after this */
            let (warmup_result, r1) = _run_pass("(warmup: ", "",    insert_function, &BigOAlgorithmType::SetResizing, 0 .. delta_set_size / 10,             &TimeUnits::MICROSECOND);
            /* if we were operating on real data, we would reset the container / database after the warmup, before running pass 1 */
            let (pass_1_result, r2) = _run_pass("; pass1: ", "",    insert_function, &BigOAlgorithmType::SetResizing, 0 .. delta_set_size,                  &TimeUnits::MICROSECOND);
            let (pass_2_result, r3) = _run_pass("; pass2: ", "): ", insert_function, &BigOAlgorithmType::SetResizing, delta_set_size .. delta_set_size * 2, &TimeUnits::MICROSECOND);

            let observed_analysis = analyse_set_resizing_algorithm(SetResizingAlgorithmMeasurements {
                measurement_name,
                pass_1_total_time: pass_1_result.elapsed_time,
                pass_2_total_time: pass_2_result.elapsed_time,
                pass_1_max_mem:    pass_1_result.min_max_delta_bytes,
                pass_2_max_mem:    pass_2_result.min_max_delta_bytes,
                delta_set_size,
            });
            OUTPUT(&format!("\n{} (r={})\n", observed_analysis, r1^r2^r3));
            assert_eq!(observed_analysis.time_complexity, expected_complexity, "Algorithm Analysis on SET RESIZING algorithm for '{}' check failed!", measurement_name);

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
    fn _run_pass<_AlgorithmClosure: Fn(u32) -> u32 + Sync,
                 T: TryInto<u64> > (result_prefix: &str,
                                    result_suffix: &str,
                                    algorithm: _AlgorithmClosure,
                                    algorithm_type: &BigOAlgorithmType,
                                    range: Range<u32>,
                                    unit: &TimeUnit<T>) -> (PassResult, u32) {
        let (pass_result, r) = run_pass(&algorithm, algorithm_type, range, unit, 1);
        OUTPUT(&format!("{}{}{}{}", result_prefix, pass_result.elapsed_time, unit.unit_str, result_suffix));
        (pass_result, r)
    }

}