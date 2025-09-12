FROM --platform=$BUILDPLATFORM rust:1.89-bookworm AS build
ARG TARGETPLATFORM
ARG BUILDPLATFORM
WORKDIR /build
COPY . .

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then apt update && apt install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross; fi;

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

RUN --mount=type=cache,target=./target/,id=${TARGETPLATFORM} \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    set -eux; \
    if [ "$TARGETPLATFORM" = "linux/arm64" ]; then TARGET_TRIPLE="aarch64-unknown-linux-gnu"; else TARGET_TRIPLE="x86_64-unknown-linux-gnu"; fi; \
    rustup target add $TARGET_TRIPLE; \
    cargo build -r --locked --target $TARGET_TRIPLE --package broker && mv "target/${TARGET_TRIPLE}/release/broker" /broker;

FROM debian:bookworm
COPY --from=build /broker /usr/bin/broker
ENTRYPOINT [ "/usr/bin/broker" ]
