//! Contains functionalities related to performing measurements

use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use crate::BigOAlgorithmComplexity;
use crate::utils::measurements::presentable_measurements::PresentableMeasurement;


/// Executes the "pre-event" steps of the measurements described by `measurements_details`.\
/// See [CustomMeasurer] for more info.
pub async fn measure_all_before_event<AlgoDataType: Send + Sync + Debug>
                                     (algo_data:                 Option<&AlgoDataType>,
                                      measurements_executors:    &mut Vec<Box<dyn CustomMeasurerExecutor<AlgoDataType>>>) {
    // measure in reversed order
    for measurement_executors in measurements_executors.iter_mut().rev() {
        measurement_executors.measure_before_event(algo_data).await;
    }
}

/// Executes the "post-event" steps of the measurements described by `measurements_details`.\
/// See [CustomMeasurer] for more info.
pub async fn measure_all_after_event<AlgoDataType: Send + Sync + Debug>
                                    (algo_data:                 Option<&AlgoDataType>,
                                     measurements_executors:    &mut Vec<Box<dyn CustomMeasurerExecutor<AlgoDataType>>>)
                                     -> Vec<CustomMeasurement> {
    let mut measurements = vec![];
    for measurement_executors in measurements_executors.iter_mut() {
        let measurement = measurement_executors.measure_after_event(algo_data).await;
        measurements.push(measurement_executors.as_custom_measurement(measurement));
    }
    measurements
}

/// Trait describing how to execute custom measurements.
/// This exists to allow storing different instantiations of [CustomMeasurer] in a single vector
pub trait CustomMeasurerExecutor<AlgoDataType: Send + Debug> {
    fn measure_before_event<'a>(&'a mut self,
                                algo_data: Option<&'a AlgoDataType>)
                               -> Pin<Box<dyn Future<Output=()> + Send + 'a>>;
    fn measure_after_event<'a>(&'a mut self,
                               algo_data: Option<&'a AlgoDataType>)
                              -> Pin<Box<dyn Future<Output=PresentableMeasurement> + Send + 'a>>;

    fn as_custom_measurement(&self, after_event_measurement: PresentableMeasurement)
                            -> CustomMeasurement;
}

/// Our domain-specific measured data -- to be used for asserting the algorithm complexity and reporting details
pub struct CustomMeasurement {
    pub name: String,
    pub expected_complexity: BigOAlgorithmComplexity,
    pub description: String,
    pub measured_data: PresentableMeasurement,
}

/// Contains the definitions for a measurer that is performed
/// through the provided asynchronous closures.
/// Measurements are done in 2 steps:
/// 1) A "pre-event" closure is executed to collect information. It may return any type;
/// 2) The second, "post-event" closure receives the returned value from the above and, finally, yields a [PresentableMeasurement]
pub struct CustomMeasurer<BeforeMeasurerOutput:                                                             Send,
                          BeforeFut:       Future<Output=BeforeMeasurerOutput>                            + Send,
                          MeasureBeforeFn: FnMut(Option<&AlgoDataType>) -> BeforeFut                      + Send + Sync,
                          AfterFut:        Future<Output=PresentableMeasurement>                          + Send,
                          MeasureAfterFn:  FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync,
                          AlgoDataType:                                                                     Send + Debug> {
    name: String,
    expected_complexity: BigOAlgorithmComplexity,
    description: String,
    before_event_measurer_fn: MeasureBeforeFn,
    before_event_measurement: Option<BeforeMeasurerOutput>,
    after_event_measurer_fn:  MeasureAfterFn,
    _phantom: PhantomData<(BeforeFut, AfterFut, AlgoDataType)>,
}

impl<BeforeMeasurerOutput:                                                             Send,
     BeforeFut:       Future<Output=BeforeMeasurerOutput>                            + Send,
     MeasureBeforeFn: FnMut(Option<&AlgoDataType>) -> BeforeFut                      + Send + Sync,
     AfterFut:        Future<Output=PresentableMeasurement>                          + Send,
     MeasureAfterFn:  FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync,
     AlgoDataType:                                                                     Send + Debug>
