FROM rust:1.71 as builder

ADD ./ /build
WORKDIR /build
RUN cargo build --release

FROM debian:stable-slim as runner

COPY --from=builder /build/target/release/log-preperation /bin/log-preperation
RUN useradd -u 1000 app
USER app
VOLUME [ "/data" ]
VOLUME [ "/archive" ]

CMD [ "/bin/log-preperation", "/data", "/data", "/archive"]

