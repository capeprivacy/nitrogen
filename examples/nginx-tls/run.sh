#!/bin/sh

ip addr add 127.0.0.1/32 dev lo
ip link set dev lo up

socat vsock-listen:5000,reuseaddr,fork tcp-connect:127.0.0.1:443 &

sh ./app.sh
