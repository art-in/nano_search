library for parsing wikipedia dumps

based on code from library [wikidump](https://crates.io/crates/wikidump) v0.3.1

main difference - streaming - this implementation allows to iterate over wiki pages without loading entire dump into memory, which allows to process big dumps, e.g. full english wikipedia dump with compressed file size ~20GB

useful article for understanding structure of wikipedia dump:  
https://dev.to/tobiasjc/understanding-the-wikipedia-dump-11f1

it uses [parse_wiki_text](https://crates.io/crates/parse_wiki_text) library for parsing [wikitext](https://en.wikipedia.org/wiki/Help:Wikitext)

it works good for parsing `simplewiki-20250601-pages-articles.xml.bz2` dump

but unfortunately fails on `enwiki-20250601-pages-articles.xml.bz2` dump - `parse_wiki_text` library getting stuck on parsing text of some pages. didn't investigate why. most likely because wikitext format evolve over time, but the lib is not - latest release was in 2019, and github repo link now is dead. there's no good rust library for parsing wikitext right now

list of wikitext parsers:  
https://www.mediawiki.org/wiki/Alternative_parsers
