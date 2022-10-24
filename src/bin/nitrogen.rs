use aws_sdk_cloudformation::model::{Stack, StackStatus};
use failure::Error;

use aws_sdk_cloudformation::{model::Parameter, output::CreateStackOutput, Client};
use clap::{Parser, Subcommand};
use home;
use std::env;
use tokio::process::Command;

include!("../template.rs");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Provision nitro-enabled ec2 instances
    Launch {
        /// Name of the CloudFormation stack/provisioned EC2 instance
        #[arg(short, long)]
        name: String,
        /// EC2-instance type. Must be Nitro compatible.
        #[arg(long)]
        instance_type: String,
        /// EC2-instance port for socat enclave connection
        #[arg(short, long, default_value_t = 5000)]
        port: usize,
        /// EC2 key-pair to use for the provisioned instance
        #[arg(short, long)]
        key_name: String,
        /// IP address range that can be used to SSH to the EC2 instance. Defaults to anywhere ("0.0.0.0/0").
        #[arg(short, long)]
        ssh_location: Option<String>,
    },

    /// Build a Nitro EIF from a given Dockerfile
    Build {
        // Dockerfile location
        #[arg(short, long)]
        dockerfile: String,
        // docker context directory
        #[arg(short, long)]
        context: String,
        // Output EIF location
        #[arg(short, long)]
        eif: String,
    },

    /// Deploy an EIF to a provisioned nitro ec2 instance
    Deploy {
        /// Domain of the provisioned ec2 instance
        instance: String,
        /// Filepath to EIF
        eif: String,
        /// Filepath to SSH key for the instance
        ssh_key: String,
        /// Number of CPUs to provision for the enclave
        cpu_count: String,
        /// Memory in MB to provision for the enclave
        memory: String,
    },

    /// Delete launched ec2 instance
    Delete {
        /// Name of the provisioned instance
        instance: String,
    },
}

fn lift_to_param(key: impl Into<String>, value: impl Into<String>) -> Parameter {
    Parameter::builder()
        .parameter_key(key)
        .parameter_value(value)
        .build()
}

async fn launch_stack(
    client: &Client,
    launch_template: &String,
    name: &String,
    instance_type: &String,
    port: &usize,
    key_name: &String,
    ssh_location: &String,
) -> Result<CreateStackOutput, Error> {
    // TODO tokio tracing, consider instrument
    println!("Launching instance...");
    println!("Instance Name: {}", name);
    println!("Instance type: {}", instance_type);
    println!("Socat Port: {}", port);
    println!("Key Name: {}", key_name);

    let stack = client
        .create_stack()
        .stack_name(name)
        .template_body(launch_template)
        .parameters(lift_to_param("InstanceName", name))
        .parameters(lift_to_param("InstanceType", instance_type))
        .parameters(lift_to_param("Port", port.to_string()))
        .parameters(lift_to_param("KeyName", key_name))
        .parameters(lift_to_param("SSHLocation", ssh_location));
    let stack_output = stack.send().await?;
    Ok(stack_output)
}

async fn get_stack(client: &Client, stack_id: &str) -> Result<Stack, Error> {
    let resp = client.describe_stacks().stack_name(stack_id).send().await?;
    let this_stack = resp.stacks().unwrap_or_default().first().unwrap();
    Ok(this_stack.clone())
}

async fn check_stack_status(
    client: &Client,
    stack_id: &str,
) -> Result<(StackStatus, String), Error> {
    let this_stack = get_stack(client, stack_id).await?;
    let stack_status = this_stack.stack_status().unwrap();
    let stack_status_reason = this_stack.stack_status_reason().unwrap_or("");
    Ok((stack_status.clone(), stack_status_reason.to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Launch {
            name,
            instance_type,
            port,
            key_name,
            ssh_location,
        } => {
            let ssh_location = ssh_location.unwrap_or("0.0.0.0/0".to_string());
            let launch_template = LAUNCH_TEMPLATE.to_string();
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);
            let stack_output = launch_stack(
                &client,
                &launch_template,
                &name,
                &instance_type,
                &port,
                &key_name,
                &ssh_location,
            )
            .await?;
            let stack_id = match stack_output.stack_id() {
                Some(x) => x,
                None => {
                    return Err(failure::err_msg(
                        "Missing `stack_id` in CreateStackOutput, please check CloudFormation \
                        logs to determine the source of the error.",
                    ))
                }
            };
            let (stack_status, stack_status_reason) = loop {
                let (status, status_reason) = check_stack_status(&client, &stack_id).await?;
                tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
                if status != StackStatus::CreateInProgress {
                    break (status, status_reason);
                }
            };
            match stack_status {
                StackStatus::CreateComplete => {
                    println!(
                        "Successfully launched enclave with stack ID {:?}",
                        stack_output.stack_id().unwrap()
                    );
                }
                StackStatus::CreateFailed => {
                    return Err(failure::err_msg(
                        "Received CreateFailed status from CloudFormation stack, please check \
                        AWS console or AWS logs for more information.",
                    ))
                }
                other_status => {
                    return Err(failure::err_msg(format!(
                        "{:#?}: {}",
                        other_status, stack_status_reason
                    )))
                }
            }
            // Stack was created successfully, report outputs to stdout
            let this_stack = get_stack(&client, stack_id).await?;
            // TODO handle missing outputs in this unwrap, maybe w/ warning instead of error?
            println!("Enclave user information:");
            for output in this_stack.outputs().unwrap() {
                println!(
                    "\t{}: {}",
                    output.output_key().unwrap(),
                    output.output_value().unwrap()
                );
            }
            Ok(())
        }
        Commands::Build {
            dockerfile,
            context,
            eif,
        } => {
            Command::new("docker")
                .args(["build", "-t", "nitrogen-build", &context, "-f", &dockerfile])
                .output()
                .await
                .expect("failed to build docker image");
            let h = home::home_dir().unwrap_or_default();
            let out = Command::new("docker")
                .args([
                    "run",
                    "-v",
                    &format!("{}/.docker:/root/.docker", h.display()),
                    "-v",
                    "/var/run/docker.sock:/var/run/docker.sock",
                    "-v",
                    &format!("{}:/root/build", env::current_dir()?.to_str().unwrap_or("")),
                    "capeprivacy/eif-builder:latest",
                    "build-enclave",
                    "--docker-uri",
                    "nitrogen-build",
                    "--output-file",
                    &format!("/root/build/{}", eif),
                ])
                .output()
                .await?;
            println!("{:?}", out);
            Ok(())
        }
        Commands::Deploy{instance, eif, ssh_key, cpu_count, memory} => {
            println!("Deploying {} to the instance... (this may take some time, especially for larger files)", eif);
            let scp_out = Command::new("scp")
                .args([
                    "-i", 
                    &ssh_key,
                    &eif,
                    format!("ec2-user@{}:~", &instance).as_str(),
                ])
                .output()
                .await?;
            println!("{:?}", scp_out);
            println!("Running enclave...");
            let ssh_out = Command::new("ssh")
                .args([
                    "-i", 
                    &ssh_key,
                    format!("ec2-user@{}", &instance).as_str(),
                    "nitro-cli",
                    "run-enclave",
                    "--enclave-cid",
                    "16",
                    "--eif-path",
                    format!("~/{}", &eif).as_str(),
                    "--cpu-count",
                    &cpu_count,
                    "--memory",
                    &memory,
                ])
                .output()
                .await?;
            println!("{:?}", ssh_out);
            Ok(())
        }
        Commands::Delete { .. } => {
            todo!("implement delete command logic");
        }
    }
}
