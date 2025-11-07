Benchmarking
---

For benchmarking index/search routines we use [criterion.rs](https://github.com/bheisler/criterion.rs).

Benchmarks are located in [benches/](../benches).

Run all benchmarks:

```sh
cargo bench
```

Run benchmarks whose ID contains a substring:

```sh
cargo bench -- <substring_regexp>
```

Limitations:

- Results can be unstable (for both memory and disk indexing/searching), in a dev container and on the host system.
