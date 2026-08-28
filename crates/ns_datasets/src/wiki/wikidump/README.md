Library for parsing wikipedia dumps

Based on code from library [wikidump](https://crates.io/crates/wikidump) v0.3.1

Main difference is streaming - this implementation allows to iterate over wiki pages without loading entire dump into memory, which allows to process big dumps, e.g. full english wikipedia dump with compressed file size ~20GB.

Useful article for understanding structure of wikipedia dump:  
https://dev.to/tobiasjc/understanding-the-wikipedia-dump-11f1

It uses [parse_wiki_text](https://crates.io/crates/parse_wiki_text) library for parsing [wikitext](https://en.wikipedia.org/wiki/Help:Wikitext).

It works well for parsing `simplewiki-20250601-pages-articles.xml.bz2` dump.

But unfortunately fails on `enwiki-20250601-pages-articles.xml.bz2` dump - `parse_wiki_text` library getting stuck on parsing text of some pages. Did not investigate why - most likely because the wikitext format evolves over time, but the library does not: its latest release was in 2019, and its GitHub repo link is now dead. There's no good Rust library for parsing wikitext right now.

List of wikitext parsers:  
https://www.mediawiki.org/wiki/Alternative_parsers

