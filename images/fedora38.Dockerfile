FROM fedora:39

WORKDIR /app

COPY . .

ENTRYPOINT ["./check_dnf.sh"]
