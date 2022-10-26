#!/bin/sh

# Start redis and move it to the background
redis-server &
pid=$!
# Give the server some time to start
sleep 3
# Keep the enclave from terminating while the redis process is running
wait $pid
