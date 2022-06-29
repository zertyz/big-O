//! Contains executors of the algorithms, gathering metrics to pass to
//! [crate::low_level_analysis] in order to have their complexity measured

pub(crate) mod common;
pub mod standard;
pub mod crud;
