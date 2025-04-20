# Get started with a build env with Rust nightly
FROM debian:bookworm as builder

# If you’re using stable, use this instead
# FROM rust:1.86-bullseye as builder

# Install required tools
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang pkg-config libssl-dev curl ca-certificates

# Install Rust and required cargo tools
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2025-02-19 \
  && rm -rf /var/lib/apt/lists/* \
  # Add cargo to the path for subsequent commands in this RUN layer
  && . "$HOME/.cargo/env"

# Add cargo to the path for subsequent RUN layers
ENV PATH="/root/.cargo/bin:${PATH}"

  # Install cargo-binstall
RUN cargo install cargo-binstall --locked
  # Install cargo-leptos using cargo-binstall
RUN   cargo-binstall cargo-leptos -y --version "0.2.28"
  # Install wasm-bindgen CLI tool
RUN  cargo install wasm-bindgen-cli --locked

ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV RUST_BACKTRACE=full

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

ENV RUSTUP_HOME=/app/.rustup

# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

# Build the app
RUN cargo leptos build --release -vv

FROM debian:bookworm-slim as runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/music_jam /app/

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 8080

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/leptos_start"]
