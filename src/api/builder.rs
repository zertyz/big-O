//! Defines our main API (using the Builder Pattern) & related internal model

// big_o::analyse_regular_async_algorithm()
//     .with_warmup(|| async {...})
//     .with_max_reattempts_per_pass(2)
//     .with_reset_fn(|| async {...})
//     .first_pass(n_elements, |n_elements| async {...} -> data)
//     .first_pass_assertions(|data| async {...})
//     .second_pass(n_elements, |n_elements| async {...} -> data)
//     .second_pass_assertions(|data| async {...})
//     .with_time_measurements(BigOThings::On)
//     .with_space_measurements(BigOThings::O1)
//     .with_auxiliary_space_measurements(BigOThings::On)
//     .add_custom_measurement("Δconn", BigOThing::O1, "total connections opened", ValueRepresentation::Unit, |data| ... -> val)
//     .add_custom_measurement_with_averages("Δcalls", BigOThing::O1, "total external service calls made", ValueRepresentation::Scientific, |data| ... -> val)
//     .run().await;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::marker::PhantomData;
use crate::BigOAlgorithmComplexity;

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
pub struct RegularAsyncAnalyzerBuilder<FirstPassFn:   Fn(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
                                       FirstPassFut:  Future<Output=PassDataType> + Send,
                                       ResetDataType: Debug,
                                       PassDataType:  Debug> {
    max_reattempts: Option<u32>,
    warmup_fn: Option<Box<dyn Fn(Option<ResetDataType>) -> Pin<Box<dyn Future<Output=PassDataType> + Send>> + Send + Sync>>,
    reset_fn: Option<Box<dyn Fn(Option<PassDataType>) -> Pin<Box<dyn Future<Output=ResetDataType> + Send>> + Send + Sync>>,

    first_pass_n: u64,
    first_pass_fn: Option<FirstPassFn>,
    first_pass_assertion: Option<Box<dyn Fn(Option<ResetDataType>) -> Pin<Box<dyn Future<Output=PassDataType> + Send>> + Send + Sync>>,

    second_pass_n: Option<u64>,
    second_pass_closure: Option<Box<dyn Fn(Option<ResetDataType>) -> Pin<Box<dyn Future<Output=PassDataType> + Send>> + Send + Sync>>,
    second_pass_assertion: Option<()>,

    time_measurement: Option<BigOAlgorithmComplexity>,
    space_measurement: Option<BigOAlgorithmComplexity>,
    auxiliary_space_measurement: Option<BigOAlgorithmComplexity>,

    custom_measurements: Vec<CustomMeasurement<Box<dyn Fn(&PassDataType) -> Pin<Box<dyn Future<Output=f64> + Send>> + Send + Sync>> >,
}

// For convenience, define a helper type for an async closure signature:
//   “Fn() -> Future<Output=()>” is tricky to store directly, so we rely on
//   pinned boxes or trait objects. This is just an example.
type AsyncVoidFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>;
type AsyncDataFn<T> = Box<dyn Fn(T) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>;


impl<FirstPassFn:   Fn(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=PassDataType> + Send,
     ResetDataType: Debug,
     PassDataType: Debug>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, ResetDataType, PassDataType> {

    pub async fn run(self) {
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

        let reset_optional_result = run_if!(&self.reset_fn, None).await;
        if reset_optional_result.is_some() {
            println!("## reset_fn is present and was executed before any passes. reset_optional_result: {reset_optional_result:?}");
        } else {
            println!("## not executing reset_fn as it is not present");
        };

        let warmup_optional_result = run_if!(&self.warmup_fn, reset_optional_result).await;
        if warmup_optional_result.is_some() {
            println!("## warmup_fn is present and was executed. warmup_optional_result: {warmup_optional_result:?}");
        } else {
            println!("## not executing warmup_fn as it is not present");
        };

        let reset_optional_result = run_if!(&self.reset_fn, warmup_optional_result).await;
        if reset_optional_result.is_some() {
            println!("## reset_fn is present and was executed after the warmup pass. reset_optional_result: {reset_optional_result:?}");
        } else {
            println!("## again, not executing reset_fn as it is not present");
        };

        let first_pass_result = run_if!(&self.first_pass_fn, reset_optional_result).await
            .expect("BUG!!! 1st pass function is not present! That should be impossible!");
        println!("## first_pass_fn was executed. first_pass_result: {first_pass_result:?}");

    }

}

impl<FirstPassFn:   Fn(Option<ResetDataType>) -> FirstPassFut + Send + Sync,
     FirstPassFut:  Future<Output=PassDataType> + Send,
     ResetDataType: Debug,
     PassDataType: Debug>
RegularAsyncAnalyzerBuilder<FirstPassFn, FirstPassFut, ResetDataType, PassDataType> {

    pub fn new() -> Self {
        Self {
            warmup_fn: None,
            max_reattempts: None,
            reset_fn: None,

            first_pass_n: 0,
            first_pass_fn: None,
            first_pass_assertion: None,

            second_pass_n: None,
            second_pass_closure: None,
            second_pass_assertion: None,

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

    /// Warmup
    pub fn warmup_pass<Fut: Future<Output=PassDataType> + Send + 'static>
                      (mut self,
                       warmup_fn: impl Fn(Option<ResetDataType>) -> Fut + Sync + Send + 'static)
                      -> Self {
        self.warmup_fn.replace(Box::new(move |reset_data| Box::pin(warmup_fn(reset_data))));
        self
    }

    /// Reset function
    pub fn with_reset_fn<Fut: Future<Output=ResetDataType> + Send + 'static>
                        (mut self,
                         reset_fn: impl Fn(Option<PassDataType>) -> Fut + Sync + Send + 'static)
                        -> Self {
        self.reset_fn.replace(Box::new(move |pass_data| Box::pin(reset_fn(pass_data))));
        self
    }

    pub fn first_pass(mut self,
                      first_pass_n: u64,
                      first_pass_fn: FirstPassFn)
                     -> Self {
        self.first_pass_n = first_pass_n;
        self.first_pass_fn.replace(first_pass_fn);
        self
    }


/*
    /// First pass
    pub fn first_pass<F>(
        mut self,
        n_elements: u64,
        closure: F
    ) -> RegularAsyncAnalyzerBuilder<W, R, F, FA1, FP2, FA2>
    where
        F: Fn(u64) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync + 'static,
        // Suppose the pass closure returns some data (String here, just as an example).
        // Adjust to your real data type as needed.
    {
        self.first_pass_n = Some(n_elements);
        self.first_pass_closure = Some(closure);
        RegularAsyncAnalyzerBuilder {
            warmup_fn: self.warmup_fn,
            max_reattempts: self.max_reattempts,
            reset_fn: self.reset_fn,

            first_pass_n: self.first_pass_n,
            first_pass_closure: self.first_pass_closure,
            first_pass_assertion: self.first_pass_assertion,

            second_pass_n: self.second_pass_n,
            second_pass_closure: self.second_pass_closure,
            second_pass_assertion: self.second_pass_assertion,

            time_measurement: self.time_measurement,
            space_measurement: self.space_measurement,
            auxiliary_space_measurement: self.auxiliary_space_measurement,

            custom_measurements: self.custom_measurements,
            _phantom: PhantomData,
        }
    }

    /// First pass assertions
    pub fn first_pass_assertions<F>(
        mut self,
        assertion_fn: F
    ) -> RegularAsyncAnalyzerBuilder<W, R, FP1, F, FP2, FA2>
    where
        F: Fn(&String) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync + 'static
    {
        self.first_pass_assertion = Some(assertion_fn);
        RegularAsyncAnalyzerBuilder {
            warmup_fn: self.warmup_fn,
            max_reattempts: self.max_reattempts,
            reset_fn: self.reset_fn,

            first_pass_n: self.first_pass_n,
            first_pass_closure: self.first_pass_closure,
            first_pass_assertion: self.first_pass_assertion,

            second_pass_n: self.second_pass_n,
            second_pass_closure: self.second_pass_closure,
            second_pass_assertion: self.second_pass_assertion,

            time_measurement: self.time_measurement,
            space_measurement: self.space_measurement,
            auxiliary_space_measurement: self.auxiliary_space_measurement,

            custom_measurements: self.custom_measurements,
            _phantom: PhantomData,
        }
    }

    /// Second pass (similar to first pass)
    pub fn second_pass<F>(
        mut self,
        n_elements: u64,
        closure: F
    ) -> RegularAsyncAnalyzerBuilder<W, R, FP1, FA1, F, FA2>
    where
        F: Fn(u64) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync + 'static,
    {
        self.second_pass_n = Some(n_elements);
        self.second_pass_closure = Some(closure);
        RegularAsyncAnalyzerBuilder {
            warmup_fn: self.warmup_fn,
            max_reattempts: self.max_reattempts,
            reset_fn: self.reset_fn,

            first_pass_n: self.first_pass_n,
            first_pass_closure: self.first_pass_closure,
            first_pass_assertion: self.first_pass_assertion,

            second_pass_n: self.second_pass_n,
            second_pass_closure: self.second_pass_closure,
            second_pass_assertion: self.second_pass_assertion,

            time_measurement: self.time_measurement,
            space_measurement: self.space_measurement,
            auxiliary_space_measurement: self.auxiliary_space_measurement,

            custom_measurements: self.custom_measurements,
            _phantom: PhantomData,
        }
    }

    /// Second pass assertions
    pub fn second_pass_assertions<F>(
        mut self,
        assertion_fn: F
    ) -> RegularAsyncAnalyzerBuilder<W, R, FP1, FA1, FP2, F>
    where
        F: Fn(&String) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync + 'static
    {
        self.second_pass_assertion = Some(assertion_fn);
        RegularAsyncAnalyzerBuilder {
            warmup_fn: self.warmup_fn,
            max_reattempts: self.max_reattempts,
            reset_fn: self.reset_fn,

            first_pass_n: self.first_pass_n,
            first_pass_closure: self.first_pass_closure,
            first_pass_assertion: self.first_pass_assertion,

            second_pass_n: self.second_pass_n,
            second_pass_closure: self.second_pass_closure,
            second_pass_assertion: self.second_pass_assertion,

            time_measurement: self.time_measurement,
            space_measurement: self.space_measurement,
            auxiliary_space_measurement: self.auxiliary_space_measurement,

            custom_measurements: self.custom_measurements,
            _phantom: PhantomData,
        }
    }

    // Measurements toggles
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

    // Add custom measurement (example)
    pub fn add_custom_measurement<FN>(
        mut self,
        name: &str,
        expected: BigOAlgorithmComplexity,
        desc: &str,
        representation: ValueRepresentation,
        measure_fn: FN
    ) -> Self
    where
        FN: Fn(&String) -> f64 + Send + Sync + 'static,
        // Example: given the result data from pass, returns a f64 measurement
    {
        self.custom_measurements.push(CustomMeasurement {
            name: name.to_string(),
            expected_complexity: expected,
            description: desc.to_string(),
            representation,
            measure_fn: Box::new(measure_fn),
        });
        self
    }

    // Variation that does something else with averages, etc.
    pub fn add_custom_measurement_with_averages<FN>(
        mut self,
        name: &str,
        expected: BigOAlgorithmComplexity,
        desc: &str,
        representation: ValueRepresentation,
        measure_fn: FN
    ) -> Self
    where
        FN: Fn(&String) -> f64 + Send + Sync + 'static,
    {
        // Implementation detail...
        self.custom_measurements.push(CustomMeasurement {
            name: name.to_string(),
            expected_complexity: expected,
            description: desc.to_string(),
            representation,
            measure_fn: Box::new(measure_fn),
        });
        self
    }

    // The final "build" or "run" method. In real code, you'll want to do
    // a bit more error-checking or ensure that all needed closures are Some(_).
    pub async fn run(self) {
        // 1) Possibly call warmup
        if let Some(warmup) = self.warmup_fn {
            (warmup)().await;
        }

        // 2) for each pass, do reset if needed, run the pass closure, measure times/spaces
        //    call the assertion closures, etc. Example pseudo-code:

        if let Some(reset) = self.reset_fn {
            (reset)().await;
        }

        let mut pass1_data = None;
        if let Some(pass1_closure) = self.first_pass_closure {
            // measure times, run closure:
            pass1_data = Some((pass1_closure)(self.first_pass_n.unwrap()).await);
        }
        if let Some(assertion) = self.first_pass_assertion {
            if let Some(ref data) = pass1_data {
                assertion(data).await;
            }
        }

        // second pass
        if let Some(reset) = self.reset_fn {
            (reset)().await;
        }

        let mut pass2_data = None;
        if let Some(pass2_closure) = self.second_pass_closure {
            pass2_data = Some((pass2_closure)(self.second_pass_n.unwrap()).await);
        }
        if let Some(assertion) = self.second_pass_assertion {
            if let Some(ref data) = pass2_data {
                assertion(data).await;
            }
        }

        // 3) custom measurements
        //    e.g. pass data into measure_fn, collect results

        if let Some(ref data) = pass1_data {
            for cm in self.custom_measurements.iter() {
                let val = (cm.measure_fn)(data);
                println!("Custom measurement {}: {}", cm.name, val);
            }
        }

        // 4) Summarize time measurement, space measurement, etc.

        // done
        println!("Analysis complete!");
    }*/
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let s = RegularAsyncAnalyzerBuilder::new()
            .warmup_pass(|reset_outcome| async move {
                println!("Warmup received the reset_fn outcome as input: {reset_outcome:?} -- and this is the ResetDataType. We are returning 1");
                1
            })
            .with_max_reattempts_per_pass(2)
            .with_reset_fn(|previous_pass_input| async move {
                println!("Reset received input {previous_pass_input:?} -- and this is the Option<PassDataType>. We are returning ''");
                'c'
            })
            .first_pass(100, |reset_outcome| async {2});
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
