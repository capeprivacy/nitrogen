# Standalone Runtime
An example running Cape runtime via Nitrogen

## Requirements
Access to capejail and kmstool images in Dockerhub.

## Running
Needs a local socat instance to run alongside the enclave executable. 

### TLS

To enable TLS you can generate self-signed certificates:

```
openssl ecparam -genkey -name secp384r1 -out server.key
openssl req -new -x509 -sha256 -key server.key -out server.crt -days 3650
```


