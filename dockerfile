FROM alpine:edge AS builder
WORKDIR /build

RUN apk update && \
    apk upgrade && \
    apk add pkgconfig libressl-dev mold musl-dev npm curl pigz brotli rustup gcc clang --no-cache

ENV PATH="/root/.cargo/bin:${PATH}"

COPY rust-toolchain.toml .

RUN rustup-init -y --profile minimal --default-toolchain none && \
    source $HOME/.cargo/env && \
    rustup show


RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.26/cargo-leptos-x86_64-unknown-linux-musl.tar.gz | tar -xz && \
    mv ./cargo-leptos-x86_64-unknown-linux-musl/* ./

COPY . .


RUN sed -i '/\[package.metadata.leptos\]/,/^\[/ s/bin-target-triple="x86_64-unknown-linux-gnu"/bin-target-triple="x86_64-unknown-linux-musl"/' Cargo.toml

ENV LEPTOS_WASM_OPT_VERSION=version_121
RUN ./cargo-leptos build -P --release -vv