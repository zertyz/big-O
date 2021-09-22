//! Defines traits, enums & structs returned / shared by this crate's functions.

use std::fmt::{Display, Formatter};

use crate::low_level_analysis::types::{
    BigOAlgorithmMeasurements, BigOAlgorithmComplexity
};

/// return result for this module's functions for analysing *constant set* & *set resizing* algorithms.
/// See [super::time_analysis] & [super::space_analysis]
pub struct BigOAlgorithmAnalysis<T: BigOAlgorithmMeasurements> {
    pub time_complexity:         BigOAlgorithmComplexity,
    pub space_complexity:        BigOAlgorithmComplexity,
    pub algorithm_measurements:  T,
}
impl<T: BigOAlgorithmMeasurements> Display for BigOAlgorithmAnalysis<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\
                   --> Algorithm  Time Analysis: {}\n\
                   --> Algorithm Space Analysis: {} ({space_measurements})\n",
               self.algorithm_measurements,
               self.time_complexity.as_time_pretty_str(),
               self.space_complexity.as_space_pretty_str(), space_measurements=self.algorithm_measurements.space_measurements())
    }
}

