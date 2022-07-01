//! Implementations for structs/enums defined in [super::types]
//! -- mainly for the `Display` & `Debug` traits.\
//! TODO 2022-06-28: The pursued benefit with this split was to make that module simpler -- maybe this doesn't compensate.

use super::types::*;
use std::fmt::{Display, Formatter};


impl BigOAlgorithmComplexity {
    /// verbose description for each enum element
    pub fn as_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1        => "Better than O(1)",
            Self::O1                  => "O(1)",
            Self::BetweenO1AndOLogN   => "Worse than O(1), but better than O(log(n))",
            Self::OLogN               => "O(log(n))",
            Self::BetweenOLogNAndON   => "Worse than O(log(n)), but better than O(n)",
            Self::ON                  => "O(n)",
            Self::BetweenONAndONLogN  => "Worse than O(n), but better than O(n.log(n))",
            Self::ONLogN              => "O(n.log(n))",
            Self::BetweenONLogNAndON2 => "Worse than O(n.log(n)), but better than O(n²)",
            Self::ON2                 => "O(n²)",
            Self::BetweenON2AndON3    => "Worse than O(n²), but better than O(n³)",
            Self::ON3                 => "O(n³)",
            Self::BetweenON3AndON4    => "Worse than O(n³), but better than O(n^4)",
            Self::ON4                 => "O(n^4)",
            Self::BetweenON4AndOkN    => "Worse than O(n^4), but better than O(k^n)",
            Self::OkN                 => "O(k^n)",
            Self::WorseThanExponential => "Worse than O(k^n)",
        }
    }
    /// same as [as_pretty_str()], with additional info for time analysis
    pub fn as_time_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1) -- aren't the machines idle? too many threads? too little RAM?",
            Self::WorseThanExponential => "Worse than Exponential!! -- worse than O(k^n) -- really, really bad algorithm, too short execution times or is there a hidden bug?",
            _ => self.as_pretty_str(),
        }
    }
    /// verbose description for each enum element, provided we're analysing space complexity
    pub fn as_space_pretty_str(&self) -> &'static str {
        match self {
            Self::BetterThanO1      => "Better than O(1) -- are initialization allocations involved? Consider using a warm up pass",
            Self::WorseThanExponential => "Worse than Exponential!! -- worse than O(k^n) -- really, really bad algorithm or is there a hidden bug?",
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


impl<'a, ScalarTimeUnit: Copy> BigOAlgorithmMeasurements for AlgorithmMeasurements<'a, ScalarTimeUnit> {
    fn space_measurements(&self) -> &BigOSpaceMeasurements {
        &self.space_measurements
    }
}
impl<'a, ScalarTimeUnit: Copy> Display for AlgorithmMeasurements<'a, ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pass_1_time  = format!("{}", self.time_measurements.pass_1_measurements);
        let pass_2_time  = format!("{}", self.time_measurements.pass_2_measurements);
        let pass_1_space = format!("{}", self.space_measurements.pass_1_measurements);
        let pass_2_space = format!("{}", self.space_measurements.pass_2_measurements);
        let pass_1_space_per_n = format!("{}", self.space_measurements.pass_1_measurements.fmt_over_n(self.passes_info.pass1_n));
        let pass_2_space_per_n = format!("{}", self.space_measurements.pass_2_measurements.fmt_over_n(self.passes_info.pass2_n));
        write!(f, "'{}' regular-algorithm measurements:\n\
                   pass          Δt              Δs             n            s⁻           t⁻\n\
                   1) {:>13}  {:>14}  {:>12}  {:>12}  {:>12.3}{}\n\
                   2) {:>13}  {:>14}  {:>12}  {:>12}  {:>12.3}{}\n",

               self.measurement_name,

               pass_1_time, pass_1_space, self.passes_info.pass1_n,
               pass_1_space_per_n,
               self.time_measurements.pass_1_measurements.elapsed_time as f64 / self.passes_info.pass1_n as f64, self.time_measurements.pass_1_measurements.time_unit.unit_str,

               pass_2_time, pass_2_space, self.passes_info.pass2_n,
               pass_2_space_per_n,
               self.time_measurements.pass_2_measurements.elapsed_time as f64 / self.passes_info.pass2_n as f64, self.time_measurements.pass_2_measurements.time_unit.unit_str
        )
    }
}


