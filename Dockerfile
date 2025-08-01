FROM --platform=$BUILDPLATFORM ghcr.io/linkerd/dev:v47-rust-musl AS build
ARG TARGETARCH
ENV CARGO="cargo auditable"
WORKDIR /app
COPY justfile Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,id=cargo-git-${TARGETARCH},target=/usr/local/cargo/git/db \
    --mount=type=cache,id=cargo-${TARGETARCH},target=/usr/local/cargo/registry/ \
    just profile=release fetch
RUN --mount=type=cache,id=cargo-git-${TARGETARCH},target=/usr/local/cargo/git/db \
    --mount=type=cache,id=cargo-${TARGETARCH},target=/usr/local/cargo/registry/ \
    just profile=release build && \
    mv target/$(just --evaluate _target)/release/hokay target/hokay

FROM scratch AS runtime
COPY --from=build /app/target/hokay /usr/local/bin/hokay
ENTRYPOINT ["/usr/local/bin/hokay"]
