//! Resting place for [PresentableMeasurement]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::time::Duration;
use once_cell::sync::Lazy;

/// Holds and present custom unit measurements with auto-scaling
pub struct PresentableMeasurement {
    pub(crate) value: f64,
    /// := (threshold, scale, unit, format)
    auto_scale: &'static [(f64, f64, Cow<'static, str>, &'static str)],
}
impl Default for PresentableMeasurement {
    fn default() -> Self {
        Self {
            value: 0.0,
            auto_scale: &[],
        }
    }
}

impl Display for PresentableMeasurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (scaled_value, suffix, format) = self.auto_scale.iter()
            .find(|&&(threshold, _, _, _)| self.value >= threshold)
            .map_or(
                (self.value, &Cow::Borrowed("<missing_unit_suffix_please_fix>"), ":.2"),
                |(_threshold, rate, suffix, format)| (self.value / rate, suffix, format));
        match format {
            ":.0"  => write!(f, "{:.0}{}",  scaled_value, suffix),
            ":.1"  => write!(f, "{:.1}{}",  scaled_value, suffix),
            ":.2"  => write!(f, "{:.2}{}",  scaled_value, suffix),
            ":.3"  => write!(f, "{:.3}{}",  scaled_value, suffix),
            ":.3e" => write!(f, "{:.3e}{}", scaled_value, suffix),
            ":D"   => write!(f, "{:?}",     Duration::from_secs_f64(scaled_value)),
            _ => panic!("Unknown format '{format}'. Please update this code")
        }
    }
}

/// Builds a [PresentableMeasurement] able to display & auto-scale
/// quantities representing "a duration".
pub fn duration_measurement(duration: Duration) -> PresentableMeasurement {
    const AUTO_SCALE_DATA: &[(f64, f64, Cow<'static, str>, &'static str)] = &[
        (0.0, 1.0, Cow::Borrowed(""), ":D"),
    ];
    
    PresentableMeasurement {
        value: duration.as_secs_f64(),
        auto_scale: AUTO_SCALE_DATA,
    }
}

/// Builds a [PresentableMeasurement] able to display & auto-scale
/// quantities representing "a number of bytes".
pub fn bytes_measurement(value: f64) -> PresentableMeasurement {
    static AUTO_SCALE_DATA: Lazy<Vec<(f64, f64, Cow<'static, str>, &'static str)>> = Lazy::new(|| {
        [
            ((1u64<<40) as f64, "TiB", ":.2"),
            ((1u64<<30) as f64, "GiB", ":.2"),
            ((1u64<<20) as f64, "MiB", ":.2"),
            ((1u64<<10) as f64, "KiB", ":.2"),
            (1.0,               "b",   ":.0"),
            (0.0,               "b",   ":.0"),
        ]
        .into_iter()
        .map(|(threshold, suffix, format)| (
            threshold,
            if threshold != 0.0 { threshold } else { 1.0 },
            Cow::Borrowed(suffix),
            format
        ))
        .collect()
    });

    PresentableMeasurement {
        value,
        auto_scale: AUTO_SCALE_DATA.as_slice(),
    }
}

/// Builds a [PresentableMeasurement] able to display & auto-scale
/// quantities representing "a rate of bytes per second".
pub fn bytes_per_second_measurement(value: f64) -> PresentableMeasurement {
    static AUTO_SCALE_DATA: Lazy<Vec<(f64, f64, Cow<'static, str>, &'static str)>> = Lazy::new(|| {
        [
            ((1u64<<40) as f64, "TiB/s", ":.2"),
            ((1u64<<30) as f64, "GiB/s", ":.2"),
            ((1u64<<20) as f64, "MiB/s", ":.2"),
            ((1u64<<10) as f64, "KiB/s", ":.2"),
            (       1.0,        "b/s",   ":.2"),
            (  1.0/60.0,        "b/min", ":.2"),
            (1.0/3600.0,        "b/hr",  ":.2"),
            (       0.0,        "b/s",   ":.0"),
        ]
        .into_iter()
        .map(|(threshold, suffix, format)| (
            threshold,
            if threshold != 0.0 { threshold } else { 1.0 },
            Cow::Borrowed(suffix),
            format
        ))
        .collect()
    });

    PresentableMeasurement {
        value,
        auto_scale: AUTO_SCALE_DATA.as_slice(),
    }
}

/// Builds a [PresentableMeasurement] able to display & auto-scale
/// quantities representing "a quantity of `custom_unit`".
pub fn custom_unit_measurement(value: f64, custom_unit: &'static str) -> PresentableMeasurement {

    static mut AUTO_SCALE_DATA: Option<HashMap<&str, Vec<(f64, f64, Cow<'static, str>, &'static str)>>> = None;

    // non-synchronized one-time cache
    let auto_scale_data = unsafe {
        #[allow(static_mut_refs)]
        // safety: this crate ensures a single thread will call the measurement & report functions.
        // The alternative would be to use Mutexes, which are not needed in this case
        &mut AUTO_SCALE_DATA
    }
        .get_or_insert_with(HashMap::new)
        .entry(custom_unit)
        .or_insert_with(|| {
            [
                (100_000.0, custom_unit, ":.3e"),
                (      1.0, custom_unit, ":.2"),
                (      0.0, custom_unit, ":.0"),
            ]
            .into_iter()
            .map(|(threshold, suffix, format)| (threshold, 1.0, Cow::Borrowed(suffix), format))
            .collect()
        });

    PresentableMeasurement {
        value,
        auto_scale: auto_scale_data.as_slice(),

    }
}

