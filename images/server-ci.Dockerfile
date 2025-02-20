FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build

EXPOSE 80

ENTRYPOINT ["scripts/run_server.sh"]
