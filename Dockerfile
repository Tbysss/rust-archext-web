FROM rust:1.71 as builder

ADD ./log-preperation /builder
WORKDIR /builder
RUN cargo build -r

