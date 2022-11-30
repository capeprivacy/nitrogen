backend "file" {
    path = "/vault/file"
    default_lease_ttl = "168h"
    max_lease_ttl = "720h"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = "true"
}

api_addr = "http://0.0.0.0:8200"
cluster_addr = "https://0.0.0.0:8201"
