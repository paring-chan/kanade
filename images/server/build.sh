#!/usr/bin/env bash

set -ex

bash build-frontend.sh

FRONTEND_PATH=/frontend cargo zigbuild --release --target x86_64-unknown-linux-musl --bin kanade-server
mkdir -p bin
cp ../../target/x86_64-unknown-linux-musl/release/kanade-server bin/

docker build -t oci.pari.ng/kanade/server .
