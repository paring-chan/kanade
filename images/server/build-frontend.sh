#!/usr/bin/env bash

cd ../../apps/frontend
rm -rf dist
bun run build
cp -r dist ../../images/server/frontend
