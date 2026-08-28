use anyhow::{Result, bail};

/// Unwraps error message out of [`anyhow::Result`]
pub fn err<T>(target: Result<T>) -> Result<String> {
    match target {
        Ok(_) => bail!("should be an error"),
        Err(message) => Ok(message.to_string()),
    }
}
