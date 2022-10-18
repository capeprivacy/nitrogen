# Nitrogen CLI

A command line interface to build and deploy web services to AWS Nitro Enclaves.

## Commands

- `nitrogen launch <ec2-instance-type> <port>`
- `nitrogen build <Dockerfile>`
- `nitrogen deploy <enclave-image-file> <ec2-hostname>`

## Features

- Spins up any EC2 instance type (with Nitro Enclaves enabled)
- Creates a security group for a specified port.
- Sets up SSH.
- Runs a socat proxy from public internet (TCP) into the nitro enclave (VSOCK).
- Builds any Dockerfile into an Enclave Image File (EIF).
- Deploys any EIF and launches a nitro enclave.

## Examples

`nitrogen launch m5n.16xlarge 443`
> Hostname: ec2-1-234-56-789.compute-1.amazonaws.com
> 
> Ports: 22, 443

`nitrogen build Dockerfile --name nginx`
> Filename: nginx.eif

`nitrogen deploy nginx.eif ec2-1-234-56-789.compute-1.amazonaws.com`
> Listening: ec2-1-234-56-789.compute-1.amazonaws.com:443

`curl https://ec2-1-234-56-789.compute-1.amazonaws.com/`
> Hello World
