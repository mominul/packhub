# Stage 1: Compute the recipe file
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache and Build the Rust binary
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN apt update && apt install -y clang llvm pkg-config nettle-dev

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Stage 3: Minimal final runtime image
FROM debian:bookworm-slim AS runtime

RUN apt update && apt install -y libssl-dev ca-certificates clang llvm pkg-config nettle-dev
RUN update-ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/packhub /app/packhub
COPY /pages /app/pages

EXPOSE 80
ENTRYPOINT ["/app/packhub"]
