FROM vault:latest


RUN apk --no-cache add socat=1.7.4.1-r1

COPY config.hcl config.hcl

COPY run.sh ./
RUN ["chmod", "+x", "./run.sh"]

COPY app.sh ./
RUN ["chmod", "+x", "./app.sh"]

CMD ["/bin/sh", "/run.sh"]
