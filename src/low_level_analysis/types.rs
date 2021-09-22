//! Defines traits, enums & structs returned / shared by this crate's functions.

use std::fmt::{Display, Formatter};

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
impl BigOAlgorithmComplexity {
    /// verbose description for each enum element
    pub fn as_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1)",
            Self::O1                => "O(1)",
            Self::OLogN             => "O(log(n))",
            Self::BetweenOLogNAndON => "Worse than O(log(n)) but better than O(n)",
            Self::ON                => "O(n)",
            Self::WorseThanON       => "Worse than O(n)",
        }
    }
    /// same as [as_pretty_str()], with additional info for time analysis
    pub fn as_time_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1) -- aren't the machines idle? too many threads? too little RAM?",
            Self::WorseThanON       => "Worse than O(n) -- really bad algorithm or CPU cache sizes might be playing a role?",
            _ => self.as_pretty_str(),
        }
    }
    /// verbose description for each enum element, provided we're analysing space complexity
    pub fn as_space_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1) -- are initialization allocations involved? Consider using a warm up pass",
            Self::WorseThanON       => "Worse than O(n) -- really, really bad algorithm or is there a hidden bug?",
            _ => self.as_pretty_str(),
        }
    }
}

/// base trait for [SetResizingAlgorithmMeasurements] & [ConstantSetAlgorithmMeasurements], made public
/// to attend to rustc's rules. Most probably this trait is of no use outside it's own module.
pub trait BigOAlgorithmMeasurements: Display {
    fn space_measurements(&self) -> &BigOSpaceMeasurements;
}

/// Type for [BigOAlgorithmAnalysis::algorithm_measurements] when analyzing algorithms
/// that do not change the set size they operate on -- select/update, get, sort, fib, ...
/// See also [SetResizingAlgorithmMeasurements]
pub struct ConstantSetAlgorithmMeasurements<'a,ScalarTimeUnit: Copy> {
    /// a name for these measurements, for presentation purposes
    pub measurement_name:   &'a str,
    /// each pass info for use in the time & space complexity analysis
    pub passes_info:        ConstantSetAlgorithmPassesInfo,
    pub time_measurements:  BigOTimeMeasurements<'a,ScalarTimeUnit>,
    pub space_measurements: BigOSpaceMeasurements,
}
impl<'a,ScalarTimeUnit: Copy> BigOAlgorithmMeasurements for ConstantSetAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn space_measurements(&self) -> &BigOSpaceMeasurements {
        &self.space_measurements
    }
}
impl<'a,ScalarTimeUnit: Copy> Display for ConstantSetAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // placing those in string variables since {:>12} seem not to work on implementers of Display
        let pass_1_time  = format!("{}", self.time_measurements.pass_1_measurements);
        let pass_2_time  = format!("{}", self.time_measurements.pass_2_measurements);
        let pass_1_space = format!("{}", self.space_measurements.pass_1_measurements);
        let pass_2_space = format!("{}", self.space_measurements.pass_2_measurements);
        write!(f, "'{}' constant set algorithm measurements:\n\
                   pass          Δt              Δs            Σn            ⊆r            t⁻\n\
                   1) {:>13}  {:>14}  {:>12}  {:>12}  {:>12.3}{}\n\
                   2) {:>13}  {:>14}  {:>12}  {:>12}  {:>12.3}{}\n",

               self.measurement_name,

               pass_1_time, pass_1_space, self.passes_info.pass_1_set_size,
               self.passes_info.repetitions, self.time_measurements.pass_1_measurements.elapsed_time as f64 / self.passes_info.repetitions as f64, self.time_measurements.pass_1_measurements.time_unit.unit_str,

               pass_2_time, pass_2_space, self.passes_info.pass_2_set_size,
               self.passes_info.repetitions, self.time_measurements.pass_2_measurements.elapsed_time as f64 / self.passes_info.repetitions as f64, self.time_measurements.pass_2_measurements.time_unit.unit_str
        )
    }
}