impl<'a, ScalarTimeUnit: Copy> BigOAlgorithmMeasurements for ConstantSetIteratorAlgorithmMeasurements<'a, ScalarTimeUnit> {
    fn space_measurements(&self) -> &BigOSpaceMeasurements {
        &self.space_measurements
    }
}
impl<'a, ScalarTimeUnit: Copy> Display for ConstantSetIteratorAlgorithmMeasurements<'a, ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pass_1_time  = format!("{}", self.time_measurements.pass_1_measurements);
        let pass_2_time  = format!("{}", self.time_measurements.pass_2_measurements);
        let pass_1_space = format!("{}", self.space_measurements.pass_1_measurements);
        let pass_2_space = format!("{}", self.space_measurements.pass_2_measurements);
        write!(f, "'{}' constant set iterator-algorithm measurements:\n\
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


impl<'a,ScalarTimeUnit: Copy> BigOAlgorithmMeasurements for SetResizingIteratorAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn space_measurements(&self) -> &BigOSpaceMeasurements {
        &self.space_measurements
    }
}
impl<'a,ScalarTimeUnit: Copy> Display for SetResizingIteratorAlgorithmMeasurements<'a,ScalarTimeUnit> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pass_1_time  = format!("{}", self.time_measurements.pass_1_measurements);
        let pass_2_time  = format!("{}", self.time_measurements.pass_2_measurements);
        let pass_1_space = format!("{}", self.space_measurements.pass_1_measurements);
        let pass_2_space = format!("{}", self.space_measurements.pass_2_measurements);
        write!(f, "'{}' set resizing iterator-algorithm measurements:\n\
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


impl BigOSpacePassMeasurements {
    /// Presents either the used or freed memory, with b, KiB, MiB or GiB unit suffixes -- and with the optional `-`, `+` or a null prefix:
    ///  * `-` denotes RAM was freed instead of allocated
    ///  * `+` means RAM was allocated (and remained so)
    ///  * a null prefix indicates RAM was allocated, but got freed -- so no extra RAM is being used.
    ///
    /// When `n` is 1.0, shows the absolute RAM usage;
    /// otherwise, `n` should be the number of elements and the output will represent the memory usage per element
    pub fn fmt_over_n(&self, n: u32) -> String {
        let used_memory = (self.used_memory_after as f32 - self.used_memory_before as f32) / n as f32;
        let sign = if used_memory > 0.0 {"+"} else if used_memory < 0.0 {"-"} else {""};
        let used_memory = std::cmp::max( self.max_used_memory    - self.used_memory_before,
                                              self.used_memory_before - self.min_used_memory ) as f32 / n as f32;
        let memory_unit = if used_memory.abs() > (1<<30) as f32 {"GiB"}                        else if used_memory.abs() > (1<<20) as f32 {"MiB"}                              else if used_memory.abs() > (1<<10) as f32 {"KiB"}                              else {"b"};
        let memory_delta = if used_memory.abs() > (1<<30) as f32 {used_memory / (1<<30) as f32} else if used_memory.abs() > (1<<20) as f32 {used_memory.abs() / (1<<20) as f32} else if used_memory.abs() > (1<<10) as f32 {used_memory.abs() / (1<<10) as f32} else {used_memory.abs()};
        // emit the dot or not
        if (memory_delta.round()-memory_delta).abs() < 1e-3 && memory_unit == "b" {
            format!("{}{:.0}{}", sign, memory_delta, memory_unit)
        } else {
            format!("{}{:.2}{}", sign, memory_delta, memory_unit)
        }
    }
}
impl Display for BigOSpacePassMeasurements {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fmt_over_n(1))
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


impl<T> Default for TimeUnit<T> {
    fn default() -> Self {
        Self { unit_str: "N/A", duration_conversion_fn_ptr: |_| panic!("use of default TimeUnit") }
    }
}


#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [types_impl](super) submodule -- using 'serial_test' crate in order to make time measurements more reliable.


    use crate::{
        low_level_analysis::{
            types::BigOAlgorithmComplexity,
        },
    };
    use serial_test::serial;


    /// assures serializations & implementors of *Display* from [types] work without panics
    /// -- also outputs them for manual inspection
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial]
    fn serialization() {
        println!("BigOAlgorithmComplexity enum members, as strings:");
        let enum_members = [
            BigOAlgorithmComplexity::BetterThanO1,
            BigOAlgorithmComplexity::O1,
            BigOAlgorithmComplexity::OLogN,
            BigOAlgorithmComplexity::BetweenOLogNAndON,
            BigOAlgorithmComplexity::ON,
            BigOAlgorithmComplexity::BetweenONAndONLogN,
            BigOAlgorithmComplexity::ONLogN,
            BigOAlgorithmComplexity::BetweenONLogNAndON2,
            BigOAlgorithmComplexity::ON2,
            BigOAlgorithmComplexity::BetweenON2AndON3,
            BigOAlgorithmComplexity::ON3,
            BigOAlgorithmComplexity::BetweenON3AndON4,
            BigOAlgorithmComplexity::ON4,
            BigOAlgorithmComplexity::BetweenON4AndOkN,
            BigOAlgorithmComplexity::OkN,
            BigOAlgorithmComplexity::WorseThanExponential,
        ];
        for enum_member in enum_members {
            println!("\t{:?}:\n\t\t=> '{}'", enum_member, enum_member.as_pretty_str());
        }
        println!("\n");

        let special_enum_members = [
            BigOAlgorithmComplexity::BetterThanO1,
            BigOAlgorithmComplexity::WorseThanExponential,
        ];

        println!(" .as_time_pretty_str():");
        for enum_member in special_enum_members {
            println!("\t{:?}:\n\t\t=> '{}'", enum_member, enum_member.as_pretty_str());
        }
        println!("\n");

        println!(" .as_space_pretty_str():");
        for enum_member in special_enum_members {
            println!("\t{:?}:\n\t\t=> '{}'", enum_member, enum_member.as_pretty_str());
        }
        println!("\n");
    }
}
