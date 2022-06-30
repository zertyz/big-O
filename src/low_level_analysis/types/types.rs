//! See [super] for docs.\
//! See [super::types_impl] as well for implementations of the structs/enums defined here.

use std::fmt::Display;


/// Possible time & space complexity analysis results, in big-O notation.
/// Results are for a single operation -- remember a pass have several operations,
/// so the time for the analysis should have '* 2 * p' added -- 'p' being the size
/// for each one of the 2 passes required for the analysis.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BigOAlgorithmComplexity {
    BetterThanO1,
    O1,
    OLogN,
    BetweenOLogNAndON,
    ON,
    WorseThanON,
}

/// Specifies if the iterator algorithm under analysis alters the data set it works on or if it has no side-effects on it.\
/// Different math applies on each case, as well as different parameters to the iterator function required by the [crate::runner].
/// The "Iterator Algorithms" term is used in this crate to distinguish them from "Standard Algorithms". They differ in the sense
/// that Iterator Algorithms operate on a single element at a time (from a rather huge set) -- and, as said, different math applies
/// to infer, at runtime, their complexity, depending on if they alter the set size or not;
/// on the other hand, "Standard Algorithms" don't need that distinction: they may either build or consult a data set (provided the
/// set is build/consumed from the ground up) and their runtime math is the same as for the "Constant Set Iterator Algorithms".
#[derive(Debug)]
pub enum BigOIteratorAlgorithmType {
    /// the iterator algorithm under analysis change the data set size it operates on. Examples: insert/delete, enqueue/dequeue, ...\
    /// See [math::set_resizing_iterator_algorithm_analysis()]
    SetResizing,
    /// the algorithm under analysis doesn't change the data set size it operates on. Examples: queries, sort, fib, ...\
    /// See [math::constant_set_iterator_algorithm_analysis()]
    ConstantSet,
}

/// base trait for [SetResizingAlgorithmMeasurements] & [ConstantSetAlgorithmMeasurements], made public
/// to attend to rustc's rules. Most probably this trait is of no use outside it's own module.
pub trait BigOAlgorithmMeasurements: Display {
    fn space_measurements(&self) -> &BigOSpaceMeasurements;
}

/// return result for this module's functions for analysing *constant set* & *set resizing* algorithms.
/// See [super::time_analysis] & [super::space_analysis]
pub struct BigOAlgorithmAnalysis<T: BigOAlgorithmMeasurements> {
    pub time_complexity:         BigOAlgorithmComplexity,
    pub space_complexity:        BigOAlgorithmComplexity,
    pub algorithm_measurements:  T,
}

/// Type for [BigOAlgorithmAnalysis::algorithm_measurements] when analyzing regular algorithms
/// (non-iterator algorithms) -- such as sort, fib, ...
pub struct AlgorithmMeasurements<'a, ScalarTimeUnit: Copy> {
    /// a name for these measurements, for presentation purposes
    pub measurement_name:   &'a str,
    /// each pass info for use in the time & space complexity analysis
    pub passes_info:        AlgorithmPassesInfo,
    pub time_measurements:  BigOTimeMeasurements<'a, ScalarTimeUnit>,
    pub space_measurements: BigOSpaceMeasurements,
}

/// Type for [BigOAlgorithmAnalysis::algorithm_measurements] when analyzing iterator algorithms
/// that do not change the set size they operate on -- select/update, get, ...
/// See also [SetResizingAlgorithmMeasurements]
pub struct ConstantSetIteratorAlgorithmMeasurements<'a, ScalarTimeUnit: Copy> {
    /// a name for these measurements, for presentation purposes
    pub measurement_name:   &'a str,
    /// each pass info for use in the time & space complexity analysis
    pub passes_info:        ConstantSetIteratorAlgorithmPassesInfo,
    pub time_measurements:  BigOTimeMeasurements<'a, ScalarTimeUnit>,
    pub space_measurements: BigOSpaceMeasurements,
}

/// Type for [BigOAlgorithmAnalysis::algorithm_measurements] when analyzing iterator algorithms
/// that change the set size they operate on -- insert/delete, enqueue/dequeue, push/pop, add/remove, ...
/// See also [ConstantSetAlgorithmMeasurements]
pub struct SetResizingIteratorAlgorithmMeasurements<'a, ScalarTimeUnit: Copy> {
    /// a name for these measurements, for presentation purposes
    pub measurement_name: &'a str,
    /// each pass info for use in the time & space complexity analysis
    pub passes_info:        SetResizingIteratorAlgorithmPassesInfo,
    pub time_measurements:  BigOTimeMeasurements<'a, ScalarTimeUnit>,
    pub space_measurements: BigOSpaceMeasurements,
}

/// represents an algorithm's run-time time measurements for passes 1 & 2, so that it can have it's time complexity analyzed
pub struct BigOTimeMeasurements<'a, ScalarTimeUnit: Copy> {
    pub pass_1_measurements: BigOTimePassMeasurements<'a, ScalarTimeUnit>,
    pub pass_2_measurements: BigOTimePassMeasurements<'a ,ScalarTimeUnit>,
}

