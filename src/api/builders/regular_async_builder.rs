//! Defines our main API (using the Builder Pattern) & related internal model
//! to allow async "regular" algorithms analysis.
//!
//! By "regular algorithms" we mean "algorithms that don't alter the amount of data they operate on"
//! -- such as algorithms that transform or update the data.
//!
//! Alias, better defined in opposition to [super::dynamic_async_builder].

use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use keen_retry::{loggable_retry_errors, ResolvedResult, RetryResult};
use crate::api::types::{AlgoAssertionAsyncFn, AlgoManipulationAsyncFn};
use crate::{low_level_analysis, BigOAlgorithmComplexity, OUTPUT};
use crate::low_level_analysis::types::{AlgorithmMeasurements, AlgorithmPassesInfo, BigOAlgorithmAnalysis, BigOPassMeasurements, BigOSpaceMeasurements, BigOTimeMeasurements};
use crate::runners::common::run_async_pass_verbosely;
use crate::utils::measurements::measurer::{measure_all_after_event, measure_all_before_event, CustomMeasurement, CustomMeasurer, CustomMeasurerExecutor};
use crate::utils::measurements::presentable_measurements::PresentableMeasurement;

/// TODO: add docs from elsewhere
pub struct RegularAsyncAnalyzerBuilder<FirstPassFn:   FnMut(Option<AlgoDataType>) -> FirstPassFut + Send + Sync,
                                       FirstPassFut:  Future<Output=AlgoDataType> + Send,
                                       SecondPassFn:  FnMut(Option<AlgoDataType>) -> SecondPassFut + Send + Sync,
                                       SecondPassFut: Future<Output=AlgoDataType> + Send,
                                       AlgoDataType:  Send + Sync + Debug> {

    test_name: String,

    reset_fn: Option<AlgoManipulationAsyncFn<AlgoDataType>>,

    max_reattempts: Option<u32>,
    warmup_fn: Option<AlgoManipulationAsyncFn<AlgoDataType>>,

    first_pass_n: u32,
    first_pass_fn: Option<FirstPassFn>,
    first_pass_measurements: Option<Vec<CustomMeasurement>>,
    first_pass_assertion_fn: Option<AlgoAssertionAsyncFn<AlgoDataType>>,

    second_pass_n: u32,
    second_pass_fn: Option<SecondPassFn>,
    second_pass_measurements: Option<Vec<CustomMeasurement>>,
    second_pass_assertion_fn: Option<AlgoAssertionAsyncFn<AlgoDataType>>,

    expected_time_complexity: Option<BigOAlgorithmComplexity>,
    expected_space_complexity: Option<BigOAlgorithmComplexity>,
    auxiliary_space_measurement: Option<BigOAlgorithmComplexity>,

    /// Measurements are done in a "delta" fashion.
    /// For details, see [Self::add_custom_measurement()].
    custom_measurers: Vec<Box<dyn CustomMeasurerExecutor<AlgoDataType>>>,
}


