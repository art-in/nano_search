library for parsing wikipedia dumps

based on code from library [wikidump](https://crates.io/crates/wikidump) v0.3.1

main difference - streaming - this implementation allows to iterate over wiki pages without loading entire dump into memory, which allows to process big dumps, e.g. full english wikipedia dump with compressed file size ~20GB

it uses [parse_wiki_text](https://crates.io/crates/parse_wiki_text) library for parsing wiki format text

useful article for understanding structure of wikipedia dump:  
https://dev.to/tobiasjc/understanding-the-wikipedia-dump-11f1

TODO: move to separate crate
