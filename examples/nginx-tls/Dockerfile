FROM nginx:stable-alpine

RUN apk --no-cache add socat=1.7.4.3-r0

COPY nginx.conf /etc/nginx/conf.d/nginx.conf
COPY nitrogen.key /etc/ssl/private/nitrogen.key
COPY nitrogen.pem /etc/ssl/certs/nitrogen.pem
COPY index.html /etc/nginx/html/index.html

COPY run.sh ./
RUN ["chmod", "+x", "./run.sh"]

COPY app.sh ./
RUN ["chmod", "+x", "./app.sh"]

CMD ["/bin/sh", "/run.sh"]
