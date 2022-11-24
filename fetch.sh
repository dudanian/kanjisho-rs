#!/bin/bash

rm -rf data

wget -P data http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz
wget -P data http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz
wget -P data http://ftp.edrdg.org/pub/Nihongo/kradzip.zip

unzip -d data data/*.zip
gunzip -k data/*.gz

mv data/JMdict_e data/JMdict_e.xml