CPU profiling
---

1. Build an optimized binary with debug info:

    ```sh
    cargo build --profile=profiling
    ```

2. Run the binary and record a profile with [samply](https://github.com/mstange/samply):

    ```sh
    samply record ./target/profiling/index
    ```

3. Open recorded profile in [Firefox Profiler](https://profiler.firefox.com/)

Limitations:

- Symbolication may not work (i.e. the profiler can show binary addresses instead of function names) when recording/serving a profile from inside a dev container, but works when profiling from host system.

- Off‑CPU samples are not collected when profiling inside a dev container (which is Linux) or on host Linux.  

    From [samply's readme](https://github.com/mstange/samply/blob/229c9e8ba442bff22bb2f5d205818bc7c9d7bb33/README.md?plain=1#L75): 
    > On macOS and Windows, both on- and off-cpu samples are collected (so you can see under which stack you were blocking on a lock, for example). On Linux, only on-cpu samples are collected at the moment.

