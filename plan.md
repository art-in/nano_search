ideas to what implement next:

---

compare with elastic

1. [done] find good dataset with text articles
1. [done] parse and clean dataset
    1. find xml parser for rust
        - https://github.com/RazrFalcon/roxmltree
        - https://github.com/DrPlantabyte/kiss-xml
    1. clean documents from special characters
    1. dump parsed documents as separate files for directory

    RES: using `wikidump` crate for parsing wiki doc format. not optimal, as it loads entire simplewiki dump into memory, but ok for now

1. [in progress] install elasticsearch
1. build elastic index from dataset
1. build nano_search index from dataset
1. find queries
1. shoot queries to both indices and compare result

---

persist nano_search index to disk, and search with it
