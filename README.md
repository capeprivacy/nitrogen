<div align="center">
  <img src="./media/nitrogen-logo.svg" alt="Nitrogen logo" width="400">
</div>

# Nitrogen CLI

[![Discord](https://img.shields.io/discord/1027271440061435975.svg?logo=discord)](https://discord.gg/S8WMGUg8ab)

Nitrogen is a tool for deploying web services to AWS Nitro Enclaves. Given a dockerfile and an ssh key, Nitrogen will spin up an EC2, configure the network, and build and deploy your web service. You get back a hostname that’s ready to go. Nitrogen is fully open source and it comes with pre-built scripts for deploying popular services like Nginx, Redis, and MongoDB.

## Install

Nitrogen can easily be installed with the following:

For Linux or Mac:

```
$ curl -fsSL https://raw.githubusercontent.com/capeprivacy/nitrogen/main/install.sh | sh
```

For Windows Powershell

```
$ iex (irm https://raw.githubusercontent.com/capeprivacy/nitrogen/main/install.ps1)
```

_Note: An AWS account is required. If you have AWS cli configured you can [retrieve your credentials](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html#cli-configure-files-where) with `cat ~/.aws/credentials`. See [troubleshooting](https://github.com/capeprivacy/nitrogen#troubleshooting) if your AWS account uses MFA_

```bash
export AWS_ACCESS_KEY_ID=<YOUR ACCESS KEY>
export AWS_SECRET_ACCESS_KEY=<YOUR SECRET>
```

## Commands

- `nitrogen setup <stack_name> <ssh_public_key>`
- `nitrogen build <dockerfile_directory>`
- `nitrogen deploy <stack_name> <ssh_private_key>`
- `nitrogen delete <stack_name>`

## Features

- Spins up any enclave supported EC2 instance type (with Nitro Enclaves enabled)
- Creates a security group for a specified port.
- Sets up SSH.
- Runs a socat proxy from public internet (TCP) into the nitro enclave (VSOCK).
- Builds any Dockerfile into an Enclave Image File (EIF).
- Deploys any EIF and launches a nitro enclave.

## Examples

### Nginx Example

```sh
$ nitrogen setup nitrogen-test ~/.ssh/id_rsa.pub --instance-type m5n.16xlarge
>  INFO nitrogen: Spinning up enclave instance 'nitrogen-test'.
>  INFO nitrogen::commands::setup: Successfully created enclave instance. stack_id="arn:aws:cloudformation:us-east-1:657861442343:stack/nitrogen-test/c93c7c80-5581-11ed-8a2b-0e2f3ffeccf1"
>  INFO nitrogen: User enclave information: name="nitrogen-test" instance_id="i-07daa284594ff02bc" public_ip="44.197.181.14" availability_zone="us-east-1b" public_dns="ec2-44-197-181-14.compute-1.amazonaws.com"
```

```sh
$ nitrogen build examples/nginx/
> Filename: nitrogen.eif
```

```sh
$ nitrogen deploy nitrogen-test ~/.ssh/id_rsa
> EIF is now running public_dns="ec2-1-234-56-789.compute-1.amazonaws.com:5000"
```

```sh
$ curl http://ec2-1-234-56-789.compute-1.amazonaws.com:5000/
> <!DOCTYPE html>
<html>
    <head>
        <title>Hello Nitrogen!</title>
    </head>
</html>
```

### Nginx TLS Examples

See [here](examples/nginx-tls/README.md).

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

## Contributors

Thank you to [@kalebpace](https://github.com/kalebpace) for contributing the name for the [nitrogen crate](https://crates.io/crates/nitrogen).
