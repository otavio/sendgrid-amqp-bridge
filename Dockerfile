FROM rust:1.32.0-slim as builder

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
	git-core libssl-dev pkg-config

WORKDIR /usr/src/
COPY . .

RUN cargo install --path .

FROM debian:stretch-slim

COPY --from=builder /usr/src/misc/entrypoint.sh /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/sendgrid-amqp-bridge /usr/local/bin/

CMD ["/usr/local/bin/entrypoint.sh"]
