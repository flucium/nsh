FROM ubuntu:22.04
RUN apt-get update && apt-get install curl -y && apt-get install git -y && apt-get install build-essential -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |  bash -s -- -y