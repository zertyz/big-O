//! Contains common types used throughout our API

use std::future::Future;
use std::pin::Pin;

/// Defines optional operations to be applied on the Algorithm under analysis' data -- such as "Reset" and "Warmup"
pub type AlgoManipulationAsyncFn<AlgoDataType> = Box<dyn FnMut(Option<AlgoDataType>) -> Pin<Box<dyn Future<Output=AlgoDataType> + Send>> + Send + Sync>;
/// Defines optional assertions to be applied on the Algorithm's data after a pass is executed -- to allow ensuring the pass worked as expected
pub type AlgoAssertionAsyncFn<AlgoDataType> = Box<dyn FnMut(&AlgoDataType) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>;