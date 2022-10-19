#!/bin/sh

echo "********* start unit"
unitd --no-daemon --control 127.0.0.1:8080 --control unix:/var/run/control.unit.sock &
pid=$!

echo "********* wait 3 seconds to give Unit some time to start"
sleep 3

curl -X PUT --data-binary @bundle.pem --unix-socket /var/run/control.unit.sock http://localhost/certificates/bundle

curl -X PUT --data-binary @config.json --unix-socket /var/run/control.unit.sock http://localhost/config/

mkdir /www
mkdir /www/static
echo "Hello World!" > /www/static/hello

wait $pid
