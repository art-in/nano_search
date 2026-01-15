This is an opinionated guide to CPU and memory profiling.

There are many profilers available; most do not require specific configurations from the target binary. They all utilize stack unwinding and symbolication, but differ in how they "inject" into a running process and how they later visualize the recorded profile.


Most profilers work with release builds, but using a "profiling" build ensures the process is fast (matching release build efficiency), accurate, and feature-rich (e.g., mapping function names to actual source code lines). See [Cargo.toml](../Cargo.toml) for details on the "profiling" build profile.

So this project is not strictly tied to any specific profiler. Below just the tools I prefer to use in macOS. For details on why these were chosen and other profilers I have tested, see https://github.com/art-in/nano_search/issues/5.

Profiling inside the dev container
---

Unfortunately profiling cannot be reliable done inside dev container, if you run it on macOS host (and I expect the same to be on Windows host). Situation may be better on Linux host.

Most likely you will have to install and run all required rust build tools and profilers directly on host system. Main reasons for that:
1. Disk IO inside container is slow (see [docs/indexing](indexing.md)), so CPU/time profiles recorded inside container will probably show inaccurate picture.
2. Your favourite profiler may not work or perform worse in Linux. E.g. you can't run XCode Instruments in Linux container, or you can record profile with Heaptrack in Linux container, but you have no GUI in your host system to visualize it, or you can run samply in Linux container, but it doesn't record off-CPU work there, etc.

CPU profiling
---

Install: [samply](https://github.com/mstange/samply).

1. Build the binary:

    ```sh
    cargo build --profile=profiling
    ```

2. Run and record a profile:

    ```sh
    samply record ./target/profiling/nano_search --engines=nano --dataset=cisi index
    ```

3. Open recorded profile in [Firefox Profiler](https://profiler.firefox.com/)

Limitations:

- Symbolication may not work (i.e. the profiler can show binary addresses instead of function names), when recording/serving a profile from inside a dev container, but works when profiling from host system.

- Off‑CPU samples are not recorded in Linux (and thus in dev container)

    From [samply's readme](https://github.com/mstange/samply/blob/229c9e8ba442bff22bb2f5d205818bc7c9d7bb33/README.md?plain=1#L75):

    > On macOS and Windows, both on- and off-cpu samples are collected (so you can see under which stack you were blocking on a lock, for example). On Linux, only on-cpu samples are collected at the moment.

Memory profiling on macOS
---

Install: (1) XCode from appstore, (2) [cargo-instruments](https://github.com/cmyr/cargo-instruments).


1. Build, run and record profile:

    ```sh
    cargo instruments --profile=profiling --template=alloc --time-limit=30000 -- --engines=nano --dataset=cisi index
    ```

