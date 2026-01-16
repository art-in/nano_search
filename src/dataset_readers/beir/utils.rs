use std::hash::{DefaultHasher, Hasher};

use anyhow::{Context, Result};

/// Parses document/query ID strings to integer.
///
/// Different BEIR datasets have different ID formats, e.g.:
/// - as numeric string ("1045")
/// - as numeric string with prefix ("MED-911")
/// - as random text string ("632589828c8b9fca2c3a59e97451fde8fa7d188d")
/// - as structured string ("test-environment-aeghhgwpe-pro02b")
/// - as structured numeric string ("4d3d4471-2019-04-18T11:45:01Z-00002-000")
///
/// It tries to extract numeric representation where possible, otherwise
/// fallback to hashing
pub fn parse_id(input: &str) -> Result<u64> {
    let id_str = if input.matches('-').count() == 1 {
        input.rsplit('-').next().context("should take last part")?
    } else {
        input
    };

    id_str.parse::<u64>().map_or_else(
        |_| {
            let mut hasher = DefaultHasher::new();
            hasher.write(input.as_bytes());
            Ok(hasher.finish())
        },
        Ok,
    )
}

/// Extracts a string field value from a JSON object.
pub fn extract_string<'a>(
    json: &'a serde_json::Map<String, serde_json::Value>,
    field_name: &str,
) -> Result<&'a str> {
    json.get(field_name)
        .with_context(|| format!("{field_name} field should exist"))?
        .as_str()
        .with_context(|| format!("{field_name} field should be a string"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id() -> Result<()> {
        // extract numeric representation
        assert_eq!(parse_id("1045")?, 1045);
        assert_eq!(parse_id("MED-911")?, 911);

        // fallback to hashing
        assert_eq!(
            parse_id("632589828c8b9fca2c3a59e97451fde8fa7d188d")?,
            2_277_164_672_189_146_571
        );
        assert_eq!(
            parse_id("test-environment-aeghhgwpe-pro02b")?,
            17_880_531_886_978_462_505
        );
        assert_eq!(
            parse_id("4d3d4471-2019-04-18T11:45:01Z-00002-000")?,
            16_211_700_675_218_413_973
        );

        Ok(())
    }
}
