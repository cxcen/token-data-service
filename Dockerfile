# Build stage
FROM rust:1.86.0 as builder

WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Copy data_service project
COPY data_service data_service/

# Build the project
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release -p data_service

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/data_service .

# Set environment variables
ENV RUST_LOG=info
ENV PORT=8080

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["./data_service"] 