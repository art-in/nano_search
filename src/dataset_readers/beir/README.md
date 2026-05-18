Reader for [BEIR](https://github.com/beir-cellar/beir)-formatted dataset.

Dataset may be downloaded in two ways:

- Manually

    1. Download zip file from https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets
    1. Decompress and place into datasets/dataset_name/ dir
    1. Initialize reader with `BeirDatasetReader::from_dir("datasets/dataset_name")`

- Automatically

    1. Initialize with `BeirDatasetReader::from_hf(dataset_name)`  

        Use [BEIR-Name](https://github.com/beir-cellar/beir?tab=readme-ov-file#beers-available-datasets) as a dataset name.  

        Dataset will be downloaded from https://huggingface.co/BeIR/datasets and decompressed automatically.  

Each dataset consists of following parts:

- corpus - source documents
- queries - search queries
- qrels - relevance score of certain documents for certain queries

