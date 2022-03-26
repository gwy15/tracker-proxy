FROM rust:slim-buster as builder
WORKDIR /code

COPY . .
RUN cargo b --release --no-default-features --features rustls \
    && strip target/release/tracker-proxy

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/tracker-proxy .
ENTRYPOINT [ "./tracker-proxy" ]
CMD []
