backend "file" {
    path = "/vault/file"
    default_lease_ttl = "168h"
    max_lease_ttl = "720h"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_cert_file = "vault.pem"
  tls_key_file  = "vault.key"
}

api_addr = "http://0.0.0.0:8200"
cluster_addr = "https://0.0.0.0:8201"
