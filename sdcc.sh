#!/usr/bin/env bash

set -ex

rm -rf examples/sdcc
mkdir -p examples/sdcc

pushd examples/sdcc > /dev/null
sdcc \
    -mmcs51 \
    --no-xinit-opt \
    ../sdcc.c
makebin -p sdcc.ihx sdcc.bin
d52 -t sdcc.bin
expand sdcc.d52 > sdcc.d52.expanded
mv sdcc.d52.expanded sdcc.d52
popd > /dev/null

cp examples/sdcc/sdcc.bin examples/print.rom
cargo run --release --example print
