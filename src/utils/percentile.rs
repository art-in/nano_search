use anyhow::{Context, Result};

pub trait GetPercentile {
    fn perc(&self, percentile: f64) -> Result<f64>;
}

impl GetPercentile for inc_stats::Percentiles<f64> {
    fn perc(&self, percentile: f64) -> Result<f64> {
        self.percentile(percentile)?
            .context("percentile data should not be empty")
    }
}
