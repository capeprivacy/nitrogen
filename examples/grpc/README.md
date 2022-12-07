Nitrogen gRPC example
==========

```
nitrogen setup grpc-stack ~/.ssh/id_rsa.pub
nitrogen build . -e grpc.eif
nitrogen deploy grpc-stack ~/.ssh/id_rsa -e grpc.eif

cargo run --bin helloworld-client ec2-55-87-191-108.compute-1.amazonaws.com -p 5000

# - OR - #

grpcurl -plaintext -import-path ./proto -proto helloworld.proto -d '{"name": "Tonic"}' 'ec2-55-87-191-108.compute-1.amazonaws.com:5000' helloworld.Greeter/SayHello

nitrogen delete grpc-stack
```