impl<FirstPassFn:   FnMut(Option<AlgoDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=AlgoDataType> + Send,
     SecondPassFn:  FnMut(Option<AlgoDataType>) -> SecondPassFut + Send + Sync,
     SecondPassFut: Future<Output=AlgoDataType> + Send,
     AlgoDataType:  Send + Sync + Debug>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, SecondPassFn, SecondPassFut, AlgoDataType> {

    pub async fn test_algorithm(self) {
        let max_attempts = self.max_reattempts.unwrap_or(0);
        let result = self.raw_analyse_algorithm(None).await
            .retry_with_async(|(moved_self, algo_data)| {
                OUTPUT("retrying...\n");
                moved_self.raw_analyse_algorithm(Some(algo_data))
            })
            .with_delays((0..max_attempts).map(|_| Duration::from_secs(1)))
            .await;
        let failure_msg = match result {
            ResolvedResult::Ok { .. } => None,
            ResolvedResult::Fatal { error, .. } => Some(error),
            ResolvedResult::Recovered { .. } => None,
            ResolvedResult::GivenUp { retry_errors, fatal_error, .. } => Some(format!("Given up with '{}' after {max_attempts} attempts. Previous transient errors: {}", fatal_error, loggable_retry_errors(&retry_errors))),
            ResolvedResult::Unrecoverable { retry_errors, fatal_error, .. } => Some(format!("Stopped after retrying for {max_attempts} attempts due to the fatal outcome '{}'. Previous transient errors: {}", fatal_error, loggable_retry_errors(&retry_errors))),
        };
        if let Some(failure_msg) = failure_msg {
            panic!("{}", failure_msg);
        }
    }

    async fn raw_analyse_algorithm(mut self, previous_attempt_algo_data: Option<AlgoDataType>) -> RetryResult<Self, (Self, AlgoDataType), AlgoDataType, String> {
        OUTPUT(&format!("Running '{}' async algorithm:\n", self.test_name));

        // first reset
        let algo_data = match &mut self.reset_fn {
            Some(reset_fn) => {
                let (_reset_pass_result, algo_data) = run_async_pass_verbosely("  Resetting: ", ";", previous_attempt_algo_data, reset_fn, OUTPUT).await;
                Some(algo_data)
            },
            None => None,
        };

        // warm-up (with another possible reset)
        let algo_data = match &mut self.warmup_fn {
            Some(warmup_fn) => {
                let (_warmup_pass_result, algo_data) = run_async_pass_verbosely("  Warming up: ", ";", algo_data, warmup_fn, OUTPUT).await;
                // reset again
                if let Some(reset_fn) = &mut self.reset_fn {
                    let (_reset_pass_result, algo_data) = run_async_pass_verbosely("  Resetting again: ", ";", None, reset_fn, OUTPUT).await;
                    Some(algo_data)     // return the "after second reset" data
                } else {
                    Some(algo_data)     // return the "after warmup" data
                }
            }
            None => algo_data  // return the "after first reset" data
        };

        // execute the 2 passes + any assertions
        // TODO: the custom measurements are missing from here -- see "test_run()" for more info
        let first_pass_fn = self.first_pass_fn.as_mut().expect("BUG! First pass function not present");
        let second_pass_fn = self.second_pass_fn.as_mut().expect("BUG! Second pass function not present");
        // pass 1
        let (pass1_result, algo_data) = run_async_pass_verbosely("  Pass 1: ", ";", algo_data, first_pass_fn, OUTPUT).await;
        // assertions on pass 1 data
        if let Some(ref mut first_pass_assertion_fn) = self.first_pass_assertion_fn {
            first_pass_assertion_fn(&algo_data).await;
        }
        // pass 2
        let (pass2_result, algo_data) = run_async_pass_verbosely("  Pass 2: ", "", Some(algo_data), second_pass_fn, OUTPUT).await;
        // assertions on pass 2 data
        if let Some(ref mut second_pass_assertion_fn) = self.second_pass_assertion_fn {
            second_pass_assertion_fn(&algo_data).await;
        }

        // analysis
        let measurements = AlgorithmMeasurements {
            measurement_name: self.test_name.as_str(),
            passes_info: AlgorithmPassesInfo {
                pass1_n: self.first_pass_n,
                pass2_n: self.second_pass_n,
            },
            time_measurements: BigOTimeMeasurements {
                pass_1_measurements: pass1_result.time_measurements,
                pass_2_measurements: pass2_result.time_measurements,
            },
            space_measurements: BigOSpaceMeasurements {
                pass_1_measurements: pass1_result.space_measurements,
                pass_2_measurements: pass2_result.space_measurements,
            },
            pass1_measurements: BigOPassMeasurements {
                time_measurements: pass1_result.time_measurements,
                space_measurements: Default::default(),
                custom_measurements: vec![],
            },
            pass2_measurements: BigOPassMeasurements {
                time_measurements: Default::default(),
                space_measurements: Default::default(),
                custom_measurements: vec![],
            },
        };
        let observed_time_complexity  = low_level_analysis::time_analysis::analyse_time_complexity(&measurements.passes_info, &measurements.time_measurements);
        let observed_space_complexity = low_level_analysis::space_analysis::analyse_space_complexity(&measurements.passes_info, &measurements.space_measurements);
        let algorithm_analysis = BigOAlgorithmAnalysis {
            time_complexity: observed_time_complexity,
            space_complexity: observed_space_complexity,
            algorithm_measurements: measurements,
        };

        OUTPUT("\n\n");
        OUTPUT(&format!("{}\n", algorithm_analysis));

        if let Some(expected_space_complexity) = self.expected_space_complexity {
            if observed_space_complexity as u32 > expected_space_complexity as u32 {
                let msg = format!("\n ** Aborted due to SPACE complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?}\n\n",
                                         self.test_name, expected_space_complexity, observed_space_complexity);
                OUTPUT(&msg);
                return RetryResult::Fatal { input: (self, algo_data), error: msg }
            }
        }

        if let Some(expected_time_complexity) = self.expected_time_complexity {
            if observed_time_complexity as u32 > expected_time_complexity as u32 {
                let msg = format!("\n ** TIME complexity mismatch on '{}' operation: maximum: {:?}, measured: {:?} -- a reattempt may be performed...\n\n",
                                         self.test_name, expected_time_complexity, observed_time_complexity);
                OUTPUT(&msg);
                return RetryResult::Transient { input: (self, algo_data), error: msg }
            }
        }

        RetryResult::Ok { reported_input: self, output: algo_data }

    }

    async fn test_run(mut self) {
        println!("## Wonderful!! We are ready to run.");
        println!("## This is the data we got:");
        println!("##   max_reattempts: {:?}", self.max_reattempts);
        println!("##   warmup_fn: {}", presence(&self.warmup_fn));
        println!("##   reset_fn: {}", presence(&self.reset_fn));

        fn presence<T>(val: &Option<T>) -> &'static str {
            val.as_ref().map(|_| "present").unwrap_or("absent")
        }

        println!("##");
        println!("## running:");

        /// macro to:
        /// 1) Run closures with no parameters as well as 1 parameter
        /// 2) Return an Option<ret> where ret is whatever the closure returns and None is returned if the closure is not present
        macro_rules! run_if {
            // Case for closures with no parameters
            ($opt_closure:expr) => {
                async {
                    if let Some(closure) = $opt_closure {
                        Some(closure().await)
                    } else {
                        None
                    }
                }
            };
            // Case for closures with one parameter
            ($opt_closure:expr, $param:expr) => {
                async {
                    if let Some(closure) = $opt_closure {
                        Some(closure($param).await)
                    } else {
                        None
                    }
                }
            };
        }

        let reset_optional_result = run_if!(&mut self.reset_fn, None).await;
        if reset_optional_result.is_some() {
            println!("## reset_fn is present and was executed before any passes. reset_optional_result: {reset_optional_result:?}");
        } else {
            println!("## not executing reset_fn as it is not present");
        }

        let warmup_optional_result = run_if!(&mut self.warmup_fn, reset_optional_result).await;
        if warmup_optional_result.is_some() {
            println!("## warmup_fn is present and was executed. warmup_optional_result: {warmup_optional_result:?}");
        } else {
            println!("## not executing warmup_fn as it is not present");
        }

        let reset_optional_result = run_if!(&mut self.reset_fn, warmup_optional_result).await;
        if reset_optional_result.is_some() {
            println!("## reset_fn is present and was executed after the warmup pass. reset_optional_result: {reset_optional_result:?}");
        } else {
            println!("## again, not executing reset_fn as it is not present");
        }

        println!("## taking pre-1st pass measurements -- in the reverse order they were declared");
        measure_all_before_event(reset_optional_result.as_ref(), &mut self.custom_measurers).await;

        let first_pass_result = run_if!(&mut self.first_pass_fn, reset_optional_result).await
            .expect("BUG!!! 1st pass function is not present! That should be impossible!");
        println!("## first_pass_fn was executed. first_pass_result: {first_pass_result:?}");

        println!("## taking post-1st pass delta measurements -- in the same order they were declared");
        let first_pass_measurements = {
            measure_all_after_event(Some(&first_pass_result), &mut self.custom_measurers).await
        };
        println!("## dumping 2nd pass measurements:");
        for custom_measurement in first_pass_measurements.iter() {
            println!("  ## {} = {}", custom_measurement.name, custom_measurement.measured_data);
        }
        self.first_pass_measurements.replace(first_pass_measurements);

        let first_pass_assertion_result = run_if!(&mut self.first_pass_assertion_fn, &first_pass_result).await;
        if first_pass_assertion_result.is_some() {
            println!("## first_pass_assertion_fn is present and was executed after the first pass.");
        } else {
            println!("## not executing first_pass_assertion_fn as it is not present");
        }

        let reset_optional_result = run_if!(&mut self.reset_fn, Some(first_pass_result)).await;
        if reset_optional_result.is_some() {
            println!("## reset_fn is present and was executed after the first pass. reset_optional_result: {reset_optional_result:?}");
        } else {
            println!("## again, not executing reset_fn as it is not present");
        }

        println!("## taking pre-2nd pass measurements -- in the reverse order they were declared");
        measure_all_before_event(reset_optional_result.as_ref(), &mut self.custom_measurers).await;

        let second_pass_result = run_if!(&mut self.second_pass_fn, reset_optional_result).await
            .expect("BUG!!! 2st pass function is not present! That should be impossible!");
        println!("## second_pass_fn was executed. second_pass_result: {second_pass_result:?}");

        println!("## taking post-2nd pass delta measurements -- in the same order they were declared");
        let second_pass_measurements = measure_all_after_event(Some(&second_pass_result), &mut self.custom_measurers).await;
        println!("## dumping 2nd pass measurements:");
        for custom_measurement in second_pass_measurements.iter() {
            println!("  ## {} = {}", custom_measurement.name, custom_measurement.measured_data);
        }
        self.second_pass_measurements.replace(second_pass_measurements);

        let second_pass_assertion_result = run_if!(&mut self.second_pass_assertion_fn, &second_pass_result).await;
        if second_pass_assertion_result.is_some() {
            println!("## second_pass_assertion_fn is present and was executed after the second pass.");
        } else {
            println!("## not executing second_pass_assertion_fn as it is not present");
        }

    }

}

