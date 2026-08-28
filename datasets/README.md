Datasets with document texts and search queries, used for measuring and
comparing performance and search quality of target search engines.

Small datasets, like "cisi", are committed directly into the repository.
These are best used in unit tests and benchmarks, since they require no
downloading/preparation and do not pollute macro-benchmarks with network
activity.

Bigger datasets should be downloaded manually by running the corresponding
`download.sh` script. The [beir](../crates/ns_datasets/src/beir)
dataset reader is also able to auto-download one of the BEIR datasets from
HuggingFace.

