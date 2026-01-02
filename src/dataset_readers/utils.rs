use anyhow::{Context, Result, bail};

use super::BeirDatasetReader;
use super::cisi::CisiDatasetReader;
use super::json::JsonDatasetReader;
use super::model::Dataset;
use crate::dataset_readers::WikiDatasetReader;

pub fn init_dataset_by_name(dataset_name: &str) -> Result<Box<dyn Dataset>> {
    let dataset: Box<dyn Dataset> = match dataset_name {
        "cisi" => Box::new(CisiDatasetReader::new("datasets/cisi")),

        // wikipedia
        "enwiki" => {
            Box::new(WikiDatasetReader::new("datasets/enwiki/dump.xml.bz2")?)
        }
        "simplewiki" => Box::new(WikiDatasetReader::new(
            "datasets/simplewiki/dump.xml.bz2",
        )?),
        "enwiki_json" => {
            Box::new(JsonDatasetReader::new("datasets/enwiki_json/wiki.json"))
        }

        // BEIR
        "beir_nfcorpus" => {
            // "nfcorpus" has unsual format on huggingface, so load it from
            // dir instead as a special case. see:
            // https://huggingface.co/datasets/BeIR/nfcorpus/discussions/5
            Box::new(BeirDatasetReader::from_dir(
                "datasets/beir_nfcorpus/data",
            )?)
        }
        n if n.starts_with("beir_") => {
            let dataset_name = n.strip_prefix("beir_").context("stripped")?;
            Box::new(BeirDatasetReader::from_hf(dataset_name)?)
        }
        _ => bail!("unknown dataset '{dataset_name}'"),
    };
    Ok(dataset)
}
