use anyhow::{Context, Result};
use colored::Colorize;

use crate::dataset_readers::utils::init_dataset_by_name;
use crate::engines::utils::engine_open_from_disk_by_names;
use crate::eval::evaluate_search_quality_for_query;

const SEARCH_LIMIT: u64 = 10;

pub fn search(engines: &[String], dataset: &str) -> Result<()> {
    println!("initializing search engines: {}", engines.join(","));
    println!("initializing dataset '{dataset}'");

    let engines = engine_open_from_disk_by_names(engines)?;
    let dataset = init_dataset_by_name(dataset)?;

    let query = dataset.queries()?.nth(10).context("should get query")?;

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
