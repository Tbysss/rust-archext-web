FROM rust:1.77 as builder

ADD ./ /build
WORKDIR /build
RUN cargo build --release

FROM debian:stable-slim as runner

COPY --from=builder /build/target/release/log-preperation /bin/log-preperation
RUN useradd -u 1000 app

RUN mkdir -p /data && mkdir -p /archive
RUN chown -R app /archive && \
    chown -R app /data

USER app
VOLUME [ "/data" ]
VOLUME [ "/archive" ]

CMD [ "/bin/log-preperation", "/data", "/data", "/archive"]

