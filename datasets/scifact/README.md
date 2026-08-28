Dataset homepage: https://github.com/allenai/scifact

Originally found in https://github.com/beir-cellar/beir#beers-available-datasets

One of the smallest BEIR datasets.

This dataset looks optimal for benchmarks - not too large and not too small.

Committing it into the repo to avoid auto-loading from huggingface.co, so bench
results are not poluted with
- network load (when loading it at the first time running bench)
- and parquet parsing (which appears to be noticiable, comparing to jsonl)

