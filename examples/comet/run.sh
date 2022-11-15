#!/bin/sh

ip addr add 127.0.0.1/32 dev lo
ip link set dev lo up

socat -ddd vsock-listen:5000,reuseaddr,fork tcp-connect:127.0.0.1:50001 &

echo "Running comet on port 50001, socat to 5000..."

RUST_LOG=debug /comet --identity ec2-54-91-199-105.compute-1.amazonaws.com:5000 --port 50001 &
comet_pid=$!
sleep 3
echo "comet running in background"
wait $comet_pid
