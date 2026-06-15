Nano Search
===

A sandbox for learning [Information Retrieval](https://en.wikipedia.org/wiki/Information_retrieval) by building a search engine from scratch.

Inspired by projects like [Lucene](https://github.com/apache/lucene), [Tantivy](https://github.com/quickwit-oss/tantivy) and [Meilisearch](https://github.com/meilisearch/meilisearch).

Prioritizes simplicity, readability, and experimentation over raw performance or production readiness.

Learning Resources
---

- Fundamentals: [Information Retrieval: Implementing and Evaluating Search Engines](https://www.amazon.com/Information-Retrieval-Implementing-Evaluating-Engines/dp/0262528878) by Buttcher
- Lucene internals: [Inside Apache Solr and Lucene: Algorithms and Engineering Deep Dive](https://www.amazon.com/dp/B0FVGK63XT) by Aliev
- Tantivy internals: Paul Masurel's [blog](https://fulmicoton.com/)
- Meilisearch internals: Clément Renault's [blog](https://blog.kerollmops.com/)

Features
---

- Pluggable search engine interface
  - `nano` - this implementation
  - `tantivy` - reference lexical engine built on [Tantivy](https://github.com/quickwit-oss/tantivy)
  - `vector` - reference semantic engine (see [lexical search vs semantic search](https://github.com/art-in/nano_search/issues/6))

- Index
  - [x] In-memory and on-disk modes
  - [x] Multi-segment index
  - [x] Multi-threaded indexing
  - Text analysis
    - [x] Stop-word removal
    - Tokenization
      - [x] Whitespace tokenization
      - [ ] Punctuation-aware tokenization
      - [ ] Unicode-aware tokenization
    - Normalization
      - [x] Lowercasing
      - [ ] Stemming
      - [ ] Lemmatization
  - Index compression
    - [x] Delta encoding and [bit packing](https://fulmicoton.com/posts/bitpacking/) for posting lists
    - [ ] [VByte / Varint](https://lemire.me/blog/2017/09/27/stream-vbyte-breaking-new-speed-records-for-integer-compression/) encoding for posting lists
    - [ ] [FST](https://burntsushi.net/transducers/) for term dictionary
    - [ ] Skip lists
  - Dynamic / incremental / [LSM](https://en.wikipedia.org/wiki/Log-structured_merge-tree)-based indexing
    - [ ] Background segment merging
    - [ ] Concurrent search during indexing
  - Document management
    - [x] Add
    - [ ] Update / Delete
  - Columnar storage
    - [ ] Field schemas
    - [ ] Doc values

- Search
  - Query types
    - [x] Term query
    - [ ] Boolean query (AND, OR, NOT)
    - [ ] Phrase query
    - [ ] Range query
  - Query expansion
    - [ ] Synonym expansion
    - [ ] Spelling correction
  - Ranking
    - [x] [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
    - [x] [BM25](https://en.wikipedia.org/wiki/Okapi_BM25)
    - [ ] Custom scorer
  - Faceting
    - [ ] Term facets
    - [ ] Range facets

- Vector / semantic search
  - [ ] [HNSW](https://en.wikipedia.org/wiki/Hierarchical_navigable_small_world) index
  - [ ] Hybrid retrieval (lexical + semantic fusion)

- Interfaces
  - [ ] CLI
  - [ ] HTTP API

- Evaluation
  - [x] [Precision / Recall](https://en.wikipedia.org/wiki/Precision_and_recall)
  - [x] [nDCG](https://en.wikipedia.org/wiki/Discounted_cumulative_gain)
  - [ ] MAP
  - [ ] MRR

- Datasets
  - [x] CISI
  - [x] Wikipedia
  - [x] [BEIR](https://github.com/beir-cellar/beir)

- Tooling
  - Performance
    - [x] Benchmarking
    - [x] Profiling
  - Testing
    - [x] Unit tests
    - [x] Integration tests
  - Debugging
    - [ ] Index inspection

- Documentation
  - [x] Benchmarking
  - [x] Profiling
  - [ ] Architecture
  - [ ] Installation / running
