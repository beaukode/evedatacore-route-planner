FROM rust:1.81 AS build
COPY . /app
WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin server && \
    cp target/release/server /app/server

FROM debian:stable-slim
EXPOSE 8000
HEALTHCHECK --interval=1m --timeout=3s --start-interval=1s --start-period=30s \
    CMD curl --fail http://127.0.0.1:8000/ || exit 1
RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/server /app/server

WORKDIR /app
ENV RUST_LOG=info
ENV ROCKET_CLI_COLORS=0
ENV ROCKET_LOG_LEVEL=normal
ENV ROCKET_ADDRESS=0.0.0.0
ENV STARMAP_PATH=/data/starmap.bin
CMD ["/app/server"]
