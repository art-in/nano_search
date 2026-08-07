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

- Indexing
  - [x] In-memory and on-disk modes
  - [x] Multi-segment index
  - [x] Multi-threaded indexing
  - [x] Streaming / iterator-based indexing
  - [ ] Skip lists (for fast posting-list seeking)
  - [ ] Field schemas (per-field store/index/tokenize config)
  - [ ] Columnar storage (for fast per-field sorting, filtering, faceting)
  - Compression
    - [x] Delta encoding and [bit packing](https://fulmicoton.com/posts/bitpacking/) for posting lists
    - [ ] [VByte / Varint](https://lemire.me/blog/2017/09/27/stream-vbyte-breaking-new-speed-records-for-integer-compression/) encoding for posting lists
    - [ ] [FST](https://burntsushi.net/transducers/) for term dictionary
  - Dynamic / incremental / [LSM](https://en.wikipedia.org/wiki/Log-structured_merge-tree)-based indexing
    - [ ] Background segment merging
    - [ ] Concurrent search during indexing
  - Document management
    - [x] Add
    - [ ] Update / Delete

- Search
  - [ ] Multi-threaded search
  - [ ] Query result caching
  - [ ] Pagination
  - [ ] Snippets / highlighting
  - [ ] Structured filtering (exact/range filters)
  - [ ] Sort by field (bypass relevance ranking)
  - Queries
    - [x] Term / bag-of-words query
    - [x] Boolean query (AND, OR, NOT)
    - [ ] Phrase query
    - [ ] Range query
    - [ ] Fuzzy / typo-tolerant query
    - [ ] Prefix query / autocomplete
  - Query expansion
    - [ ] Synonym expansion
    - [ ] Spelling correction
  - Ranking
    - [x] [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
    - [x] [BM25](https://en.wikipedia.org/wiki/Okapi_BM25)
    - [ ] Field boosting
    - [ ] Custom ranking functions
  - Faceting
    - [ ] Term facets
    - [ ] Range facets

- Text analysis (for both indexing and search)
  - [x] Stop-word removal
  - Tokenization
    - [x] Whitespace tokenization
    - [ ] Unicode-aware tokenization
    - [ ] Punctuation-aware tokenization
  - Normalization
    - [x] Lowercasing
    - [ ] Stemming
    - [ ] Lemmatization

- Vector / semantic search
  - [ ] [HNSW](https://en.wikipedia.org/wiki/Hierarchical_navigable_small_world) index
  - [ ] Hybrid retrieval (lexical + semantic fusion)

- Interfaces
  - [x] CLI
  - [ ] HTTP API

- Evaluation
  - [x] [Precision / Recall](https://en.wikipedia.org/wiki/Precision_and_recall)
  - [x] [nDCG](https://en.wikipedia.org/wiki/Discounted_cumulative_gain)
  - [ ] MAP
  - [ ] MRR

- Dataset readers
  - For performance benchmarks and quality evaluation
    - [x] [CISI](https://www.kaggle.com/datasets/dmaso01dsta/cisi-a-dataset-for-information-retrieval)
    - [x] [Wikipedia](https://dumps.wikimedia.org/)
    - [x] [BEIR](https://github.com/beir-cellar/beir)
  - For indexing local files
    - [ ] txt
    - [ ] html
    - [ ] pdf
    - [ ] docx

- Tooling
  - Testing
    - [x] Unit tests
    - [x] Integration tests
  - Performance
    - [x] Benchmarking
    - [x] Profiling
  - Debugging
    - [ ] Index inspection

- Documentation
  - [x] Benchmarking
  - [x] Profiling
  - [ ] Architecture
  - [ ] Index format
  - [ ] Query language
  - [ ] Installation / running
