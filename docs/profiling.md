CPU profiling
---

1. Build optimized binary with debug info:

    ```sh
    cargo build --profile=profiling
    ```

2. Run binary, record profile with [samply](https://github.com/mstange/samply), open https://profiler.firefox.com/ in web browser and load recorded profile into it:

    ```sh
    samply record ./target/profiling/index
    ```

Limitations:

- symbolication doesn't work (i.e. profiler shows addressed instead of function names) when recording/serving profile from inside dev container, but works ok when profiling from host macos
- off-cpu samples are not collected when profiling insided dev container (which is Linux) or host Linux:
    > On macOS and Windows, both on- and off-cpu samples are collected (so you can see under which stack you were blocking on a lock, for example). On Linux, only on-cpu samples are collected at the moment.

    from [samply's readme](https://github.com/mstange/samply/blob/229c9e8ba442bff22bb2f5d205818bc7c9d7bb33/README.md?plain=1#L75)
