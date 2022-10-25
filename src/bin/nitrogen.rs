use aws_sdk_cloudformation::Client;
use clap::{Parser, Subcommand};
use failure::Error;
use nitrogen::commands::{build, delete, deploy, setup};
use nitrogen::template::SETUP_TEMPLATE;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// Memory in MB to provision for the enclave
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

            println!("Enclave user information:");
            for (out_key, out_val) in outputs.iter() {
                println!("\t- {}: {}", out_key, out_val);
            }
            Ok(())
        }
        Commands::Build {
            dockerfile,
            context,
            eif,
        } => {
            let out = build(&dockerfile, &context, &eif).await?;
            println!("{:?}", out);
            Ok(())
        }
        Commands::Deploy {
            instance,
            eif,
            ssh_key,
            cpu_count,
            memory,
        } => {
            let out = deploy(
                &instance,
                &eif,
                &ssh_key,
                &cpu_count,
                memory.unwrap_or_default(),
            )
            .await?;
            println!("{:?}", out);
            Ok(())
        }
        Commands::Delete { name } => {
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);

            delete(&client, &name).await?;

            println!("Delete successful {}", name);
            Ok(())
        }
    }
}
