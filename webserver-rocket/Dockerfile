FROM rust:1.71 as builder

ADD ./src /app/src
ADD ./Cargo.toml /app/Cargo.toml
WORKDIR /app
RUN cargo build --release

FROM debian:stable-slim as runner

COPY --from=builder /app/target/release/webserver-rocket /app/webserver
ADD ./static /app/static
ADD ./templates /app/templates
ADD ./Rocket.toml /app/Rocket.toml
RUN useradd -u 1000 app
RUN mkdir -p /data
RUN chown -R app /app && \
    chown -R app /data
USER app

VOLUME [ "/data" ]

EXPOSE 8000

WORKDIR /app
CMD [ "./webserver"]