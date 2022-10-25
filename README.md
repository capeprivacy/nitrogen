<div align="center">
  <img src="./media/nitrogen-logo.svg" alt="Nitrogen logo" width="400">
</div>

# Nitrogen CLI

Nitrogen is a tool for deploying web services to AWS Nitro Enclaves. Given a dockerfile and an ssh key, Nitrogen will spin up an EC2, configure the network, and build and deploy your web service. You get back a hostname that’s ready to go. Nitrogen is fully open source and it comes with pre-built scripts for deploying popular services like Nginx, Redis, and MongoDB.

## Commands

- `nitrogen setup <STACK_NAME> <KEY_NAME> --instance-type <EC2_INSTANCE_TYPE> -p <PORT> -s <SSH_LOCATION>`
- `nitrogen build <DOCKER_CONTEXT> <DOCKERFILE> --eif <EIF_LOCATION>`
- `nitrogen deploy <EC2_HOSTNAME> <EIF> <SSH_KEY> <CPU_COUNT> <MEMORY>`
- `nitrogen delete <EC2_HOSTNAME>`

## Features

- Spins up any enclave supported EC2 instance type (with Nitro Enclaves enabled)
- Creates a security group for a specified port.
- Sets up SSH.
- Runs a socat proxy from public internet (TCP) into the nitro enclave (VSOCK).
- Builds any Dockerfile into an Enclave Image File (EIF).
- Deploys any EIF and launches a nitro enclave.

## Examples

```sh
$ nitrogen setup nitrogen-test ec2-key --instance-type m5n.16xlarge

> Successfully setup enclave with stack ID "arn:aws:cloudformation:us-east-1::stack/nitrogen-test/500860b0-53d1-11ed-967c-0ebc7567a9a9"
>   Enclave user information:
>     InstanceId: i-0dd81f6b48396b020
>     PublicIP: 54.164.195.92
>     AZ: us-east-1c
>     PublicDNS: ec2-54-164-195-92.compute-1.amazonaws.com
```

```sh
$ nitrogen build Dockerfile --eif ./nginx.eif

> Filename: nginx.eif
```

```sh
$ nitrogen deploy ec2-1-234-56-789.compute-1.amazonaws.com nginx.eif ~/.ssh/id_rsa

> Listening: ec2-1-234-56-789.compute-1.amazonaws.com:443
```

```sh
$ curl https://ec2-1-234-56-789.compute-1.amazonaws.com/

> Hello World
```
