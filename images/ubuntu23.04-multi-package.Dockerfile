FROM ubuntu:23.04

WORKDIR /app

COPY . .

ENTRYPOINT ["./check_apt_multiple.sh"]
