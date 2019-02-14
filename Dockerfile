FROM rust:1.32.0-slim as builder

RUN apt-get update \
        && apt-get install -y --no-install-recommends git-core libssl-dev pkg-config \
        && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/
COPY . .

RUN cargo install --path .

FROM debian:stretch-slim

RUN apt-get update \
        && apt-get install -y --no-install-recommends libssl1.1 ca-certificates \
        && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/misc/entrypoint.sh /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/sendgrid-amqp-bridge /usr/local/bin/

CMD ["/usr/local/bin/entrypoint.sh"]
