# Build stage
FROM rust:1.86.0 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY data_service data_service/
COPY ws_client ws_client/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release -p data-service

FROM debian:bookworm-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/data-service .
CMD rm -rf /app/target

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["./data-service"]