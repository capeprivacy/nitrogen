#!/bin/sh

socat tcp-listen:5000,reuseaddr,fork vsock-connect:16:5000 &

sh ./app.sh
