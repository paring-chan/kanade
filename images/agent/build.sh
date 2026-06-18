#!/usr/bin/env bash

set -ex

cargo zigbuild --release --target x86_64-unknown-linux-musl --bin kanade-agent
mkdir -p bin
cp ../../target/x86_64-unknown-linux-musl/release/kanade-agent bin/

docker build -t oci.pari.ng/kanade/agent .
