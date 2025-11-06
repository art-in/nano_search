Reader for [BEIR](https://github.com/beir-cellar/beir)-formatted dataset.

Each dataset consists of following files:

- `qrels/`
    - `test.tsv` - lines of tab-separated values (query-id, corpus-id, score)
    - `dev.tsv` (optional)
    - `train.tsv` (optional)
- `corpus.jsonl` - lines of json objects (_id, title, text, metadata)
- `queries.jsonl` - lines of json objects (_id, text, metadata)
