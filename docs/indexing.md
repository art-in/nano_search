Indexing inside Docker container on macOS is slow
---

Indexing within a Docker development container on a macOS host is significantly slower compared to running the same process directly on the host system. For example, indexing 5 million wiki documents with the nano_search engine takes about 50 minutes inside the container, while it takes around 6.5 minutes on macOS natively - ~7.5× difference.

This slowdown is primarily due to Docker for macOS using I/O virtualization for communication between the guest and host OS, which is currently quite slow (see [Docker on MacOS is still slow? (2025)](https://www.paolomainardi.com/posts/docker-performance-macos-2025)). Since indexing is a disk I/O‑intensive operation, performance suffers considerably.

**Workaround:**  
To improve performance, avoid setting the target index directory inside `/workspaces/nano_search` (which is bound to the host filesystem and requires constant synchronization). Instead, use a directory like `/tmp`. For example, indexing the same 5 million wiki documents inside the container with the target directory set to `/tmp/index_nano` takes about 14 minutes — still ~2× slower than on the host, but ~3.5× faster than indexing inside the workspace directory.
