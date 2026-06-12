Nano Search
===

An experimental search engine written in Rust for learning how search engines work.

This is a sandbox for implementing and experimenting with information retrieval techniques inspired by projects like [Tantivy](https://github.com/quickwit-oss/tantivy), [Meilisearch](https://github.com/meilisearch/meilisearch), and [Lucene](https://github.com/apache/lucene). The focus is on understanding ideas by building them from scratch, measuring their impact, and comparing the results.

The project prioritizes simplicity, readability, and experimentation over raw performance or production readiness.

Goals
---

- Learn how modern search engines are built.
- Experiment with indexing and retrieval algorithms.
- Compare search quality and performance against existing engines.
- Keep implementation simple, readable, and easy to modify.

Features
---

- [x] Pluggable search engine interface
  - `nano` - this implementation
  - `tantivy` - reference lexical engine built on [Tantivy](https://github.com/quickwit-oss/tantivy)
  - `vector` - reference semantic engine (see [lexical search vs semantic search](https://github.com/art-in/nano_search/issues/6))
- [x] [LSM](https://en.wikipedia.org/wiki/Log-structured_merge-tree)-based inverted index for lexical search
  - [x] On-disk and in-memory modes
  - [x] Multi-segment index
  - [x] Multi-threaded indexing
- [ ] Text analysis
  - [x] Tokenization
  - [x] Stop-word removal
  - [ ] Normalization
    - [ ] Stemming
    - [ ] Lemmatization
- [ ] Query types
  - [x] Term query
  - [ ] Boolean query (AND, OR, NOT)
  - [ ] Phrase query
  - [ ] Range query
- [x] Ranking
  - [x] [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
  - [x] [BM25](https://en.wikipedia.org/wiki/Okapi_BM25)
- [ ] Index compression
  - [x] Delta encoding and [bit packing](https://fulmicoton.com/posts/bitpacking/) for posting lists
  - [ ] [FST](https://burntsushi.net/transducers/) for term dictionary
  - [ ] Skip lists
- [ ] Document operations
  - [x] Add
  - [ ] Update / Delete
- [ ] Dynamic indexing
  - [ ] Background segment merging
- [ ] HTTP API
- [x] Search quality evaluation
  - [x] Precision
  - [x] Recall
  - [x] [nDCG](https://en.wikipedia.org/wiki/Discounted_cumulative_gain)
- [x] Dataset readers
  - [x] CISI
  - [x] Wikipedia
  - [x] [BEIR](https://github.com/beir-cellar/beir)
- [x] Benchmarking and profiling
- [x] Regression tests
- [ ] Documentation
    - [x] Benchmarking
    - [x] Profiling
    - [ ] Architecture
    - [ ] How to install/run
- [ ] Vector search
  - [ ] [HNSW](https://en.wikipedia.org/wiki/Hierarchical_navigable_small_world)
  - [ ] Hybrid retrieval (lexical + semantic fusion)