/// the elapsed time & unit taken to run one of the 2 passes for the algorithm time complexity analysis.
/// Contained in [BigOTimeMeasurements].
#[derive(Clone,Copy)]
pub struct BigOTimePassMeasurements<'a, ScalarTimeUnit> where ScalarTimeUnit: Clone+Copy {
    /// the time it took to run a pass
    pub elapsed_time: u64,
    /// unit for the measurements in this struct
    pub time_unit: &'a TimeUnit<ScalarTimeUnit>,
}

/// represents an algorithm's run-time memory usage measurements for passes 1 & 2, in bytes, so that it can have it's space complexity analyzed
pub struct BigOSpaceMeasurements {
    pub pass_1_measurements: BigOSpacePassMeasurements,
    pub pass_2_measurements: BigOSpacePassMeasurements,
}

/// memory usage measurements, in bytes, for a pass execution -- 2 of them are stored in [BigOSpaceMeasurements]
/// and are required to perform the space complexity analysis.
#[derive(Debug,Clone,Copy)]
pub struct BigOSpacePassMeasurements {
    /// heap memory in use just before starting the pass
    pub used_memory_before: usize,
    /// heap memory in use just after the pass execution
    pub used_memory_after:  usize,
    /// maximum heap memory in use during the pass execution
    pub max_used_memory:    usize,
    /// minimum heap memory in use during the pass execution
    pub min_used_memory:    usize,
}

/// Represents the "pass" information (info for the runner that measures time & space resource consumptions)
/// for regular Algorithms which we want to perform the complexity analysis for.\
/// Note that "Regular Algorithms" is in opposition to Iterator Algorithms
pub struct AlgorithmPassesInfo {
    /// elements processed on "pass 1"
    pub pass1_n: u32,
    /// elements pocessed on "pass 2"
    pub pass2_n: u32,
}

/// Represents the pass information for Iterator Algorithms that don't alter the set size of the data they operate on
/// (Selects / Updates / Sort / Fib...)
pub struct ConstantSetIteratorAlgorithmPassesInfo {
    /// set size when running "pass 1"
    pub pass_1_set_size: u32,
    /// set size when running "pass 2"
    pub pass_2_set_size: u32,
    /// number of times the algorithm ran on each pass -- or, in other words, the "n" in the O(n) notation;
    /// each algorithm iteration should behave as executing on the same element without leaving side-effects
    pub repetitions: u32,
}

/// Represents the pass information for Iterator Algorithms that alter the set size of the data they operate on
/// (Inserts / Deletes / Pushes / Pops / Enqueues / Dequeues...)
pub struct SetResizingIteratorAlgorithmPassesInfo {
    /// number of elements added / removed on each pass;
    /// each algorithm iteration should either add or remove a single element
    /// and the test set must start or end with 0 elements
    pub delta_set_size: u32,
}

/// Specifies a time unit for the 'big-O' crate when measuring / reporting results.
/// Please use one of the prebuilt 'TimeUnits' constants instead of instantiating this:
/// [TimeUnits::NANOSECOND], [TimeUnits::MICROSECOND], [TimeUnits::MILLISECOND],  [TimeUnits::SECOND]
pub struct TimeUnit<T> {
    /// printable unit suffix: 'ns', 'µs', etc.
    pub unit_str: &'static str,
    /// one of [std::time::Duration]'s 'as_micros', 'as_seconds', ... function to convert a Duration object into a scalar
    pub(crate) duration_conversion_fn_ptr: fn(&std::time::Duration) -> T,
}
impl<T> TimeUnit<T> {
    /// the same as [Self::default()], from which we can return a read-only reference
    const CONST_DEFAULT: TimeUnit<T> = Self { unit_str: "N/A", duration_conversion_fn_ptr: |_| panic!("use of default TimeUnit") };
}

/// prebuilt [TimeUnit] constants
pub struct TimeUnits {}
impl TimeUnits {
    pub const NANOSECOND:  TimeUnit<u128> = TimeUnit { unit_str: "ns", duration_conversion_fn_ptr: std::time::Duration::as_nanos};
    pub const MICROSECOND: TimeUnit<u128> = TimeUnit { unit_str: "µs", duration_conversion_fn_ptr: std::time::Duration::as_micros};
    pub const MILLISECOND: TimeUnit<u128> = TimeUnit { unit_str: "ms", duration_conversion_fn_ptr: std::time::Duration::as_millis};
    pub const SECOND:      TimeUnit<u64>  = TimeUnit { unit_str: "s",  duration_conversion_fn_ptr: std::time::Duration::as_secs};

    /// returns a reference to the constant default TimeUnit<T> -- acts as a placeholder for mutable variables
    pub fn get_const_default<'a,T>() -> &'a TimeUnit<T> {
        &TimeUnit::<T>::CONST_DEFAULT
    }
}