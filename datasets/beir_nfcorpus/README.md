dataset homepage: https://www.cl.uni-heidelberg.de/statnlpgroup/nfcorpus/

originally found in https://github.com/beir-cellar/beir#beers-available-datasets

smallest BEIR dataset

normally BEIR datasets should be auto-loaded from huggingface.co with
`BeirDatasetReader::from_hf("nfcorpus")`, but this is the only dataset
that has unusual format there, so load from homepage instead.
see: https://huggingface.co/datasets/BeIR/nfcorpus/discussions/5
