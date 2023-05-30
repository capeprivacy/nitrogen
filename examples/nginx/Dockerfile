FROM nginx:stable-alpine

RUN apk --no-cache add socat=1.7.4.4-r0

COPY index.html /usr/share/nginx/html/index.html

COPY run.sh ./
RUN ["chmod", "+x", "./run.sh"]

COPY app.sh ./
RUN ["chmod", "+x", "./app.sh"]

CMD ["/bin/sh", "/run.sh"]
