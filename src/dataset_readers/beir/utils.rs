use anyhow::{Context, Result};

/// Parses document/query ID strings to integer.
///
/// Different BEIR datasets have different ID formats, e.g.:
/// - as numeric string ("1045")
/// - as numeric string with prefix ("MED-911")
pub fn parse_id(input: &str) -> Result<u64> {
    if input.contains("-") {
        Ok(input
            .split('-')
            .next_back()
            .context("should take last part")?
            .parse::<u64>()?)
    } else {
        Ok(input.parse::<u64>()?)
    }
}
