# CISI (a dataset for Information Retrieval)

A public dataset from the University of Glasgow's Information Retrieval Group

## Context

This is a text-based dataset that can be used for Information Retrieval (IR). It is publicly available from the University of Glasgow (http://ir.dcs.gla.ac.uk/resources/test_collections/cisi/).

## Content

The data were collected by the Centre for Inventions and Scientific Information ("CISI") and consist of text data about 1,460 documents and 112 associated queries. Its purpose is to be used to build models of information retrieval where a given query will return a list of document IDs relevant to the query. The file "CISI.REL" contains the correct list (ie. "gold standard" or "ground proof") of query-document matching and your model can be compared against this "gold standard" to see how it has performed.

## Acknowledgements

As mentioned above, this dataset has been made publicly available by the Information Retrieval Group at the University of Glasgow (https://www.gla.ac.uk/schools/computing/research/researchsections/ida-section/informationretrieval/).

## Inspiration

How close can your algorithm get to the gold standard in CISI.REL?

## Files

- CISI.ALL (2.23 MB)

    A file of 1,460 "documents" each with a unique ID (.I), title (.T), author (.A), abstract (.W) and list of cross-references to other documents (.X). It is the dataset for training IR models when used in conjunction with the Queries (CISI.QRY).

- CISI.QRY (68.16 kB)

    A file containing 112 queries each with a unique ID (.I) and query text (.W).

- CISI.REL (80.96 kB)

    A file containing the mapping of query ID (column 0) to document ID (column 1). A query may map to more than one document ID. This file contains the "ground truth" that links queries to documents. Use this to train and test your algorithm.

## Source

Downloaded from: https://www.kaggle.com/datasets/dmaso01dsta/cisi-a-dataset-for-information-retrieval
