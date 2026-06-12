Benchmarking
===

Benchmarking measures search engine performance (efficiency), while search quality (effectiveness) is measured separately by the evaluation procedure. The goal is to achieve both high efficiency and high effectiveness by maximizing search quality while minimizing resource usage.

The main performance metrics are:

- Indexing and search time
- Peak RAM usage
- Final index disk size

Main questions benchmarking should answer:

- How did the last code change affect performance?
  - Did it improve or degrade performance? By how much?
  - Is a performance regression small enough for the added feature?
- How does one search engine compare to another?
  - For example, how much slower is nano_search than tantivy?

How to Run
---

Run benchmarks with stdout enabled:

```sh
cargo bench -- --nocapture
```

Metrics
---

Callgrind (CPU & Cache)

| Metric | Description |
|--------|-------------|
| `Instructions`     | Total count of CPU instructions executed in user-space code. |
| `L1 Hits`          | Memory accesses resolved by the simulated L1 CPU cache. |
| `LL Hits`          | Memory accesses resolved by the simulated last-level CPU cache after lower-level cache misses. |
| `RAM Hits`         | Memory accesses that missed CPU caches and required simulated RAM access. |
| `Total read+write` | Total simulated memory read and write operations (L1+LL+RAM). |
| `Estimated Cycles` | Estimated CPU cycles derived from Valgrind's simulation model, not real hardware counters. |

DHAT (Heap Allocation)

| Metric | Description |
|--------|-------------|
| `Total bytes`      | Total heap-allocated bytes. |
| `Total blocks`     | Total heap allocation operations performed. |
| `At t-gmax bytes`  | Peak simultaneously allocated heap memory. |
| `At t-gmax blocks` | Peak simultaneously allocated heap blocks. |
| `At t-end bytes`   | Heap memory still allocated at program termination. |
| `At t-end blocks`  | Heap blocks still allocated at program termination. |

Harness: Wall-Clock vs. User-Space Instrumentation
---

We use [Gungraun](https://github.com/gungraun/gungraun), a [Valgrind](https://valgrind.org)-based benchmark harness for Rust.

Previously we used [Criterion.rs](https://github.com/bheisler/criterion.rs), which measures wall-clock time. Wall-clock benchmarks are affected by OS noise and require many repetitions to produce stable macro-benchmark results. This makes them slow. Criterion is better suited for micro-benchmarks, while this project mainly needs macro-benchmarks that measure complete indexing and search pipelines.

Gungraun measures user-space instruction counts, memory accesses, heap allocations, and estimated CPU cycles. It is much more stable than wall-clock timing and faster because it does not require repeated runs.

A limitation of the Gungraun/Valgrind approach is that it measures only user-space execution. It does not capture system I/O. As a result, changes that reduce system I/O may have little effect on reported metrics while significantly improving wall-clock time. For example, removing frequent `Write::flush()` calls reduced indexing time by more than 3x while changing instruction and cycle counts by only ~1%.

There is no perfect benchmarking approach.
- Wall-clock benchmarks measure the complete execution, but are unstable and slow due to repeated runs.
- Instrumentation-based benchmarks are stable, but are slow too due to instrumentation and cannot measure system I/O.

Recommended workflow:

- Run instrumentation-based benchmarks for regression tracking with `cargo bench`.
- Run indexing and evaluation on large datasets (e.g. `msmarco` or `enwiki`):

    ```sh
    cargo run --release -- --engines=nano --dataset=beir_msmarco index
    cargo run --release -- --engines=nano --dataset=beir_msmarco eval
    ```

    These commands report wall-clock execution time at the end.

Metric Fluctuation
---

Instruction counts, estimated cycles, and heap allocations can still vary slightly between identical runs.

Sources of variation include:

- Standard Library HashMap. The in-memory term dictionary uses `HashMap`, which randomizes its hash seed to prevent collision attacks. This changes bucket placement between runs and slightly changes cache behavior.

- Disk I/O and thread starvation. Search engines spend much of their time waiting for disk I/O. If a worker thread stalls on disk access, it repeatedly executes `backoff.spin()` before blocking on a condition variable. Valgrind counts these extra instructions, so changes in disk latency also change instruction counts.

Small metric changes are expected, so benchmarks are configured to ignore changes below a small threshold.

Macro-Benchmark Scope
---

Gungraun supports both library and binary benchmarks.

Library benchmarks are ideal for micro-benchmarks, because they can measure individual functions.

Macro-benchmarks are different. Code executed inside threads is not measured by default. To measure work performed by threads, you must explicitly annotate thread code with `callgrind::toggle_collect()` or specify functions to watch. This is inconvenient and easy to forget when adding new threads.

To avoid this complexity, we use global collection by disabling the benchmark entry point or by using binary benchmarks. Downside is that it also collects unrelated runtime code, such as Gungraun's wrapper logic and CLI argument parsing. But this extra noise is preferable to accidentally missing work performed by background threads. Absolute metric values are less important than the difference between benchmark runs.

Disk Metrics
---

We are interested in minimizing on-disk size of generated index files.

Unfortunately, because Valgrind operates strictly in user-space, Gungraun cannot natively collect or diff disk usage metrics, nor does it support custom metrics.

As a workaround, index size is printed to stdout by the CLI `index` command, and exposed in benchmarks with `--nocapture` arg.

