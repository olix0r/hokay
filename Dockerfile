# Multi-arch dockerfile

FROM --platform=$BUILDPLATFORM ghcr.io/linkerd/dev:v46-rust-musl AS build
ARG TARGETARCH

WORKDIR /usr/src/hokay

# Copy manifests with dummy source and fetch dependencies
COPY justfile Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() { println!(\"cargo-build cache stub\"); }" > src/main.rs
RUN case "$TARGETARCH" in \
      amd64) T=x86_64-unknown-linux-musl ;; \
      arm64) T=aarch64-unknown-linux-musl ;; \
      arm) T=armv7-unknown-linux-musleabihf ;; \
      *) T="$TARGETARCH-unknown-linux-musl" ;; \
    esac && \
    just-cargo target="$T" profile=release fetch

# Copy full source & build actual binary
COPY src ./src
RUN rm -f target/*/release/deps/hokay
RUN case "$TARGETARCH" in \
      amd64) T=x86_64-unknown-linux-musl ;; \
      arm64) T=aarch64-unknown-linux-musl ;; \
      arm) T=armv7-unknown-linux-musleabihf ;; \
      *) T="$TARGETARCH-unknown-linux-musl" ;; \
    esac && \
    just-cargo target="$T" profile=release build

FROM scratch
COPY --from=build /usr/src/hokay/target/*/release/hokay /usr/local/bin/hokay
