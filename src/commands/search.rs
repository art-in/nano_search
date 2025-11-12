use anyhow::{Context, Result};
use colored::Colorize;

use super::common::{init_dataset, init_search_engines_open};
use crate::eval::evaluate_search_quality_for_query;
use crate::eval::model::QueriesSource;

const SEARCH_LIMIT: u64 = 10;

pub fn search_command() -> Result<()> {
    let engines = init_search_engines_open()?;
    let dataset = init_dataset();

    let query = dataset.queries().nth(10).context("should get query")?;

    println!("query (id={}): {}", query.id, query.text);
    println!("relevant docs: {:?}", query.relevant_docs);

    for engine in engines {
        println!("searching with {} engine", engine.get_name().red());
        let found_docids = engine.search(&query.text, SEARCH_LIMIT)?;

        println!("found docids: {:?}", found_docids);

        let quality = evaluate_search_quality_for_query(
            &found_docids,
            &query.relevant_docs,
            SEARCH_LIMIT,
        )?;

        println!("search limit: {SEARCH_LIMIT}");
        println!("precision: {:.1}%", quality.precision * 100.0);
        println!("recall   : {:.1}%", quality.recall * 100.0);
        println!("NDCG     : {:.1}%", quality.ndcg * 100.0);
    }

    Ok(())
}
