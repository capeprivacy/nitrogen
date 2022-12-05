# Standalone Runtime
An example running Cape runtime via Nitrogen

## Requirements
Access to capejail and kmstool images in Dockerhub.
```
docker login
```

Enable Docker kit
```
export DOCKER_BUILDKIT=1
```

Fetch the runtime submodule
```
git submodule update --init --recursive
```



## Running
Needs a local socat instance to run alongside the enclave executable. 

### TLS

To enable TLS you can generate self-signed certificates:

```
openssl ecparam -genkey -name secp384r1 -out server.key
openssl req -new -x509 -sha256 -key server.key -out server.crt -days 3650
```


### Using port 5000
The setup for the ec2 instance points to port 5000, so the socat instance
redirects port 5000 to the runtime websocket. 

### Using with Cape CLI
If you want to deploy/run with Cape CLI you will need to reset the key file locally. 

```
cape login    # follow promps for browser authentication

# We need to reset the key file under CLI and register against the local 
# runtime instance
rm ~/.config/cape/capekey.pub.der

cape key --url wss://<NITROGEN_EC2_INSTANCE>:5000 --insecure
cape deploy <CUSTOM_CAPE_FUNCTION> --url wss://<NITROGEN_EC2_INSTANCE>:5000

echo `<CUSTOM_FUNCTION_INPUT>`|  ./cape run -v <DEPLOYED_FUNCTION_ID>  -u wss://<NITROGEN_EC2_INSTANCE>:5000 --insecure -f -

```



