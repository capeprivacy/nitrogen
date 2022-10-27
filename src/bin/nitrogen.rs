use std::io;

use aws_sdk_cloudformation::Client;
use clap::{Parser, Subcommand};
use failure::Error;
use nitrogen::commands::{build, delete, deploy, setup};
use nitrogen::template::SETUP_TEMPLATE;
use tracing::{debug, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Provision nitro-enabled ec2 instances
    Setup {
        /// Name of the CloudFormation stack/provisioned EC2 instance
        name: String,
        /// EC2 key-pair to use for the provisioned instance
        key_name: String,
        /// EC2-instance type. Must be Nitro compatible
        #[arg(long, default_value_t = String::from("m5a.xlarge"))]
        instance_type: String,
        /// EC2-instance port for socat enclave connection
        #[arg(short, long, default_value_t = 5000)]
        port: usize,
        /// IP address range that can be used to SSH to the EC2 instance.
        #[arg(short, long, default_value_t = String::from("0.0.0.0/0"))]
        ssh_location: String,
    },

    /// Build a Nitro EIF from a given Dockerfile
    Build {
        /// Docker context directory
        context: String,
        /// Dockerfile location
        dockerfile: String,
        /// Output EIF location
        #[arg(short, long, default_value_t = String::from("./nitrogen.eif"))]
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
        #[arg(short, long, default_value_t = 2)]
        cpu_count: u8,
        /// Memory in MB to provision for the enclave. Defaults to 5x EIF size if not supplied.
        #[arg(short, long)]
        memory: Option<u64>,
    },

    /// Delete launched ec2 instance
    Delete {
        /// Name of the provisioned instance
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let tracing_directive = if cli.verbose {
        "nitrogen=debug,aws_config=info"
    } else {
        "nitrogen=info,aws_config=off"
    };
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(tracing_directive)
        .init();

    match cli.command {
        Commands::Setup {
            name,
            instance_type,
            port,
            key_name,
            ssh_location,
        } => {
            let ssh_location = ssh_location.to_string();
            let instance_type = instance_type.to_string();
            let setup_template = SETUP_TEMPLATE.to_string();
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);
            info!("Spinning up enclave instance '{}'.", name);
            let outputs = setup(
                &client,
                &setup_template,
                &name,
                &instance_type,
                &port,
                &key_name,
                &ssh_location,
            )
            .await?;

            info!(
                name,
                instance_id = outputs[0].1,
                public_ip = outputs[1].1,
                availability_zone = outputs[2].1,
                public_dns = outputs[3].1,
                "User enclave information:"
            );
            // TODO return outputs as a JSON struct
            Ok(())
        }
        Commands::Build {
            dockerfile,
            context,
            eif,
        } => {
            info!(context, dockerfile, "Building EIF from dockerfile.");
            let out = build(&dockerfile, &context, &eif).await?;
            debug!(docker_output=?out, "Docker output:");
            Ok(())
        }
        Commands::Deploy {
            instance,
            eif,
            ssh_key,
            cpu_count,
            memory,
        } => {
            info!(eif, "Deploying EIF to {}", instance);
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);
            let out = deploy(&client, &instance, &eif, &ssh_key, &cpu_count, memory).await?;
            debug!("{:?}", out);
            Ok(())
        }
        Commands::Delete { name } => {
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);

            info!("Deleting enclave stack '{}'.", name);
            delete(&client, &name).await?;
            Ok(())
        }
    }
}
