pub const LAUNCH_TEMPLATE: &'static str = r##"{
  "AWSTemplateFormatVersion" : "2010-09-09",

  "Description" : "AWS CloudFormation Sample Template EC2InstanceWithSecurityGroupSample: Create an Amazon EC2 instance running the Amazon Linux AMI. The AMI is chosen based on the region in which the stack is run. This example creates an EC2 security group for the instance to give you SSH access. **WARNING** This template creates an Amazon EC2 instance. You will be billed for the AWS resources used if you create a stack from this template.",

  "Parameters" : {
    "KeyName": {
      "Description" : "Name of an existing EC2 KeyPair to enable SSH access to the instance",
      "Type": "AWS::EC2::KeyPair::KeyName",
      "ConstraintDescription" : "must be the name of an existing EC2 KeyPair."
    },

    "InstanceName": {
        "Description": "Name of the ec2 instance",
        "Type": "String"
    },

    "InstanceType" : {
      "Description" : "Type of the ec2 instance",
      "Type" : "String",
      "AllowedValues" : [ "m5a.xlarge"],
      "ConstraintDescription" : "must be a valid EC2 instance type that supports Nitro Enclaves, m5a.xlarge and larger."
    },

    "SSHLocation" : {
      "Description" : "The IP address range that can be used to SSH to the EC2 instances",
      "Type": "String",
      "MinLength": "9",
      "MaxLength": "18",
      "AllowedPattern": "(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})/(\\d{1,2})",
      "ConstraintDescription": "must be a valid IP CIDR range of the form x.x.x.x/x."
   }
  },

  "Mappings" : {
    "AWSInstanceType2Arch" : {
      "m5a.xlarge"    : { "Arch" : "HVM64"  },
      "m5a.2xlarge"   : { "Arch" : "HVM64"  },
      "m5a.4xlarge"   : { "Arch" : "HVM64"  },
      "m5a.8xlarge"   : { "Arch" : "HVM64"  },
      "m5a.12xlarge"  : { "Arch" : "HVM64"  }
    },

    "AWSInstanceType2NATArch" : {
      "m5a.xlarge"    : { "Arch" : "NATHVM64"  },
      "m5a.2xlarge"   : { "Arch" : "NATHVM64"  },
      "m5a.4xlarge"   : { "Arch" : "NATHVM64"  },
      "m5a.8xlarge"   : { "Arch" : "NATHVM64"  },
      "m5a.12xlarge"  : { "Arch" : "NATHVM64"  }
    }
,
    "AWSRegionArch2AMI" : {
      "af-south-1"       : {"HVM64" : ""},
      "ap-east-1"        : {"HVM64" : ""},
      "ap-northeast-1"   : {"HVM64" : "ami-078296f82eb463377"},
      "ap-northeast-2"   : {"HVM64" : "ami-0c76973fbe0ee100c"},
      "ap-northeast-3"   : {"HVM64" : "ami-0953215c6a0ce5164"},
      "ap-south-1"       : {"HVM64" : "ami-01216e7612243e0ef"},
      "ap-southeast-1"   : {"HVM64" : "ami-0f62d9254ca98e1aa"},
      "ap-southeast-2"   : {"HVM64" : "ami-067e6178c7a211324"},
      "ca-central-1"     : {"HVM64" : "ami-046a5648dee483245"},
      "cn-north-1"       : {"HVM64" : ""},
      "cn-northwest-1"   : {"HVM64" : ""},
      "eu-central-1"     : {"HVM64" : "ami-05ff5eaef6149df49"},
      "eu-north-1"       : {"HVM64" : "ami-0bcf2639b551f6b31"},
      "eu-south-1"       : {"HVM64" : ""},
      "eu-west-1"        : {"HVM64" : "ami-0ea0f26a6d50850c5"},
      "eu-west-2"        : {"HVM64" : "ami-06672d07f62285d1d"},
      "eu-west-3"        : {"HVM64" : "ami-0ddab716196087271"},
      "me-south-1"       : {"HVM64" : ""},
      "sa-east-1"        : {"HVM64" : "ami-0895310529c333a0c"},
      "us-east-1"        : {"HVM64" : "ami-026b57f3c383c2eec"},
      "us-east-2"        : {"HVM64" : "ami-0f924dc71d44d23e2"},
      "us-west-1"        : {"HVM64" : "ami-09208e69ff3feb1db"},
      "us-west-2"        : {"HVM64" : "ami-08e2d37b6a0129927"}
    }
  },

  "Resources" : {
    "EC2Instance" : {
      "Type" : "AWS::EC2::Instance",
      "Metadata": {
        "AWS::CloudFormation::Init": {
            "configSets": {
                "Install": [
                    "Install"
                ]
            },
            "Install": {
                "packages": {
                    "yum": {
                      "docker": [],
                      "aws-nitro-enclaves-cli": []
                    }
                },
                "services": {
                    "sysvinit": {
                        "docker": {
                            "enabled": "true",
                            "ensureRunning": "true"
                        }
                    }
                },
                "commands": {
                    "docker_for_ec2_user": {
                        "command": "usermod -aG docker ec2-user"
                    },
                    "nitro_cli_for_ec2_user": {
                        "command": "usermod -aG ne ec2-user"
                    }
                }
            }
        }
      },
      "Properties" : {
        "InstanceType" : { "Ref" : "InstanceType" },
        "SecurityGroups" : [ { "Ref" : "InstanceSecurityGroup" } ],
        "KeyName" : { "Ref" : "KeyName" },
        "ImageId" : { "Fn::FindInMap" : [ "AWSRegionArch2AMI", { "Ref" : "AWS::Region" },
                          { "Fn::FindInMap" : [ "AWSInstanceType2Arch", { "Ref" : "InstanceType" }, "Arch" ] } ] },
        "EnclaveOptions": {
            "Enabled": true
        },
        "Tags" : [
            {"Key" : "Name", "Value" : { "Ref": "InstanceName"}}
        ],
        "UserData": {
          "Fn::Base64":{
            "Fn::Join":[
              "",
              [
                "#!/bin/bash -xe\n",
                "yum install -y aws-cfn-bootstrap\n",
                "amazon-linux-extras enable aws-nitro-enclaves-cli\n",
                "# Install the files and packages from the metadata\n",
                "/opt/aws/bin/cfn-init -v ",
                "         --stack ",
                { "Ref":"AWS::StackName" },
                "         --resource EC2Instance ",
                "         --configsets Install ",
                "         --region ",
                { "Ref":"AWS::Region" },
                "\n",
                "systemctl start nitro-enclaves-allocator.service && systemctl enable nitro-enclaves-allocator.service\n",
                "systemctl start docker && systemctl enable docker\n",
                "docker pull alpine/socat:latest\n",
                "docker run -d --name socat alpine/socat tcp-listen:80,fork,keepalive,reuseaddr vsock-connect:16:5000,keepalive\n"
              ]
            ]
          }
        }
      }
    },

    "InstanceSecurityGroup" : {
      "Type" : "AWS::EC2::SecurityGroup",
      "Properties" : {
        "GroupDescription" : "Enable SSH access via port 22",
        "SecurityGroupIngress" : [ 
          {
            "IpProtocol" : "tcp",
            "FromPort" : "22",
            "ToPort" : "22",
            "CidrIp" : { "Ref" : "SSHLocation"}
          },
          {
            "IpProtocol" : "tcp", 
            "FromPort" : 80, 
            "ToPort" : 80, 
            "CidrIp" : "0.0.0.0/0"
          } 
        ]
      }
    }
  },

  "Outputs" : {
    "InstanceId" : {
      "Description" : "InstanceId of the newly created EC2 instance",
      "Value" : { "Ref" : "EC2Instance" }
    },
    "AZ" : {
      "Description" : "Availability Zone of the newly created EC2 instance",
      "Value" : { "Fn::GetAtt" : [ "EC2Instance", "AvailabilityZone" ] }
    },
    "PublicDNS" : {
      "Description" : "Public DNSName of the newly created EC2 instance",
      "Value" : { "Fn::GetAtt" : [ "EC2Instance", "PublicDnsName" ] }
    },
    "PublicIP" : {
      "Description" : "Public IP address of the newly created EC2 instance",
      "Value" : { "Fn::GetAtt" : [ "EC2Instance", "PublicIp" ] }
    }
  }
}"##;