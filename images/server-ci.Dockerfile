FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build

EXPOSE 3000

ENTRYPOINT ["target/debug/packhub"]