/// Builds a [PresentableMeasurement] able to display & auto-scale
/// quantities representing "a rate of `custom_unit` quantities per second".
fn custom_unit_per_second_measurement(value: f64, custom_unit: &'static str) -> PresentableMeasurement {

    static mut AUTO_SCALE_DATA: Option<HashMap<&str, Vec<(f64, f64, Cow<'static, str>, &'static str)>>> = None;

    // non-synchronized cache per `custom_unit`
    let auto_scale_data = unsafe {
        #[allow(static_mut_refs)]
        // safety: this crate ensures a single thread will call the measurement & report functions.
        // The alternative would be to use Mutexes, which are not needed in this case
        &mut AUTO_SCALE_DATA
    }
        .get_or_insert_with(HashMap::new)
        .entry(custom_unit)
        .or_insert_with(|| {
            [
                (100_000.0,  1.0,        format!("{custom_unit}/s"), ":.3e"),
                (1.0,        1.0,        format!("{custom_unit}/s"), ":.2"),
                (1.0/60.0,   1.0/60.0,   format!("{custom_unit}/min"), ":.2"),
                (1.0/3600.0, 1.0/3600.0, format!("{custom_unit}/hr"), ":.2"),
                (0.0,        1.0,        format!("{custom_unit}/s"), ":.0"),
            ]
            .into_iter()
            .map(|(threshold, rate, suffix, format)| (threshold, rate, Cow::Owned(suffix), format))
            .collect()
        });

    PresentableMeasurement {
        value,
        auto_scale: auto_scale_data.as_slice(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_measurement() {
        let expected_representations = [
            ( Duration::from_secs_f64(    3600.0), "3600s" ),
            ( Duration::from_secs_f64(      60.0), "60s"   ),
            ( Duration::from_secs_f64(       0.0), "0ns"   ),
            ( Duration::from_secs_f64(    0.001 ), "1ms"   ),
            ( Duration::from_secs_f64( 0.000001 ), "1µs"   ),
        ];
        let measurement_fn = duration_measurement;
        for (value, expected_representation) in expected_representations {
            let observed_representation = measurement_fn(value).to_string();
            assert_eq!(&observed_representation, expected_representation, "Measurement representation doesn't match");
        }
    }

    #[test]
    fn test_bytes_measurement() {
        let expected_representations = [
            (                              0.0, "0b"      ),
            (                            10.15, "10b"     ),
            (                     1024.0*10.15, "10.15KiB"),
            (              1024.0*1024.0*10.15, "10.15MiB"),
            (       1024.0*1024.0*1024.0*10.15, "10.15GiB"),
            (1024.0*1024.0*1024.0*1024.0*10.15, "10.15TiB"),
        ];
        let measurement_fn = bytes_measurement;
        for (value, expected_representation) in expected_representations {
            let observed_representation = measurement_fn(value).to_string();
            assert_eq!(&observed_representation, expected_representation, "Measurement representation doesn't match");
        }
    }

    #[test]
    fn test_bytes_per_second_measurement() {
        let expected_representations = [
            (                              0.0, "0b/s"      ),
            (                            0.011, "39.60b/hr" ),
            (                            0.921, "55.26b/min"),
            (                            10.15, "10.15b/s"  ),
            (                     1024.0*10.15, "10.15KiB/s"),
            (              1024.0*1024.0*10.15, "10.15MiB/s"),
            (       1024.0*1024.0*1024.0*10.15, "10.15GiB/s"),
            (1024.0*1024.0*1024.0*1024.0*10.15, "10.15TiB/s"),
        ];
        let measurement_fn = bytes_per_second_measurement;
        for (value, expected_representation) in expected_representations {
            let observed_representation = measurement_fn(value).to_string();
            assert_eq!(&observed_representation, expected_representation, "Measurement representation doesn't match");
        }
    }

    #[test]
    fn test_custom_unit_measurement() {
        let expected_representations = [
            (             0.0, "0req"       ),
            (           10.15, "10.15req"   ),
            (         10393.6, "10393.60req"),
            (      10643046.4, "1.064e7req" ),
            (   10898479513.6, "1.090e10req"),
            (11160043021926.4, "1.116e13req"),
        ];
        let measurement_fn = |val| custom_unit_measurement(val, "req");
        for (value, expected_representation) in expected_representations {
            let observed_representation = measurement_fn(value).to_string();
            assert_eq!(&observed_representation, expected_representation, "Measurement representation doesn't match");
        }
    }

    #[test]
    fn test_custom_unit_per_second_measurement() {
        let expected_representations = [
            (             0.0, "0req/s"       ),
            (           0.011, "39.60req/hr"  ),
            (            0.92, "55.20req/min" ),
            (           10.15, "10.15req/s"   ),
            (         10393.6, "10393.60req/s"),
            (      10643046.4, "1.064e7req/s" ),
            (   10898479513.6, "1.090e10req/s"),
            (11160043021926.4, "1.116e13req/s"),
        ];
        let measurement_fn = |val| custom_unit_per_second_measurement(val, "req");
        for (value, expected_representation) in expected_representations {
            let observed_representation = measurement_fn(value).to_string();
            assert_eq!(&observed_representation, expected_representation, "Measurement representation doesn't match");
        }
    }
}