# Stage 1: Build the Rust binary
FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Minimal final image
FROM debian:bullseye-slim

WORKDIR /app
COPY --from=builder /app/target/release/packhub /app/packhub
COPY .env /app/.env
COPY packhub.asc /app/packhub.asc
COPY secret_key.asc /app/secret_key.asc

EXPOSE 80
ENTRYPOINT ["/app/packhub"]
