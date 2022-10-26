FROM redis

RUN apt update && apt install -y \
        socat \
        iproute2

ADD run.sh /
RUN ["chmod", "+x", "/run.sh"]

COPY app.sh /
RUN ["chmod", "+x", "/app.sh"]

CMD ["/run.sh"]
