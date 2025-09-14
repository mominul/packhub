FROM opensuse/tumbleweed:latest

WORKDIR /app

COPY . .

ENTRYPOINT ["./check_zypper_multiple.sh"]
