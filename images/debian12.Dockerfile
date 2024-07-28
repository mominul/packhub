FROM debian:12

WORKDIR /app

COPY . .

ENV DIST=debian

ENTRYPOINT ["./check_apt.sh"]