impl<FirstPassFn:   FnMut(Option<AlgoDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=AlgoDataType> + Send,
     SecondPassFn:  FnMut(Option<AlgoDataType>) -> SecondPassFut + Send + Sync,
     SecondPassFut: Future<Output=AlgoDataType> + Send,
     AlgoDataType:  Send + Sync + Debug + 'static>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, SecondPassFn, SecondPassFut, AlgoDataType> {

    pub fn new(test_name: impl Into<String>) -> Self {
        Self {

            test_name: test_name.into(),

            reset_fn: None,

            warmup_fn: None,
            max_reattempts: None,

            first_pass_n: 0,
            first_pass_fn: None,
            first_pass_measurements: None,
            first_pass_assertion_fn: None,

            second_pass_n: 0,
            second_pass_fn: None,
            second_pass_measurements: None,
            second_pass_assertion_fn: None,

            expected_time_complexity: None,
            expected_space_complexity: None,
            auxiliary_space_measurement: None,

            custom_measurers: vec![],
        }
    }

    /// Max reattempts
    pub fn with_max_reattempts(mut self, attempts: u32) -> Self {
        self.max_reattempts = Some(attempts);
        self
    }

    /// The optional `reset_fn` is executed before any of the passes ([Self::warmup_pass()], [Self::first_pass()], [Self::second_pass()])
    /// and is intended as setting up (or cleaning) any data so the passes may be cleanly analysed.
    pub fn with_reset_fn<Fut: Future<Output=AlgoDataType> + Send + 'static>
                        (mut self,
                         reset_fn: impl Fn(Option<AlgoDataType>) -> Fut + Sync + Send + 'static)
                        -> Self {
        self.reset_fn.replace(Box::new(move |algo_data| Box::pin(reset_fn(algo_data))));
        self
    }

    /// Run the given `warmup_fn` so that the measurements are more stable between [Self::first_pass()] and [Self::second_pass()].
    /// By running the algorithm with a smaller set of data, things like populating caches and establishing connections
    /// gets out of the way when measuring time and space.
    /// Use it if your measurements are inconsistent between passes 1 and 2.
    pub fn warmup_pass<Fut: Future<Output=AlgoDataType> + Send + 'static>
                      (mut self,
                       mut warmup_fn: impl FnMut(Option<AlgoDataType>) -> Fut + Sync + Send + 'static)
                      -> Self {
        self.warmup_fn.replace(Box::new(move |algo_data| Box::pin(warmup_fn(algo_data))));
        self
    }

    /// Informs the Algorithms Analyser of the code to run on the "first pass".
    /// `first_pass_fn` must execute the same algorithm as [Self::second_pass()],
    /// but with a considerably lower `first_pass_n` -- ideally half.
    pub fn first_pass(mut self,
                      first_pass_n: u32,
                      first_pass_fn: FirstPassFn)
                     -> Self {
        self.first_pass_n = first_pass_n;
        self.first_pass_fn.replace(first_pass_fn);
        self
    }

    /// Optionally provide code to run after the [Self::first_pass()] is complete and all
    /// measurements are done. The intention is to assert on the `algo_data` generated,
    /// ensuring the first pass ran as expected.
    /// The function is designed to panic should any assertion fail.
    pub fn first_pass_assertion<Fut: Future<Output=()> + Send + 'static>
                               (mut self,
                                mut first_pass_assertion_fn: impl FnMut(&AlgoDataType) -> Fut + Sync + Send + 'static)
                               -> Self {
        self.first_pass_assertion_fn.replace(Box::new(move |algo_data| Box::pin(first_pass_assertion_fn(algo_data)) ));
        self
    }

    /// Informs the Algorithms Analyser of the code to run on the "second pass".
    /// `second_pass_fn` must execute the same algorithm as [Self::first_pass()],
    /// but with a considerably higher `second_pass_n` -- ideally the double.
    pub fn second_pass(mut self,
                       second_pass_n: u32,
                       second_pass_fn: SecondPassFn)
                      -> Self {
        self.second_pass_n = second_pass_n;
        self.second_pass_fn.replace(second_pass_fn);
        self
    }


    /// Optionally provide code to run after the [Self::second_pass()] is complete and all
    /// measurements are done. The intention is to assert on the `algo_data` generated,
    /// ensuring the second pass ran as expected.
    /// The function is designed to panic should any assertion fail.
    pub fn second_pass_assertion<Fut: Future<Output=()> + Send + 'static>
                                (mut self,
                                 mut second_pass_assertion_fn: impl FnMut(&AlgoDataType) -> Fut + Sync + Send + 'static)
                                -> Self {
        self.second_pass_assertion_fn.replace(Box::new(move |algo_data| Box::pin(second_pass_assertion_fn(algo_data)) ));
        self
    }

    pub fn with_time_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.expected_time_complexity = Some(measure);
        self
    }

    pub fn with_space_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.expected_space_complexity = Some(measure);
        self
    }

    pub fn with_auxiliary_space_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.auxiliary_space_measurement = Some(measure);
        self
    }

    pub fn add_custom_measurement<BeforeMeasurerOutput:                              Send + 'static,
                                  BeforeFut: Future<Output=BeforeMeasurerOutput>   + Send + 'static,
                                  AfterFut:  Future<Output=PresentableMeasurement> + Send + 'static>
                                 (mut self,
                                  name: impl Into<String>,
                                  expected_complexity: BigOAlgorithmComplexity,
                                  description: impl Into<String>,
                                  before_event_measurer_fn: impl FnMut(Option<&AlgoDataType>) -> BeforeFut + Send + Sync + 'static,
                                  after_event_measurer_fn:  impl FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync + 'static)
                                 -> Self {
        let measurer_executor = Box::new(CustomMeasurer::new(name, expected_complexity, description, before_event_measurer_fn, after_event_measurer_fn));
        self.custom_measurers.push(measurer_executor);
        self
    }

    pub fn add_custom_measurement_with_averages<BeforeMeasurerOutput:                              Send + 'static,
                                                BeforeFut: Future<Output=BeforeMeasurerOutput>   + Send + 'static,
                                                AfterFut:  Future<Output=PresentableMeasurement> + Send + 'static>
                                               (mut self,
                                                name: &str,
                                                expected_complexity: BigOAlgorithmComplexity,
                                                description: &str,
                                                before_event_measurer_fn: impl FnMut(Option<&AlgoDataType>) -> BeforeFut + Send + Sync + 'static,
                                                after_event_measurer_fn:  impl FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync + 'static)
                                               -> Self {
        let measurer_executor = Box::new(CustomMeasurer::new(name, expected_complexity, description, before_event_measurer_fn, after_event_measurer_fn));
        self.custom_measurers.push(measurer_executor);
        self
    }
}


#[cfg(test)]
mod tests {
    use std::future;
    use std::time::{Duration, Instant};
    use crate::utils::measurements;
    use super::*;

    #[tokio::test]
    async fn full_options() {
        let s = RegularAsyncAnalyzerBuilder::new("full options")
            .with_reset_fn(|previous_pass_input| async move {
                println!("Reset received input {previous_pass_input:?} -- and this is the Option<PassDataType>. We are returning ''");
                -1
            })
            .warmup_pass(|reset_outcome| async move {
                println!("Warmup received the reset_fn outcome as input: {reset_outcome:?} -- and this is the ResetDataType. We are returning 1");
                0
            })
            .with_max_reattempts(2)
            .first_pass(100, |_reset_outcome| async { 1 })
            .first_pass_assertion(|&algo_data| async move {
                assert_eq!(algo_data, 1, "Unexpected data was generated in the 1st pass");
            })
            .second_pass(100, |_reset_outcome| async { 2 })
            .second_pass_assertion(|&algo_data| async move {
                assert_eq!(algo_data, 2, "Unexpected data was generated in the 2nd pass");
            })
            .add_custom_measurement("Δt", BigOAlgorithmComplexity::O1, "Elapsed Time",
                                    |_algo_data| future::ready(Instant::now()),
                                    |_algo_data, instant| future::ready(measurements::presentable_measurements::duration_measurement(instant.elapsed())));
        s.test_run().await;
    }

    #[tokio::test]
    async fn minimum_options() {
        let s = RegularAsyncAnalyzerBuilder::new("minimum_options")
            .first_pass(100, |_: Option<()>| async { () })
            .second_pass(100, |_: Option<()>| async { () });
        s.test_run().await;
    }

    #[tokio::test]
    async fn raw_analyse_algorithm() {
        let result = RegularAsyncAnalyzerBuilder::new("dummy analysis")
            .first_pass(10, |_: Option<()>| tokio::time::sleep(Duration::from_millis(100)))
            .second_pass(20, |_: Option<()>| tokio::time::sleep(Duration::from_millis(200)))
            .raw_analyse_algorithm(None).await;
        result.expect_ok("algorithm analysis ended with non-ok status");
    }

    #[tokio::test]
    async fn test_algorithm_retrying_once() {
        let sleep_sequence = [10, 20, 0, 0];

        RegularAsyncAnalyzerBuilder::new("dummy analysis")
            .with_reset_fn(|sleep_index| future::ready(sleep_index.unwrap_or(0)))
            .first_pass(100, |sleep_index| async move {
                let sleep_index = sleep_index.expect("BUG! No `sleep_index`!");
                let duration = Duration::from_millis(sleep_sequence[sleep_index]);
                tokio::time::sleep(duration).await;
                sleep_index+1
            })
            .second_pass(200, |sleep_index| async move {
                let sleep_index = sleep_index.expect("BUG! No `sleep_index`!");
                let duration = Duration::from_millis(sleep_sequence[sleep_index]);
                tokio::time::sleep(duration).await;
                sleep_index+1
            })
            .with_time_measurements(BigOAlgorithmComplexity::O1)
            .with_max_reattempts(1) // retry up to 1 time
            .test_algorithm().await;
    }

}