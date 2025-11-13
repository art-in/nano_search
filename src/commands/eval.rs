use std::time::Instant;

use anyhow::Result;
use colored::Colorize;

use super::common::{init_dataset, init_search_engines_open};
use crate::eval::evaluate_search_quality;
use crate::eval::model::{QueriesSource, Query, SearchQuality};
use crate::model::engine::SearchEngine;
use crate::utils::GetPercentile;

pub fn eval_command() -> Result<()> {
    let engines = init_search_engines_open()?;
    let dataset = init_dataset()?;

    for engine in engines {
        evaluate(engine.as_ref(), &mut dataset.queries()?)?;
    }

    Ok(())
}

fn evaluate(
    engine: &dyn SearchEngine,
    queries: &mut dyn Iterator<Item = Query>,
) -> Result<()> {
    let now = Instant::now();
    println!("evaluating {} engine...", engine.get_name().red());
    let quality = evaluate_search_quality(queries, engine, 10)?;
    println!(
        "evaluating {} engine... done in {:.1} seconds",
        engine.get_name(),
        now.elapsed().as_secs_f32()
    );

    println!("search quality for {} engine:", engine.get_name());
    print_quality(&quality)?;

    Ok(())
}

fn print_quality(quality: &SearchQuality) -> Result<()> {
    println!("queries count: {}", quality.queries_count);

    println!(
        "{:<10}{:<3}:  avg={:<6.3} p50={:<6.2} p90={:<6.2} p100={:<6.2}",
        "Precision",
        format!("@{}", quality.search_limit),
        quality.precision_avg,
        quality.precisions.perc(0.5)?,
        quality.precisions.perc(0.9)?,
        quality.precisions.perc(1.0)?
    );

    println!(
        "{:<10}{:<3}:  avg={:<6.3} p50={:<6.2} p90={:<6.2} p100={:<6.2}",
        "Recall",
        format!("@{}", quality.search_limit),
        quality.recall_avg,
        quality.recalls.perc(0.5)?,
        quality.recalls.perc(0.9)?,
        quality.recalls.perc(1.0)?
    );

    println!(
        "{:<10}{:<3}:  avg={}",
        "nDCG",
        format!("@{}", quality.search_limit),
        format!("{:<6.3}", quality.ndcg_avg).cyan()
    );

    Ok(())
}
