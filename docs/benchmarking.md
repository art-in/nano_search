Benchmarking measures the performance/efficiency of the search engine, while search quality/effectiveness is measured by a separate evaluation procedure. The overall goal is to achieve high efficiency and effectiveness simultaneously - maximizing search quality while minimizing execution time and hardware resource usage.

Most interesting performance metrics are:
- Index and search execution time
- Peak RAM usage
- Final index disk size

Main questions benchmarking should answer:
- How did my last code change affect performance?
    - Did it improve or degrade, and by how much?
    - Is a performance degradation small enough to be a reasonable tradeoff for adding a feature?
- How does the performance of one search engine compare against another?
    - E.g. how much nano_search is slower than tantivy?

How to Run
---

Run benchmarks with stdout capturing disabled:

```sh
cargo bench -- --nocapture
```

Metrics Description
---

Callgrind (CPU & Cache)

- `Instructions` - total count of simulated CPU instructions executed in user-space code.
- `L1 Hits` - memory accesses resolved by the simulated L1 CPU cache.
- `LL Hits` - memory accesses resolved by the simulated last-level CPU cache after lower-level cache misses.
- `RAM Hits` - memory accesses that missed CPU caches and required simulated RAM access.
- `Total read+write` - total simulated memory read and write operations (L1+LL+RAM)
- `Estimated Cycles` - estimated CPU cycles derived from Valgrind's simulation model, not real hardware counter.

DHAT (Heap Allocation)

- `Total bytes` - total heap-allocated bytes.
- `Total blocks` - total heap allocation operations performed.
- `At t-gmax bytes` - peak simultaneously allocated heap memory.
- `At t-gmax blocks` - number of simultaneously allocated heap blocks at peak memory usage.
- `At t-end bytes` - heap memory still allocated at program termination.
- `At t-end blocks` - heap blocks still allocated at program termination.

Harness: Gungraun vs. Criterion
---

To benchmark search engines, we use [Gungraun](https://github.com/gungraun/gungraun), a [Valgrind](https://valgrind.org)-based benchmark harness for Rust.

Previously, we used [Criterion.rs](https://github.com/bheisler/criterion.rs), which relies on wall-clock time. Wall-clock benchmarking is more sensitive to OS noise and typically requires many repetitions for stable macro-benchmark results, which makes it very slow. Thus Criterion is generally more suitable for micro-benchmarking, while this project primarily requires macro-benchmarks - integration-level benchmarks tracking entire multi-threaded indexing and search routines.

Gungraun measures simulated CPU instruction and cycle counts. While not 100% deterministic due to system-level boundary crossings, it is significantly more stable and reproducible than wall-clock time. Also it runs much faster, since it doesn't have to run same procedure multiple times.

Metric Fluctuation
---

CPU instruction counts, cycle estimations, and RAM allocations can still fluctuate slightly between identical consecutive runs. This is driven by some non-deterministic execution factors, for example:

- **Standard Library HashMaps:** The memory index uses the standard library's `HashMap` for term dictionaries. This collection relies on a randomized, non-deterministic hash seed to prevent collision attacks. Keys distribute into memory buckets differently on each run, creating slightly different cache hit/miss profiles.

- **Disk I/O and Thread Starvation:** Search engines are heavily bound by disk operations, making them vulnerable to "noisy neighbor" disruptions on the host system. Valgrind simulates user-space code, but does not emulate the physical disk or kernel actions. Our indexer uses `crossbeam` channels to stream documents across threads; if a thread starves because a disk read stalls, it executes atomic spin-lock cycles (`backoff.spin()`) before falling into a full condition-variable sleep. Valgrind captures these extra spin instructions, meaning disk latency variations directly alter instruction counts.

Because metrics naturally fluctuate within a narrow margin, we have configured a threshold so minor variations are ignored and not highlighted as metric changes.

Macro-Benchmarking Scope
---

Gungraun supports both library and binary benchmarks. Library benchmarks focus heavily on specific functions, making them ideal for micro-benchmarks.

However, in a multi-threaded macro-benchmark tracking background indexer pipelines, focusing on specific functions requires invasive annotations or explicit client API toggles (`callgrind::toggle_collect()`). This is inconvenient, forces us to pollute production code, and requires remembering to initialize tracking on every newly spawned thread.

To avoid this complexity, we utilize global collection by disabling the benchmark entry point in library benchmarks or by using binary benchmarks. The downside is that we collect metrics from unrelated runtime code, such as Gungraun's own wrapper logic and binary CLI argument parsing. However, capturing this noise is preferable to accidentally omitting in-thread work completely. Ultimately, absolute metric values do not matter; what matters more is the relative difference between runs.

Disk Metrics
---

We are interested in minimizing on-disk size of generated index files.

Unfortunately, because Valgrind operates strictly in user-space, Gungraun cannot natively collect or diff disk usage metrics, nor does it support custom metrics.

As a workaround, index size is printed to stdout by the CLI `index` command, and exposed in benchmarks with `--nocapture` arg.

