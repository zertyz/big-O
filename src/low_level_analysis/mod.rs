//! Exports time & space algorithm complexity analysis functions, as well as the needed types to operate on them. See:
//!   - [time_analysis]
//!   - [space_analysis]
//!   - [types]
//!
//! ... and, most importantly, tests both analysis on real functions. See [low_level_analysis::tests].

mod low_level_analysis;
pub use low_level_analysis::*;
pub mod types;
pub mod time_analysis;
pub mod space_analysis;
pub mod configs;
