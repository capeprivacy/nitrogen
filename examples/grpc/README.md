Nitrogen gRPC example
==========

Build binaries for EIF Dockerfile

```
docker build . -t hello-tonic
docker cp hello-tonic:/usr/local/cargo/bin/helloworld-server .
```