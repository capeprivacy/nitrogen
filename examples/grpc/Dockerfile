# syntax=docker/dockerfile:1

FROM rust:alpine as builder
RUN apk update && apk add --no-cache alpine-sdk protoc protobuf-dev
COPY ./ /crate/
RUN cargo install --path /crate --bin helloworld-server

FROM alpine:latest
RUN apk update && apk add --no-cache socat iproute2
COPY --from=builder /usr/local/cargo/bin/helloworld-server /
COPY run.sh /
RUN ["chmod", "+x", "/run.sh"]
CMD ["/run.sh"]