use failure::Error;
use std::path::Path;

use aws_sdk_cloudformation::{model::Parameter, output::CreateStackOutput, Client};
use clap::{Parser, Subcommand};
use tokio::fs;

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
        /// SSH key-pair
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
        // Output EIF location
        #[arg(short, long)]
        eif: String,
    },

    /// Deploy an EIF to a provisioned nitro ec2 instance
    Deploy {
        /// Name of the provisioned instance
        instance: String,
        // Filepath to EIF
        eif: String,
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
        // TODO socat port parameter
        .parameters(lift_to_param("KeyName", key_name))
        .parameters(lift_to_param("SSHLocation", ssh_location));
    let stack_output = stack.send().await?;
    Ok(stack_output)
}

async fn _check_stack(client: &Client, stack_output: &CreateStackOutput) -> Result<(), Error> {
    // TODO
    let stack_id = stack_output.stack_id().unwrap();
    let resp = client.describe_stacks().stack_name(stack_id).send().await?;
    let this_stack = resp.stacks().unwrap_or_default().first().unwrap();
    let _stack_status = this_stack.stack_status();
    Ok(())
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
            // TODO bundle this template file into binary as string const w/ `build.rs`
            let launch_template =
                fs::read_to_string(Path::new("src/templates/launchTemplate.json")).await?;
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

            println!(
                "Successfully launched enclave with stack ID {:?}",
                stack_output.stack_id().unwrap()
            );
            Ok(())
        }
        Commands::Build { .. } => {
            todo!("implement build command logic");
        }
        Commands::Deploy { .. } => {
            todo!("implement deploy command logic");
        }
        Commands::Delete{..} => {
            todo!("implement delete command logic");
        }
    }
}
