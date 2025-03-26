FROM rust:latest

WORKDIR /app

COPY . .

RUN apt update && apt install -y clang llvm pkg-config nettle-dev

RUN cargo build

EXPOSE 3000

ENTRYPOINT ["scripts/run_server.sh"]
