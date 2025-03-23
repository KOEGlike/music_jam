# Get started with a build env with Rust nightly
FROM rustlang/rust:stable-bookworm AS builder

# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz

# Install cargo-leptos
RUN cargo install --locked cargo-leptos

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown


# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

ENV SQLX_OFFLINE=true
ENV LETPOS_WASM_OPT_VERSION=versoin_119
ENV RUST_BACKTRACE=full
# RUN cargo sqlx prepare

# Build the app
RUN cargo leptos build --release -vv


FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*


# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/music_jam /app/

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/


# Copy the migrations directory if it’s needed at runtime
COPY --from=builder /app/db/migrations /app/db/migrations

# Set any required env variables and
ENV RUST_LOG="info"
ENV RUST_BACKTRACE="full"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 8080

VOLUME pfp-images [ "/app/site/uploads" ]
# Run the server
CMD ["/app/music_jam"]
