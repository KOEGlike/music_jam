FROM debian:bookworm AS builder

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang pkg-config libssl-dev curl ca-certificates

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly\
  && rm -rf /var/lib/apt/lists/* 

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-binstall --locked
RUN cargo-binstall cargo-leptos -y --version "0.2.28"

ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV RUST_BACKTRACE=full

RUN rustup target add wasm32-unknown-unknown

ENV RUSTUP_HOME=/app/.rustup

RUN mkdir -p /app
WORKDIR /app
COPY . .

RUN cargo leptos build --release -vv

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/music_jam /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 8080

CMD ["/app/music_jam"]
