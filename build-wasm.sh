#!/bin/sh

erdpy --verbose contract build .

# twiggy top -n 1000  output/attestation-size.wasm > twiggy.txt
# twiggy paths  output/attestation-size.wasm > tpaths.txt
# twiggy monos  output/attestation-size.wasm > tmonos.txt
