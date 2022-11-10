Nitrogen gRPC example
==========

```
nitrogen setup grpc ~/.ssh/id_rsa.pub
nitrogen build . -e grpc.eif
nitrogen deploy grpc ~/.ssh/id_rsa -e grpc.eif

grpcurl -plaintext -import-path ./proto -proto helloworld.proto -d '{"name": "Tonic"}' 'ec2-55-87-191-108.compute-1.amazonaws.com:5000' helloworld.Greeter/SayHello
```
