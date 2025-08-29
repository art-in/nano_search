Benchmarking
---

For benchmarking indexing/search routines we use [criterion.rs](https://github.com/bheisler/criterion.rs).

Run all benchmarks:

```sh
cargo bench
```

Run benchmark, which ID contrains substring

```sh
cargo bench -- <substring_regexp>
```

Limitations:

- results are highly unstable, for both memory and disk indexing/searching, inside container and in host system