/// Type for [BigOAlgorithmAnalysis::algorithm_measurements] when analyzing algorithms
/// that change the set size they operate on -- insert/delete, enqueue/dequeue, push/pop, add/remove, ...
/// See also [ConstantSetAlgorithmMeasurements]
pub struct SetResizingAlgorithmMeasurements<'a,ScalarTimeUnit: Copy> {
    /// a name for these measurements, for presentation purposes
    pub measurement_name: &'a str,
    /// each pass info for use in the time & space complexity analysis
    pub passes_info:        SetResizingAlgorithmPassesInfo,
    pub time_measurements:  BigOTimeMeasurements<'a,ScalarTimeUnit>,
    pub space_measurements: BigOSpaceMeasurements,
}
impl<'a,ScalarTimeUnit: Copy> BigOAlgorithmMeasurements for SetResizingAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn space_measurements(&self) -> &BigOSpaceMeasurements {
        &self.space_measurements
    }
}
impl<'a,ScalarTimeUnit: Copy> Display for SetResizingAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // placing those in string variables since {:>12} seem not to work on implementers of Display
        let pass_1_time  = format!("{}", self.time_measurements.pass_1_measurements);
        let pass_2_time  = format!("{}", self.time_measurements.pass_2_measurements);
        let pass_1_space = format!("{}", self.space_measurements.pass_1_measurements);
        let pass_2_space = format!("{}", self.space_measurements.pass_2_measurements);
        write!(f, "'{}' set resizing algorithm measurements:\n\
                   pass          Δt              Δs            Σn            t⁻\n\
                   1) {:>13}  {:>14}  {:>12}  {:>12.3}{}\n\
                   2) {:>13}  {:>14}  {:>12}  {:>12.3}{}\n",
               self.measurement_name,
               pass_1_time, pass_1_space, self.passes_info.delta_set_size,   self.time_measurements.pass_1_measurements.elapsed_time as f64 / self.passes_info.delta_set_size as f64, self.time_measurements.pass_1_measurements.time_unit.unit_str,
               pass_2_time, pass_2_space, self.passes_info.delta_set_size*2, self.time_measurements.pass_2_measurements.elapsed_time as f64 / self.passes_info.delta_set_size as f64, self.time_measurements.pass_2_measurements.time_unit.unit_str)
    }
}

/// represents an algorithm's run-time time measurements for passes 1 & 2, so that it can have it's time complexity analyzed
pub struct BigOTimeMeasurements<'a,ScalarTimeUnit: Copy> {
    pub pass_1_measurements: BigOTimePassMeasurements<'a,ScalarTimeUnit>,
    pub pass_2_measurements: BigOTimePassMeasurements<'a,ScalarTimeUnit>,
}

/// the elapsed time & unit taken to run one of the 2 passes for the algorithm time complexity analysis.
/// Contained in [BigOTimeMeasurements].
#[derive(Clone,Copy)]
pub struct BigOTimePassMeasurements<'a,ScalarTimeUnit> where ScalarTimeUnit: Clone+Copy {
    /// the time it took to run a pass
    pub elapsed_time: u64,
    /// unit for the measurements in this struct
    pub time_unit: &'a TimeUnit<ScalarTimeUnit>,
}
impl<ScalarTimeUnit: Copy> Display for BigOTimePassMeasurements<'_,ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.elapsed_time, self.time_unit.unit_str)
    }
}

