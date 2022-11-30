#!/bin/sh

socat tcp-listen:2345,reuseaddr,fork vsock-connect:16:2345 &

sh ./app.sh
