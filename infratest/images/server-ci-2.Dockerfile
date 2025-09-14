FROM rust:latest

WORKDIR /app

COPY . .

RUN apt update && apt install -y clang llvm pkg-config nettle-dev

RUN cargo build

EXPOSE 3000

COPY --chmod=0755 run_server.sh /sbin/run_server

# RUN echo "Hello, World!"
# RUN chmod +x
# RUN echo "After chmod"

# CMD ["ls", "-l"]
# CMD ["cat", "run_server.sh"]
# ENTRYPOINT ["run_server.sh"]

ENTRYPOINT ["/sbin/run_server"]