/// represents an algorithm's run-time memory usage measurements for passes 1 & 2, in bytes, so that it can have it's space complexity analyzed
pub struct BigOSpaceMeasurements {
    pub pass_1_measurements: BigOSpacePassMeasurements,
    pub pass_2_measurements: BigOSpacePassMeasurements,
}
impl BigOSpaceMeasurements {
    /// returns the resulting used memory, obtained from the space complexity analysis measurements --
    /// >0 if memory was allocated and <0 if memory was freed
    pub fn used_memory_delta(&self) -> isize {
        self.pass_2_measurements.used_memory_after as isize - self.pass_2_measurements.used_memory_before as isize +
            self.pass_1_measurements.used_memory_after as isize - self.pass_1_measurements.used_memory_before as isize
    }
    /// auxiliary space refers to the memory used during the computation and then freed before ending the computation --
    /// returns max_used_ram - max(used_memory_delta, used_memory_start), meaning: >0 if auxiliary memory was allocated
    pub fn used_auxiliary_space(&self) -> usize {
        self.pass_2_measurements.max_used_memory - std::cmp::max(self.pass_2_measurements.used_memory_after, self.pass_2_measurements.used_memory_before) +
            self.pass_1_measurements.max_used_memory - std::cmp::max(self.pass_1_measurements.used_memory_after, self.pass_1_measurements.used_memory_before)
    }
}
impl Default for BigOSpaceMeasurements {
    fn default() -> Self {
        Self { pass_1_measurements: BigOSpacePassMeasurements::default(), pass_2_measurements: BigOSpacePassMeasurements::default() }
    }
}
impl Display for BigOSpaceMeasurements {
    // shows allocated / deallocated amount + any used auxiliary space
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn to_human_readable_string(used_memory: f32) -> String {
            let memory_unit = if used_memory > (1<<30) as f32 {"GiB"}                        else if used_memory > (1<<20) as f32 {"MiB"}                        else if used_memory > (1<<10) as f32 {"KiB"}                        else {"b"};
            let memory_delta = if used_memory > (1<<30) as f32 {used_memory / (1<<30) as f32} else if used_memory > (1<<20) as f32 {used_memory / (1<<20) as f32} else if used_memory > (1<<10) as f32 {used_memory / (1<<10) as f32} else {used_memory};
            format!("{:.2}{}", memory_delta, memory_unit)
        }
        let used_or_freed = self.used_memory_delta();
        let alloc_op = if used_or_freed >= 0 { "allocated" } else { "freed" };
        let used_auxiliary_space = self.used_auxiliary_space();
        write!(f, "{}: {}; auxiliary used space: {}",
               alloc_op,
               to_human_readable_string(used_or_freed.abs() as f32),
               to_human_readable_string(used_auxiliary_space as f32))
    }
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
impl Display for BigOSpacePassMeasurements {
    // shows a summary -- just the used or freed memory, with b, KiB, MiB or GiB unit suffixes
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let used_memory = self.used_memory_after as f32 - self.used_memory_before as f32;
        let sign = if used_memory > 0.0 {"+"} else if used_memory < 0.0 {"-"} else {""};
        let used_memory = std::cmp::max( self.max_used_memory    - self.used_memory_before,
                                         self.used_memory_before - self.min_used_memory ) as f32;
        let memory_unit = if used_memory.abs() > (1<<30) as f32 {"GiB"}                        else if used_memory.abs() > (1<<20) as f32 {"MiB"}                              else if used_memory.abs() > (1<<10) as f32 {"KiB"}                              else {"b"};
        let memory_delta = if used_memory.abs() > (1<<30) as f32 {used_memory / (1<<30) as f32} else if used_memory.abs() > (1<<20) as f32 {used_memory.abs() / (1<<20) as f32} else if used_memory.abs() > (1<<10) as f32 {used_memory.abs() / (1<<10) as f32} else {used_memory.abs()};
        write!(f, "{}{:.2}{}", sign, memory_delta, memory_unit)
    }
}
impl Default for BigOSpacePassMeasurements {
    fn default() -> Self {
        Self {
            used_memory_before: 0,
            used_memory_after: 0,
            max_used_memory: 0,
            min_used_memory: 0,
        }
    }
}

/// base type for [ConstantSetAlgorithmPassesInfo] and [SetResizingAlgorithmPassesInfo]
/// which contains the pass information for the Algorithm's space & time analysis for
/// each type of algorithm
pub trait AlgorithmPassesInfo {}

/// Represents the pass information for Algorithms that don't alter the set size of the data they operate on
/// (Selects / Updates / Sort / Fib...)
#[derive(Clone,Copy)]
pub struct ConstantSetAlgorithmPassesInfo {
    /// set size when running "pass 1"
    pub pass_1_set_size: u32,
    /// set size when running "pass 2"
    pub pass_2_set_size: u32,
    /// number of times the algorithm ran on each pass;
    /// each algorithm iteration should behave as executing on the same element without leaving side-effects
    pub repetitions: u32,
}
impl AlgorithmPassesInfo for ConstantSetAlgorithmPassesInfo {}

/// Represents the pass information for Algorithms that alters the set size of the data they operate on
/// (Inserts / Deletes / Pushes / Pops / Enqueues / Dequeues...)
pub struct SetResizingAlgorithmPassesInfo {
    /// number of elements added / removed on each pass;
    /// each algorithm iteration should either add or remove a single element
    /// and the test set must start or end with 0 elements
    pub delta_set_size: u32,
}
impl AlgorithmPassesInfo for SetResizingAlgorithmPassesInfo {}

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
impl<T> Default for TimeUnit<T> {
    fn default() -> Self {
        Self { unit_str: "N/A", duration_conversion_fn_ptr: |_| panic!("use of default TimeUnit") }
    }
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