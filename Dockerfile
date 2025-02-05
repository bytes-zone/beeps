FROM bitnami/minideb:bookworm

RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

COPY target/release/beeps-server /bin/beeps-server

ENTRYPOINT [ "/bin/beeps-server" ]

EXPOSE 3000
