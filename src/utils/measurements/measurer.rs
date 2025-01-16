//! Contains functionalities related to performing measurements

use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use crate::api::builder::CustomMeasurement;
use crate::utils::measurements::presentable_measurements::PresentableMeasurement;

pub trait MeasurerExecutor<AlgoDataType: Send + Debug> {
    fn measure_before_event<'a>(&'a mut self,
                                algo_data: Option<&'a AlgoDataType>)
                               -> Pin<Box<dyn Future<Output=()> + Send + 'a>>;
    fn measure_after_event<'a>(&'a mut self,
                               algo_data: Option<&'a AlgoDataType>)
                              -> Pin<Box<dyn Future<Output=PresentableMeasurement> + Send + 'a>>;
}

/// Contains the definitions for a measurer that is performed
/// through a provided synchronous closure
pub struct Measurer<BeforeMeasurerOutput:                                                             Send,
                    BeforeFut:       Future<Output=BeforeMeasurerOutput>                            + Send,
                    MeasureBeforeFn: FnMut(Option<&AlgoDataType>) -> BeforeFut                      + Send + Sync,
                    AfterFut:        Future<Output=PresentableMeasurement>                          + Send,
                    MeasureAfterFn:  FnMut(Option<&AlgoDataType>, BeforeMeasurerOutput) -> AfterFut + Send + Sync,
                    AlgoDataType:                                                                     Send + Debug> {
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
Measurer<BeforeMeasurerOutput,
         BeforeFut,
         MeasureBeforeFn,
         AfterFut,
         MeasureAfterFn,
         AlgoDataType> {
    pub fn new(before_event_measurer_fn: MeasureBeforeFn,
               after_event_measurer_fn: MeasureAfterFn)
           -> Self {
        Self {
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
MeasurerExecutor<AlgoDataType> for
Measurer<BeforeMeasurerOutput,
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
}

pub async fn measure_all_before_event<AlgoDataType: Send + Sync + Debug>
                                     (algo_data:                 Option<&AlgoDataType>,
                                      measurements_details:      &mut Vec<CustomMeasurement<AlgoDataType>>) {
    // measure in reversed order
    for measurement_detail in measurements_details.iter_mut().rev() {
        measurement_detail.measurer_executor.measure_before_event(algo_data).await;
    }
}

pub async fn measure_all_after_event<AlgoDataType: Send + Sync + Debug>
                                    (algo_data:                 Option<&AlgoDataType>,
                                     measurements_details:      &mut Vec<CustomMeasurement<AlgoDataType>>)
                                    -> Vec<PresentableMeasurement> {
    let mut measurements = vec![];
    for measurement_details in measurements_details.iter_mut() {
        let measurement = measurement_details.measurer_executor.measure_after_event(algo_data).await;
        measurements.push(measurement);
    }
    measurements
}