# Nginx TLS Example

## Requirements

- [mkcert](https://github.com/FiloSottile/mkcert)

## mkcert

mkcert allows us to easily install a local CA and generate TLS certificates for **development only**. Make sure not to use any certificates generated this way in production. These are only useful for testing TLS in your software during development.

## Enabling TLS

This is almost as simple as the nginx example from the [README.md](../../README.md#examples). We'll just need to run the mkcert commands in between
running the `nitrogen setup` command.

So first run `nitrogen setup`:

```sh
$ nitrogen setup nitrogen-nginx-tls ~/.ssh/id_rsa.pub --instance-type m5n.16xlarge
>  INFO nitrogen: Spinning up enclave instance 'nitrogen-nginx-tls'.
>  INFO nitrogen::commands::setup: Successfully created enclave instance. stack_id="arn:aws:cloudformation:us-east-1:657861442343:stack/nitrogen-nginx-tls/c93c7c80-5581-11ed-8a2b-0e2f3ffeccf1"
>  INFO nitrogen: User enclave information: name="nitrogen-nginx-tls" instance_id="i-07daa284594ff02bc" public_ip="44.197.181.14" availability_zone="us-east-1b" public_dns="ec2-44-197-181-14.compute-1.amazonaws.com"
```

Next up is running `mkcert`. It requires copying the `public_dns` field from above, like so:

```sh
$ mkcert -cert-file nitrogen.pem -key-file nitrogen.key ec2-44-197-181-14.compute-1.amazonaws.com
```

Copy these into the example so they can be read while building the docker image:

```
$ cp nitrogen.pem nitrogen.key examples/nginx-tls
```

Build the example:

```sh
$ nitrogen build examples/nginx-tls/
> Filename: nitrogen.eif
```

```sh
$ nitrogen deploy nitrogen-nginx-tls ~/.ssh/id_rsa
> Listening: ec2-1-234-56-789.compute-1.amazonaws.com:5000
```

```sh
$ curl https://ec2-1-234-56-789.compute-1.amazonaws.com:5000/
> <!DOCTYPE html>
> <html>
>    <head>
>        <title>Nitrogen Nginx TLS!!</title>
>    </head>
>    <body>
>        <p>This page was served with TLS :D</p>
>    </body>
> </html>
```

## Clean up

Make sure to run `nitrogen delete` to clean up the cloud formation stack when you're done:

```
nitrogen delete nitrogen-nginx-tls
```
