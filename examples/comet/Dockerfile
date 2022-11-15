FROM alpine:latest

RUN apk update && apk add \
    socat \
    iproute2 \
    openblas-dev

ENV LD_LIBRARY_PATH=/usr/lib:$LD_LIBRARY_PATH

COPY run.sh /
RUN ["chmod", "+x", "/run.sh"]

COPY comet /

CMD ["/run.sh"]