CustomMeasurer<BeforeMeasurerOutput,
               BeforeFut,
               MeasureBeforeFn,
               AfterFut,
               MeasureAfterFn,
               AlgoDataType> {
    pub fn new(name: impl Into<String>,
               expected_complexity: BigOAlgorithmComplexity,
               description: impl Into<String>,
               before_event_measurer_fn: MeasureBeforeFn,
               after_event_measurer_fn: MeasureAfterFn)
           -> Self {
        Self {
            name: name.into(),
            expected_complexity,
            description: description.into(),
            before_event_measurer_fn,
            before_event_measurement: None,
            after_event_measurer_fn,
            _phantom: Default::default(),
        }
    }
}

impl<BeforeMeasurerOutput:                                                            Send,
    BeforeFut:       Future<Output=BeforeMeasurerOutput>                            + Send,
    MeasureBeforeFn: FnMut(Option<&AlgoDataType>) -> BeforeFut                      + Send + Sync,
    AfterFut:        Future<Output=PresentableMeasurement>                          + Send,
    MeasureAfterFn:  FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync,
    AlgoDataType:                                                                     Send + Sync + Debug>
CustomMeasurerExecutor<AlgoDataType> for
CustomMeasurer<BeforeMeasurerOutput,
               BeforeFut,
               MeasureBeforeFn,
               AfterFut,
               MeasureAfterFn,
               AlgoDataType> {

    fn measure_before_event<'a>(&'a mut self,
                                algo_data: Option<&'a AlgoDataType>)
                               -> Pin<Box<dyn Future<Output=()> + Send + 'a>> {
        Box::pin(async move {
            let before_event_measurement = (self.before_event_measurer_fn)(algo_data).await;
            self.before_event_measurement.replace(before_event_measurement);
        })
    }

    fn measure_after_event<'a>(&'a mut self,
                               algo_data: Option<&'a AlgoDataType>)
                              -> Pin<Box<dyn Future<Output=PresentableMeasurement> + Send + 'a>> {
        let before_event_measurement = self.before_event_measurement.take();
        let Some(before_event_measurement) = before_event_measurement else {
            panic!("BUG! Please, fix: \"after event measurer\" was called but no \"before event measurer\" took place");
        };
        Box::pin(async move {
            (self.after_event_measurer_fn)(algo_data, before_event_measurement).await
        })
    }

    fn as_custom_measurement(&self, after_event_measurement: PresentableMeasurement)
                            -> CustomMeasurement {
        CustomMeasurement {
            name: self.name.clone(),
            expected_complexity: self.expected_complexity,
            description: self.description.clone(),
            measured_data: after_event_measurement,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::measurements;
    use std::future;
    use std::ops::Add;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_custom_measurer() {
        let expected_elapsed_seconds = 38.46;
        let tolerance = 1e-2;
        let before_event_measurer = |_: Option<&()>| future::ready(Instant::now());
        let after_event_measurer = |_: Option<&()>, instant: Instant| future::ready(measurements::presentable_measurements::duration_measurement(instant.elapsed().add(Duration::from_secs_f64(expected_elapsed_seconds))));
        let mut custom_measurer = CustomMeasurer::new("t", BigOAlgorithmComplexity::BetterThanO1, "t descr", before_event_measurer, after_event_measurer);
        custom_measurer.measure_before_event(None.as_ref()).await;
        let measurement_data = custom_measurer.measure_after_event(None).await;
        assert!(measurement_data.to_string().ends_with("s"), "This doesn't look like a duration measurement");
        assert!((measurement_data.value - expected_elapsed_seconds).abs() <= tolerance, "We expect a measurement of ~{expected_elapsed_seconds:.2} seconds; got {:.2} seconds", measurement_data.value);
    }
}