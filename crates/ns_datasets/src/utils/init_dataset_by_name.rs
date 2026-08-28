use anyhow::{Context, Result, bail};

use crate::beir::BeirDatasetReader;
use crate::cisi::CisiDatasetReader;
use crate::json::JsonDatasetReader;
use crate::model::Dataset;
use crate::wiki::WikiDatasetReader;

pub fn init_dataset_by_name(dataset_name: &str) -> Result<Box<dyn Dataset>> {
    let datasets_dir = format!(
        "{crate_dir}/../../datasets",
        crate_dir = env!("CARGO_MANIFEST_DIR")
    );

    let dataset: Box<dyn Dataset> = match dataset_name {
        "cisi" => Box::new(CisiDatasetReader::new(datasets_dir + "/cisi")),

        // wikipedia
        "enwiki" => Box::new(WikiDatasetReader::new(
            datasets_dir + "/enwiki/dump.xml.bz2",
        )?),
        "simplewiki" => Box::new(WikiDatasetReader::new(
            datasets_dir + "/simplewiki/dump.xml.bz2",
        )?),
        "enwiki_json" => Box::new(JsonDatasetReader::new(
            datasets_dir + "/enwiki_json/wiki.json",
        )),

        // BEIR
        "nfcorpus" => Box::new(BeirDatasetReader::from_dir(
            datasets_dir + "/nfcorpus/data",
        )?),
        "scifact" => Box::new(BeirDatasetReader::from_dir(
            datasets_dir + "/scifact/data",
        )?),
        n if n.starts_with("beir_") => {
            let dataset_name = n.strip_prefix("beir_").context("stripped")?;
            Box::new(BeirDatasetReader::from_hf(dataset_name)?)
        }
        _ => bail!("unknown dataset '{dataset_name}'"),
    };
    Ok(dataset)
}
