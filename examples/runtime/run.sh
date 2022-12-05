#!/bin/sh

socat tcp-listen:5000,reuseaddr,fork vsock-connect:16:2345 &

sh ./app.sh
