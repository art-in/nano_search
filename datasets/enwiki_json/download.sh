#!/bin/sh

curl --output "wiki.json.bz2" -L https://www.dropbox.com/s/wwnfnu441w1ec9p/wiki-articles.json.bz2
bzip2 --decompress wiki.json.bz2

