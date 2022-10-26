FROM nginx/unit:1.28.0-minimal

RUN apt update && apt install -y \
        socat \
        iproute2

COPY config.json ./

COPY run.sh ./
RUN ["chmod", "+x", "./run.sh"]

COPY app.sh ./
RUN ["chmod", "+x", "./app.sh"]

CMD ["/run.sh"]
