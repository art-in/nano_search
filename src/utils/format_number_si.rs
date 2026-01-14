use num_traits::AsPrimitive;

/// Formats a number with human-readable SI prefixes (like K/M/G).
pub fn format_number_si<T>(value: T, units: &str) -> String
where
    T: AsPrimitive<f64>,
{
    human_format::Formatter::new()
        .with_units(units)
        .format(value.as_())
}

/// Formats number of bytes with human-readable SI prefixes, e.g. KB/MB/GB.
///
/// Not to be confused with Binary prefixes, e.g. KiB/MiB/GiB:
/// this util uses 1000 as decimal multiples of base unit, not 1024.
pub fn format_bytes_si<T>(value: T) -> String
where
    T: AsPrimitive<f64>,
{
    format_number_si(value, "B")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_format_number() {
        assert_eq!(format_number_si(-500i32, "V"), "-500.00 V");
        assert_eq!(format_number_si(1000u64, "m/sec"), "1.00 Km/sec");
        assert_eq!(format_number_si(2500.0f32, "W"), "2.50 KW");
        assert_eq!(format_number_si(0.015_f64, "l"), "0.01 l");
    }

    #[test]
    fn test_human_format_bytes() {
        assert_eq!(format_bytes_si(0u8), "0.00 B");
        assert_eq!(format_bytes_si(1u16), "1.00 B");
        assert_eq!(format_bytes_si(1024usize), "1.02 KB");
        assert_eq!(format_bytes_si(1_555_000), "1.55 MB");
        assert_eq!(format_bytes_si(1_555_000_000), "1.55 GB");
    }
}
