# Nitrogen CLI

`nitrogen launch <ec2-instance-type> <port>`
`nitrogen build <Dockerfile>`
`nitrogen deploy <enclave-image-file> <ec2-hostname>`

### Examples

`nitrogen launch m5n.16xlarge 443`
> Hostname: ec2-1-234-56-789.compute-1.amazonaws.com
> Ports: 22, 443

`nitrogen build Dockerfile --name nginx`
> Filename: nginx.eif

`nitrogen deploy nginx.eif ec2-1-234-56-789.compute-1.amazonaws.com`
> Listening: ec2-1-234-56-789.compute-1.amazonaws.com:443

`curl https://ec2-1-234-56-789.compute-1.amazonaws.com/`
> Hello World
