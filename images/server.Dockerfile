# Stage 1: Compute the recipe file
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache and Build the Rust binary
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Stage 3: Minimal final runtime image
FROM debian:bookworm-slim AS runtime

RUN apt update && apt install -y libssl-dev

WORKDIR /app
COPY --from=builder /app/target/release/packhub /app/packhub
COPY .env /app/.env
COPY packhub.asc /app/packhub.asc
COPY secret_key.asc /app/secret_key.asc

EXPOSE 80
ENTRYPOINT ["/app/packhub"]
