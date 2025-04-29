ideas to what implement next:

---

compare with other search engines, e.g. elastic or quickwit/tantivy

1. [done] find good dataset with text articles
    RES:
    - cisi
    - simplewiki
1. [done] parse and clean dataset
    1. find xml parser for rust
        - https://github.com/RazrFalcon/roxmltree
        - https://github.com/DrPlantabyte/kiss-xml
    1. clean documents from special characters
    1. dump parsed documents as separate files for directory

    RES: using `wikidump` crate for parsing wiki doc format. not optimal, since it loads entire simplewiki dump into memory, but ok for now

1. [done] build tantivy index
1. [done] build nano index
1. [done] run search query and compare result

---

persist nano_search index to disk, and search with it
