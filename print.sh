#!/usr/bin/env bash

set -ex

as31 -Fbin -Oexamples/print.rom examples/print.a51
cargo run --release --example print
