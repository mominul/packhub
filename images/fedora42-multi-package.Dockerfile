FROM fedora:42

WORKDIR /app

COPY . .

ENTRYPOINT ["./check_dnf_multiple.sh"]
