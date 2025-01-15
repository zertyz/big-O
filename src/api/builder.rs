//! Defines our main API (using the Builder Pattern) & related internal model

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use crate::BigOAlgorithmComplexity;
use crate::utils::measurements::PresentableMeasurement;

/// TODO: really needed?
#[derive(Debug)]
pub enum ValueRepresentation {
    Unit,
    Scientific,
    // ...
}

// Could store additional info about a measurement
pub struct CustomMeasurement<MeasureFn> {
    pub name: String,
    pub expected_complexity: BigOAlgorithmComplexity,
    pub description: String,
    pub representation: ValueRepresentation,
    pub measure_fn: MeasureFn,
}

// This struct accumulates all the config for the analysis
pub struct RegularAsyncAnalyzerBuilder<FirstPassFn:   FnMut(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
                                       FirstPassFut:  Future<Output=PassDataType> + Send,
                                       SecondPassFn:  FnMut(Option<ResetDataType>) -> SecondPassFut + Send + Sync,
                                       SecondPassFut: Future<Output=PassDataType> + Send,
                                       ResetDataType: Debug,
                                       PassDataType:  Debug> {

    reset_fn: Option<Box<dyn FnMut(Option<PassDataType>) -> Pin<Box<dyn Future<Output=ResetDataType> + Send>> + Send + Sync>>,

    max_reattempts: Option<u32>,
    warmup_fn: Option<Box<dyn FnMut(Option<ResetDataType>) -> Pin<Box<dyn Future<Output=PassDataType> + Send>> + Send + Sync>>,

    first_pass_n: u64,
    first_pass_fn: Option<FirstPassFn>,
    first_pass_measurements: Option<Vec<PresentableMeasurement<0>>>,
    first_pass_assertion_fn: Option<Box<dyn FnMut(&PassDataType) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>>,

    second_pass_n: u64,
    second_pass_fn: Option<SecondPassFn>,
    second_pass_measurements: Option<Vec<PresentableMeasurement<0>>>,
    second_pass_assertion_fn: Option<Box<dyn FnMut(&PassDataType) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>>,

    time_measurement: Option<BigOAlgorithmComplexity>,
    space_measurement: Option<BigOAlgorithmComplexity>,
    auxiliary_space_measurement: Option<BigOAlgorithmComplexity>,

    /// Measurements are done in a "delta" fashion.
    /// For details, see [Self::add_custom_measurement()].
    custom_measurements: Vec<CustomMeasurement<Box<dyn FnMut(&PassDataType) -> Pin<Box<dyn Future<Output=PresentableMeasurement<0>> + Send>> + Send + Sync>> >,
}


impl<FirstPassFn:   FnMut(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=PassDataType> + Send,
     SecondPassFn:  FnMut(Option<ResetDataType>) -> SecondPassFut + Send + Sync,
     SecondPassFut: Future<Output=PassDataType> + Send,
     ResetDataType: Debug,
     PassDataType:  Debug>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, SecondPassFn, SecondPassFut, ResetDataType, PassDataType> {

    pub async fn run(mut self) {
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

        let first_pass_result = run_if!(&mut self.first_pass_fn, reset_optional_result).await
            .expect("BUG!!! 1st pass function is not present! That should be impossible!");
        println!("## first_pass_fn was executed. first_pass_result: {first_pass_result:?}");

println!("## taking post-1st pass measurements -- in the same order they were declared");

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

        let second_pass_result = run_if!(&mut self.second_pass_fn, reset_optional_result).await
            .expect("BUG!!! 2st pass function is not present! That should be impossible!");
        println!("## second_pass_fn was executed. second_pass_result: {second_pass_result:?}");

println!("## taking post-2nd pass measurements -- in the same order they were declared");

        let second_pass_assertion_result = run_if!(&mut self.second_pass_assertion_fn, &second_pass_result).await;
        if second_pass_assertion_result.is_some() {
            println!("## second_pass_assertion_fn is present and was executed after the second pass.");
        } else {
            println!("## not executing second_pass_assertion_fn as it is not present");
        }

    }

}

impl<FirstPassFn:   FnMut(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=PassDataType> + Send,
     SecondPassFn:  FnMut(Option<ResetDataType>) -> SecondPassFut + Send + Sync,
     SecondPassFut: Future<Output=PassDataType> + Send,
     ResetDataType: Debug,
     PassDataType:  Debug>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, SecondPassFn, SecondPassFut, ResetDataType, PassDataType> {

    pub fn new() -> Self {
        Self {
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

            time_measurement: None,
            space_measurement: None,
            auxiliary_space_measurement: None,

            custom_measurements: vec![],
        }
    }

    /// Max reattempts
    pub fn with_max_reattempts_per_pass(mut self, attempts: u32) -> Self {
        self.max_reattempts = Some(attempts);
        self
    }

    /// The optional `reset_fn` is executed before any of the passes ([Self::warmup_pass()], [Self::first_pass()], [Self::second_pass()])
    /// and is intended as setting up (or cleaning) any data so the passes may be cleanly analysed.
    pub fn with_reset_fn<Fut: Future<Output=ResetDataType> + Send + 'static>
                        (mut self,
                         reset_fn: impl Fn(Option<PassDataType>) -> Fut + Sync + Send + 'static)
                        -> Self {
        self.reset_fn.replace(Box::new(move |pass_data| Box::pin(reset_fn(pass_data))));
        self
    }

    /// Run the given `warmup_fn` so that the measurements are more stable between [Self::first_pass()] and [Self::second_pass()].
    /// By running the algorithm with a smaller set of data, things like populating caches and establishing connections
    /// gets out of the way when measuring time and space.
    /// Use it if your measurements are inconsistent between passes 1 and 2.
    pub fn warmup_pass<Fut: Future<Output=PassDataType> + Send + 'static>
                      (mut self,
                       mut warmup_fn: impl FnMut(Option<ResetDataType>) -> Fut + Sync + Send + 'static)
                      -> Self {
        self.warmup_fn.replace(Box::new(move |reset_data| Box::pin(warmup_fn(reset_data))));
        self
    }

    /// Informs the Algorithms Analyser of the code to run on the "first pass".
    /// `first_pass_fn` must execute the same algorithm as [Self::second_pass()],
    /// but with a considerably lower `first_pass_n` -- ideally half.
    pub fn first_pass(mut self,
                      first_pass_n: u64,
                      first_pass_fn: FirstPassFn)
                     -> Self {
        self.first_pass_n = first_pass_n;
        self.first_pass_fn.replace(first_pass_fn);
        self
    }

    /// Optionally provide code to run after the [Self::first_pass()] is complete and all
    /// measurements are done. The intention is to assert on the `first_pass_data` generated,
    /// ensuring the first pass ran as expected.
    /// The function is designed to panic should any assertion fail.
    pub fn first_pass_assertion<Fut: Future<Output=()> + Send + 'static>
                               (mut self,
                                mut first_pass_assertion_fn: impl FnMut(&PassDataType) -> Fut + Sync + Send + 'static)
                               -> Self {
        self.first_pass_assertion_fn.replace(Box::new(move |first_pass_data| Box::pin(first_pass_assertion_fn(first_pass_data)) ));
        self
    }

    /// Informs the Algorithms Analyser of the code to run on the "second pass".
    /// `second_pass_fn` must execute the same algorithm as [Self::first_pass()],
    /// but with a considerably higher `second_pass_n` -- ideally the double.
    pub fn second_pass(mut self,
                       second_pass_n: u64,
                       second_pass_fn: SecondPassFn)
                      -> Self {
        self.second_pass_n = second_pass_n;
        self.second_pass_fn.replace(second_pass_fn);
        self
    }


    /// Optionally provide code to run after the [Self::second_pass()] is complete and all
    /// measurements are done. The intention is to assert on the `second_pass_data` generated,
    /// ensuring the second pass ran as expected.
    /// The function is designed to panic should any assertion fail.
    pub fn second_pass_assertion<Fut: Future<Output=()> + Send + 'static>
                                (mut self,
                                 mut second_pass_assertion_fn: impl FnMut(&PassDataType) -> Fut + Sync + Send + 'static)
                                -> Self {
        self.second_pass_assertion_fn.replace(Box::new(move |second_pass_data| Box::pin(second_pass_assertion_fn(second_pass_data)) ));
        self
    }

    pub fn with_time_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.time_measurement = Some(measure);
        self
    }

    pub fn with_space_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.space_measurement = Some(measure);
        self
    }

    pub fn with_auxiliary_space_measurements(mut self, measure: BigOAlgorithmComplexity) -> Self {
        self.auxiliary_space_measurement = Some(measure);
        self
    }

    pub fn add_custom_measurement<Fut: Future<Output=PresentableMeasurement<0>> + Send + 'static>
                                 (mut self,
                                  name: &str,
                                  expected: BigOAlgorithmComplexity,
                                  desc: &str,
                                  representation: ValueRepresentation,
                                  mut measure_fn: impl FnMut(&PassDataType) -> Fut + Sync + Send + 'static)
                                 -> Self {
        self.custom_measurements.push(CustomMeasurement {
            name: name.to_string(),
            expected_complexity: expected,
            description: desc.to_string(),
            representation,
            measure_fn: Box::new(move |pass_data| Box::pin(measure_fn(pass_data))),
        });
        self
    }

    pub fn add_custom_measurement_with_averages<Fut: Future<Output=PresentableMeasurement<0>> + Send + 'static>
                                               (mut self,
                                                name: &str,
                                                expected: BigOAlgorithmComplexity,
                                                desc: &str,
                                                representation: ValueRepresentation,
                                                mut measure_fn: impl FnMut(&PassDataType) -> Fut + Sync + Send + 'static)
                                               -> Self {
        self.custom_measurements.push(CustomMeasurement {
            name: name.to_string(),
            expected_complexity: expected,
            description: desc.to_string(),
            representation,
            measure_fn: Box::new(move |pass_data| Box::pin(measure_fn(pass_data))),
        });
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn full_options() {
        let s = RegularAsyncAnalyzerBuilder::new()
            .with_reset_fn(|previous_pass_input| async move {
                println!("Reset received input {previous_pass_input:?} -- and this is the Option<PassDataType>. We are returning ''");
                'c'
            })
            .warmup_pass(|reset_outcome| async move {
                println!("Warmup received the reset_fn outcome as input: {reset_outcome:?} -- and this is the ResetDataType. We are returning 1");
                0
            })
            .with_max_reattempts_per_pass(2)
            .first_pass(100, |_reset_outcome| async { 1 })
            .first_pass_assertion(|&first_pass_data| async move {
                assert_eq!(first_pass_data, 1, "Unexpected data was generated in the 1st pass");
            })
            .second_pass(100, |_reset_outcome| async { 2 })
            .second_pass_assertion(|&second_pass_data| async move {
                assert_eq!(second_pass_data, 2, "Unexpected data was generated in the 2nd pass");
            });
        s.run().await;
    }

    #[tokio::test]
    async fn minimum_options() {
        let s = RegularAsyncAnalyzerBuilder::new()
            .first_pass(100, |_: Option<()>| async { 1 })
            .second_pass(100, |_: Option<()>| async { 2 });
        s.run().await;
    }

}

/*#[allow(dead_code)]
pub async fn example_usage() {
    big_o_test()
        .with_warmup(|| Box::pin(async {
            // do some warmup
        }))
        .with_max_reattempts_per_pass(2)
        .with_reset_fn(|| Box::pin(async {
            // clear caches, reset environment, etc
        }))
        .first_pass(40000000, |n| Box::pin(async move {
            // do something with n...
            // measure data...
            "pass1-data".to_string()
        }))
        .first_pass_assertions(|data| Box::pin(async move {
            // check correctness, etc. possibly panic or return error
            println!("first pass data: {}", data);
        }))
        .second_pass(80000000, |n| Box::pin(async move {
            // do something with n...
            "pass2-data".to_string()
        }))
        .second_pass_assertions(|data| Box::pin(async move {
            println!("second pass data: {}", data);
        }))
        .with_time_measurements(BigOAlgorithmComplexity::ON)
        .with_space_measurements(BigOAlgorithmComplexity::O1)
        .with_auxiliary_space_measurements(BigOAlgorithmComplexity::ON)
        .add_custom_measurement(
            "Δconn",
            BigOAlgorithmComplexity::O1,
            "total connections opened",
            ValueRepresentation::Unit,
            |data| {
                // interpret data & produce a measurement
                10.0
            }
        )
        .add_custom_measurement_with_averages(
            "Δcalls",
            BigOAlgorithmComplexity::O1,
            "total external service calls made",
            ValueRepresentation::Scientific,
            |data| {
                123456.789
            }
        )
        .run()
        .await;
}*/
