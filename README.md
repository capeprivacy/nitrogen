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

>  INFO nitrogen: Spinning up enclave instance 'nitrogen-test'.
>  INFO nitrogen::commands::setup: Successfully created enclave instance. stack_id="arn:aws:cloudformation:us-east-1:657861442343:stack/nitrogen-test/c93c7c80-5581-11ed-8a2b-0e2f3ffeccf1"
>  INFO nitrogen: User enclave information: name="nitrogen-test" instance_id="i-07daa284594ff02bc" public_ip="44.197.181.14" availability_zone="us-east-1b" public_dns="ec2-44-197-181-14.compute-1.amazonaws.com"
```

```sh
$ nitrogen build examples/nginx/ examples/nginx/Dockerfile --eif ./nginx.eif

> Filename: nginx.eif
```

```sh
$ nitrogen deploy nitrogen-test nginx.eif ~/.ssh/id_rsa

> Listening: ec2-1-234-56-789.compute-1.amazonaws.com:443
```

```sh
$ curl https://ec2-1-234-56-789.compute-1.amazonaws.com/

> Hello World
```

## Troubleshooting

If you have permissions issues and your aws account has MFA enabled then attempt to use a session token before running `setup`.

```
aws sts get-session-token --serial-number arn:aws:iam::<AWS ACCOUNT NUMBER>:mfa/<USER NAME> --token-code <CODE>
```

Export the values printed from the above command:

```
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
export AWS_SESSION_TOKEN=
```

You can also use a helper script in this library called `sts.sh`. Warning: this will unset any AWS environment variables related to auth
that you have already set in your shell.

```
. sts.sh <ACCOUNT> <USER NAME> <CODE>
```
