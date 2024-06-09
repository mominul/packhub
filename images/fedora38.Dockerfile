FROM fedora:38

WORKDIR /app

COPY . .

ENTRYPOINT ["./check_dnf.sh"]
