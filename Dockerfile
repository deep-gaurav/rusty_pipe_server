FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive
RUN apt update
RUN apt install curl git build-essential -y

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output rustinstaller
RUN sh rustinstaller -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release

CMD cd /src && ./target/release/rusty_pipe_server