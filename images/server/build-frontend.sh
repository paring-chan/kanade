#!/usr/bin/env bash

rm -rf frontend
cd ../../apps/frontend
rm -rf dist
bun run build
cp -r dist ../../images/server/frontend
