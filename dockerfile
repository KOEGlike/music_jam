# Stage 1: Build stage
FROM alpine:edge AS builder

# Set architecture argument for multi-platform builds
ARG TARGETARCH

# Set working directory
WORKDIR /build

# Install dependencies for building
RUN apk update && \
    apk upgrade && \
    apk add pkgconfig libressl-dev mold musl-dev npm curl pigz brotli rustup gcc clang --no-cache

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy Rust toolchain file
COPY rust-toolchain.toml .

# Install Rust (minimal profile)
RUN rustup-init -y --profile minimal --default-toolchain none && \
    source $HOME/.cargo/env && \
    rustup show

# Download the correct cargo-leptos binary based on architecture
RUN set -ex && \
    if [ "$TARGETARCH" = "amd64" ]; then \
        ARCH_TAG="x86_64-unknown-linux-musl"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        ARCH_TAG="aarch64-unknown-linux-musl"; \
    else \
        echo "Unsupported architecture: $TARGETARCH"; exit 1; \
    fi && \
    curl -L "https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.26/cargo-leptos-${ARCH_TAG}.tar.gz" \
        | tar -xz && \
    mv ./cargo-leptos-${ARCH_TAG}/* /usr/local/bin/

# Copy the source code into the container
COPY . .

# Modify Cargo.toml for correct target triple
RUN sed -i '/\[package.metadata.leptos\]/,/^\[/ s/bin-target-triple="x86_64-unknown-linux-gnu"/bin-target-triple="x86_64-unknown-linux-musl"/' Cargo.toml

# Build the Leptos app
ENV LEPTOS_WASM_OPT_VERSION=version_121
RUN cargo-leptos --version
RUN cargo-leptos build -P --release -vv

# Stage 2: Final runtime image
FROM scratch AS final

# Copy the built binary from the builder stage
COPY --from=builder /build/target/${TARGETARCH}/release/your-binary /app

# Set entrypoint to the binary
ENTRYPOINT ["/app"]

# Optionally set a command if needed
# CMD ["--help"] 
