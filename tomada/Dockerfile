FROM rust:1.89-bookworm AS build
WORKDIR /build
COPY . .
RUN --mount=type=cache,target=./target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    DOCKER_BUILD=1 cargo build -r --locked --package broker && mv target/release/broker /broker

FROM debian:bookworm
COPY --from=build /broker /usr/bin/broker
ENTRYPOINT [ "/usr/bin/broker" ]
