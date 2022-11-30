FROM golang:1.18-alpine as builder

ARG SERVICE_VERSION
ENV SERVICE_VERSION=${SERVICE_VERSION:-unknown}
ENV PATH="/usr/local/bin:${PATH}"

RUN apk add git openssh

WORKDIR /app

RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh

RUN GH_IP=$(getent hosts github.com | cut -d ' ' -f 1) && \
    ssh-keyscan -H github.com >> /root/.ssh/known_hosts && \
    ssh-keyscan -H $GH_IP >> /root/.ssh/known_hosts && \
    ssh-keyscan -H github.com,$GH_IP >> /root/.ssh/known_hosts && \
    chmod 600 /root/.ssh/known_hosts

RUN git config \
    --global \
    url."git@github.com:".insteadOf \
    "https://github.com"

COPY runtime/go.mod .
COPY runtime/go.sum .

RUN go mod download

COPY ./runtime .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -buildvcs=false -ldflags "-X main.Version=$SERVICE_VERSION" -o bin/runtime ./cmd/runtime

FROM python:3.9-slim-bullseye

WORKDIR /runtime

# Add some pre-bundled python libraries.
RUN pip install python-dotenv
RUN pip install pyjwt[crypto]

# create chroot environment for jail
RUN adduser jailuser
RUN mkdir /chroot && \
    cp -r /bin /chroot/ && \
    cp -r /sbin /chroot/ && \
    cp -r /usr /chroot/ && \
    cp -r /etc /chroot/ && \
    cp -r /lib /chroot/ && \
    cp -r /lib64 /chroot/ && \
    mkdir /chroot/dev && \
    mknod -m 666 /chroot/dev/null c 1 3 && \
    mknod -m 666 /chroot/dev/zero c 1 5 && \
    mknod -m 666 /chroot/dev/random c 1 8 && \
    mknod -m 666 /chroot/dev/urandom c 1 9 && \
    mkdir -p /chroot/home/jailuser/ && \
    chown -R jailuser:jailuser /chroot/home/jailuser && \
    chmod -R 775 /chroot/home/jailuser/

# sudo is needed because capejail must run as root in order to `chroot` and
# `setuid` to jailuser. The chroot and separate user add additional isolation
# to protect runtime (and secrets that runtime might have in memory) from the
# user process.
RUN apt update && \
    apt install -y \
        sudo \
        wget \
        socat \
    && apt clean

RUN echo "socat --version"
# RUN wget http://www.dest-unreach.org/socat/download/socat-1.7.4.3.tar.gz
# RUN tar -xzf socat-1.7.4.3.tar.gz
# RUN cd socat-1.7.4.3
# RUN ./configure && make && sudo make install
# RUN sudo mv socat /usr/local/bin/socat

RUN useradd runtime

# Allow runtime to read files from jailuser (such as the results file)
RUN usermod -aG jailuser runtime

# Allow runtime to `sudo capejail`
RUN usermod -aG sudo runtime
RUN echo "%sudo    ALL=(ALL:ALL) NOPASSWD: /bin/capejail.sh" >> /etc/sudoers

COPY --from=builder /app/bin/runtime ./bin/runtime

COPY ./runtime/launch.py /runtime/.

RUN chown -R runtime:runtime /runtime

USER runtime

# This would have to be updated for running standalone, or use self-generated certs
COPY --chown=runtime:runtime server.key server.crt /runtime/
ENV CAPE_CERTFILE=/runtime/server.crt CAPE_KEYFILE=/runtime/server.key

COPY --from=capeprivacy/capejail:release-0c0b492 /bin/capejail /bin/

COPY ./runtime/capejail.sh /bin/capejail.sh

ENV CAPE_STANDALONE_STORAGE=true
ENV CAPE_STANDALONE_KMS=true
ENV CAPE_STANDALONE_LIFECYCLE=true
ENV CAPE_STANDALONE_EPHEMERAL=false


COPY run.sh /run.sh

COPY app.sh /app.sh

CMD ["/bin/sh", "/run.sh"]
