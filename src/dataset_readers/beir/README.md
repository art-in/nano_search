Reader for [BEIR](https://github.com/beir-cellar/beir)-formatted dataset.

Dataset itself may be downloaded in two ways:

- Manually

    1. Download from https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets
    1. Decompress and place into datasets/ dir
    1. Initialize with `BeirDatasetReader::from_dir(dir)`

- Automatically

    1. Initialize with `BeirDatasetReader::from_hf(dataset_name)`  

        Use [BEIR-Name](https://github.com/beir-cellar/beir?tab=readme-ov-file#beers-available-datasets) as a dataset name

        Dataset will be downloaded from https://huggingface.co/BeIR/datasets and decompressed automatically  

Each dataset consists of following files:

- `qrels/`
    - `test.tsv` - lines of tab-separated values (query-id, doc-id, score)
    - `dev.tsv` (optional)
    - `train.tsv` (optional)
- `corpus.jsonl` - lines of json objects with documents (_id, title, text, metadata)
- `queries.jsonl` - lines of json objects with queries (_id, text, metadata)
