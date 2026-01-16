use clap::builder::PossibleValuesParser;
use clap::{Parser, Subcommand};

use crate::engines::utils::get_all_engine_names;

#[derive(Debug, Parser)]
#[command(
    disable_help_subcommand = true,
    after_help = get_examples_string())]
pub struct Args {
    #[arg(
        long,
        value_delimiter = ',',
        default_values_t = get_all_engine_names(),
        value_parser = PossibleValuesParser::new(get_all_engine_names()),
        help = "Target search engines"
    )]
    pub engines: Vec<String>,

    #[arg(
        long,
        help = "Source of documents and evaluation queries. E.g.: cisi, \
                simplewiki, beir_scifact, etc."
    )]
    pub dataset: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Index documents")]
    Index {
        #[arg(
            long,
            default_value = None,
            help = "Number of threads to use for indexing. By default equals \
                    to number of available CPU cores."
        )]
        threads: Option<usize>,
    },

    #[command(about = "Evaluate search quality")]
    Eval,

    #[command(about = "Perform a search with a single query")]
    Search,
}

const fn get_examples_string() -> &'static str {
    "Examples:

    # minimal
    ./nano_search --dataset=cisi index

    # complex
    ./nano_search --engines=nano,tantivy --dataset=cisi index --threads=1
    ./nano_search --engines=nano,tantivy --dataset=cisi eval

    # with 'cargo run' (notice app options go after double-dash '--' separator)
    cargo run -- --engines=nano,tantivy --dataset=cisi index"
}
