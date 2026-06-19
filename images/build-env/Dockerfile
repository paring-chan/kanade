FROM debian:trixie-slim

RUN apt-get update && \
    apt-get install -y git extrepo curl wget zip unzip tar zstd build-essential && \
    extrepo enable mise && \
    apt-get update && \
    apt-get install -y mise
