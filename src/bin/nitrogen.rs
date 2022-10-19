use failure::Error;
use std::path::Path;

use aws_sdk_cloudformation::{Client, Region};
use clap::{Parser, Subcommand};
use home;
use nitro_cli::build_from_docker;
use tokio::fs;
use tokio::process::Command;

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
        /// Name of the provisioned EC2 instance
        #[arg(long)]
        instance_name: String,
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let shared_config = aws_config::from_env();

    match cli.command {
        Commands::Launch {
            instance_name,
            instance_type,
            port,
            key_name,
            ssh_location,
        } => {
            println!("Launching instance...");
            println!("Instance Name: {}", instance_name);
            println!("Instance type: {}", instance_type);
            println!("Socat Port: {}", port);
            println!("Key Name: {}", key_name);
            let _ssh_location = ssh_location.unwrap_or("0.0.0.0/0".to_string());
            todo!("implement launch command logic");
            // TODO bundle this template into binary w/ `build.rs`et cmd =
            let launch_template =
                fs::read_to_string(Path::new("src/templates/launchTemplate.json")).await?;
        }
        Commands::Build { dockerfile, .. } => {
            Command::new("docker")
                .args(["build", "-t", "nitrogen-build", ".", "-f", &dockerfile])
                .output()
                .await
                .expect("failed to build docker image");

            // docker run -v ~/.docker:/root/.docker -v /var/run/docker.sock:/var/run/docker.sock capeprivacy/eif-builder build-enclave --docker-uri docker.io/capeprivacy/runtime:release-ffc4e1c --output-file runtime.eif

            let h = home::home_dir().unwrap();
            let out = Command::new("docker")
                .args([
                    "run",
                    "-v",
                    &format!("{}/.docker:/root/.docker", h.display()),
                    "-v",
                    "/var/run/docker.sock:/var/run/docker.sock",
                    "capeprivacy/eif-builder:latest",
                    "build-enclave",
                    "--docker-uri",
                    "nitrogen-build",
                    "--output-file",
                    "runtime.eif",
                ])
                .output()
                .await
                .unwrap();
            println!("{:?}", out);
            Ok(())
        }
        Commands::Deploy { .. } => {
            todo!("implement deploy command logic");
        }
        Commands::Delete{..} => {
            todo!("implement delete command logic");
        }
    }
}
