FROM nginx/unit:1.26.1-python3.9

RUN apt update && apt install -y \
        socat \
        iproute2

COPY config.json ./

COPY run.sh ./
RUN ["chmod", "+x", "./run.sh"]

COPY app.sh ./
RUN ["chmod", "+x", "./app.sh"]

CMD ["/run.sh"]
