//! Implementations for structs/enums defined in [super::types]
//! -- mainly for the `Display` & `Debug` traits.\
//! TODO 2022-06-28: The pursued benefit with this split was to make that module simpler -- maybe this doesn't compensate.

use super::types::*;
use std::fmt::{Display, Formatter};


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


impl<ScalarTimeUnit: Copy> Display for BigOTimePassMeasurements<'_,ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.elapsed_time, self.time_unit.unit_str)
    }
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


impl AlgorithmPassesInfo for ConstantSetAlgorithmPassesInfo {}


impl AlgorithmPassesInfo for SetResizingAlgorithmPassesInfo {}


impl<T> Default for TimeUnit<T> {
    fn default() -> Self {
        Self { unit_str: "N/A", duration_conversion_fn_ptr: |_| panic!("use of default TimeUnit") }
    }
}
