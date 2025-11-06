#!/bin/sh

curl --output data.zip "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/nfcorpus.zip"

unzip data.zip
mv nfcorpus data
