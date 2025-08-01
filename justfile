arch := env_var_or_default("TARGETARCH", "amd64")

_target := if arch == "amd64" {
    "x86_64-unknown-linux-musl"
} else if arch == "arm64" {
    "aarch64-unknown-linux-musl"
} else if arch == "arm" {
    "armv7-unknown-linux-musleabihf"
} else {
    error("Unsupported TARGETARCH: " + arch)
}

profile := 'release'

docker *args:
    docker buildx build . \
        --platform linux/amd64,linux/arm64,linux/arm/v7 \
        {{ args }}

fetch:
    @just-cargo target={{_target}} profile={{profile}} fetch

clippy:
    @just-cargo target={{_target}} profile={{profile}} clippy

build:
    @just-cargo target={{_target}} profile={{profile}} build
