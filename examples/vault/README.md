# Vault Example

## Requirements

- vault CLI (see [here](https://developer.hashicorp.com/vault/docs/install) for installation instructions)

## Deploy Vault

```
$ nitrogen setup vault-nitro ~/.ssh/id_rsa.pub
```

```
$ nitrogen build examples/vault
```

```
$ nitrogen deploy vault-nitro ~/.ssh/id_rsa
```

After the last command you should see something like:

```
...
2022-11-24T19:23:19.595376Z  INFO nitrogen::commands::deploy: EIF is now running public_dns="ec2-54-159-199-57.compute-1.amazonaws.com"
2022-11-24T19:23:19.595512Z  INFO nitrogen::commands::deploy: Check enclave status...
2022-11-24T19:23:20.159233Z  INFO nitrogen::commands::deploy: Enclave up and running!
```

## Setup Vault

Here we've deployed Vault in production mode so it takes a few more steps to fully set up Vault.

First, set the address of the ec2 instance:

```
$ export VAULT_ADDR="ec2-54-159-199-57.compute-1.amazonaws.com"
```

Next initialize vault:

```
$ vault operator init
```

Should see some output like:

```
Unseal Key 1: J6jRd1lZ7eU+wmBmtE4OsmymVtWfPmpGnh7z/o4e68JG
Unseal Key 2: cLZnWKAt2KxSpsMxNAOZ05Pcv1XmxFugcrzh3fNp+GCw
Unseal Key 3: quW9NrKUdrLcpYCnHRqUORXH+VjX9O8QwHafQETtVxcU
Unseal Key 4: ZI9pTMq7KEy7e2zC+AJx5iSANTBQA4ts0UxJeh7EyEqS
Unseal Key 5: F5lI1mjyR7h24lDtY95uBLLRs+mupcTvJBRl+aDtSd4a

Initial Root Token: hvs.FDnNk0XLP7jv2g18UnMkN3mi

Vault initialized with 5 key shares and a key threshold of 3. Please securely
distribute the key shares printed above. When the Vault is re-sealed,
restarted, or stopped, you must supply at least 3 of these keys to unseal it
before it can start servicing requests.

Vault does not store the generated root key. Without at least 3 keys to
reconstruct the root key, Vault will remain permanently sealed!

It is possible to generate new unseal keys, provided you have a quorum of
existing unseal keys shares. See "vault operator rekey" for more information.
```

Then for 3 of the key shares call the following:

```
$ vault operator unseal <KEY SHARE>
```

Finally log in with the root token:

```
$ vault login <ROOT TOKEN>
```

## Store Keys

To store keys in production mode we must first set up a secrets engine:

```
$ vault secrets enable kv
```

And then we can store and retrieve secrets:

```
$ vault kv put -mount=kv sec foo=bar
```

```
$ vault kv get -mount=kev sec
```

## TLS

**Note: reading the [nginx tls example](../nginx-tls/README.md) could be helpful for some more background**

With some additional setup we can enable TLS.

First, generate the certificate and key with `mkcert`:

```
$ mkcert -cert-file vault.pem -key-file vault.key <INSTANCE HOST NAME>
$ cp vault.pem vault.key examples/vault
```

```
$ nitrogen build examples/vault -d Dockerfile.tls
```

```
$ nitrogen deploy vault-nitro ~/.ssh/id_rsa
```

```
$ export VAULT_ADDR="https://ec2-54-159-199-57.compute-1.amazonaws.com"
```

Then you can follow the rest of the sets from above but this is using TLS!
