FROM ubuntu:24.04

WORKDIR /app

COPY . .

ENV DIST=ubuntu

ENTRYPOINT ["./check_apt_multiple.sh"]
