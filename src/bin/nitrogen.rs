use rand::{distributions::Alphanumeric, Rng};
use std::env::temp_dir;
use std::fs::{create_dir, File};
use std::io;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use aws_sdk_cloudformation::Client;
use clap::{Parser, Subcommand};
use failure::Error;
use nitrogen::commands::{build, delete, deploy, setup};
use nitrogen::template::SETUP_TEMPLATE;
use tracing::{debug, info};

use rust_embed::{EmbeddedFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "examples/"]
struct Asset;

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
        /// File of public key to be used for ssh with the provisioned instance
        public_key_file: String,
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
        /// Dockerfile directory
        dockerfile_dir: String,
        /// Output EIF location
        #[arg(short, long, default_value_t = String::from("./nitrogen.eif"))]
        eif: String,
    },

    /// Deploy an EIF to a provisioned nitro ec2 instance
    Deploy {
        /// Domain of the provisioned ec2 instance
        instance: String,
        /// Filepath to EIF
        #[arg(short, long, default_value_t = String::from("./nitrogen.eif"))]
        eif: String,
        /// Filepath to SSH key for the instance
        ssh_key: String,
        /// Number of CPUs to provision for the enclave
        #[arg(short, long, default_value_t = 2)]
        cpu_count: u64,
        /// Memory in MB to provision for the enclave. Defaults to 5x EIF size if not supplied.
        #[arg(short, long)]
        memory: Option<u64>,
    },

    /// Delete launched ec2 instance
    Delete {
        /// Name of the provisioned instance
        name: String,
    },

    Start {
        /// Name of the CloudFormation stack/provisioned EC2 instance
        name: String,
        /// File of public key to be used for ssh with the provisioned instance
        public_key_file: String,
        /// File of private key to be used for ssh
        private_key: String,
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
            public_key_file,
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
                &public_key_file,
                &ssh_location,
            )
            .await?;

            info!(
                "Open ports: {}, {}",
                "22",
                port,
            );
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
            dockerfile_dir,
            eif,
        } => {
            info!(dockerfile_dir, "Building EIF from dockerfile.");
            let out = build(&dockerfile_dir, &eif).await?;
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
            let out = deploy(&client, &instance, &eif, &ssh_key, cpu_count, memory).await?;
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
        Commands::Start {
            name,
            public_key_file,
            port,
            instance_type,
            ssh_location,
            private_key,
        } => {
            let dockerfile =
                Asset::get(&format!("{}/Dockerfile", name)).expect("unable to get dockerfile");
            let appsh = Asset::get(&format!("{}/app.sh", name)).expect("unable to get app.sh");
            let runsh = Asset::get(&format!("{}/run.sh", name)).expect("unable to get run.sh");

            let dir = temp_dir();

            let random_id: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect();

            let id = format!("{}-{}", name, random_id);

            let proj_dir = dir.as_path().join(&id);

            create_dir(&proj_dir)?;

            let dockerfile_path = &proj_dir.join("Dockerfile");

            create_file(dockerfile_path, dockerfile)?;
            create_file(&proj_dir.join("run.sh"), appsh)?;
            create_file(&proj_dir.join("app.sh"), runsh)?;

            let ssh_location = ssh_location.to_string();
            let instance_type = instance_type.to_string();
            let setup_template = SETUP_TEMPLATE.to_string();
            let shared_config = aws_config::from_env().load().await;
            let client = Client::new(&shared_config);
            setup(
                &client,
                &setup_template,
                &id,
                &instance_type,
                &port,
                &public_key_file,
                &ssh_location,
            )
            .await?;

            // TODO should save this somewhere else than their current directory
            let eif_path = &format!("{}.eif", name);

            build(
                &proj_dir.to_str().unwrap().to_string(),
                eif_path,
            )
            .await?;

            println!("Sleeping for 20s to give ec2 instance a chance to boot...");
            tokio::time::sleep(Duration::from_secs(20)).await;

            let out = deploy(&client, &id, eif_path, &private_key, 2, None).await?;

            println!("{:?}", out);

            Ok(())
        }
    }
}

fn create_file(path: &Path, embedded: EmbeddedFile) -> Result<(), Error> {
    let mut f = File::create(path)?;
    let bytes = embedded.data.as_ref();

    f.write_all(bytes)?;

    Ok(())
}
