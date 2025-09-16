FROM rust:latest

WORKDIR /app

COPY . .

RUN apt update && apt install -y clang llvm pkg-config nettle-dev

RUN cargo build

EXPOSE 3000

COPY --chmod=0755 run_server.sh /sbin/run_server

ENTRYPOINT ["/sbin/run_server"]
