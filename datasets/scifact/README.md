dataset homepage: https://github.com/allenai/scifact

originally found in https://github.com/beir-cellar/beir#beers-available-datasets

one of the smallest BEIR datasets.

this dataset looks optimal for benchmarks - not too large and not too small.

committing it into the repo to avoid auto-loading from huggingface.co, so bench
results are not poluted with
- network load (when loading it at the first time running bench)
- and parquet parsing (which appears to be noticiable, comparing to jsonl)

