use clap::{Parser, Subcommand};

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
        #[arg(short,long)]
        instance_name: String,
        /// EC2-instance type. Must be Nitro compatible.
        #[arg(short,long)]
        instance_type: String,
        /// EC2-instance port for socat enclave connection
        #[arg(short,long)]
        socat_port: usize,
        /// SSH key-pair
        #[arg(short,long)]
        key_name: String,
        /// IP address range that can be used to SSH to the EC2 instance. Defaults to anywhere ("0.0.0.0/0").
        #[arg(short,long)]
        ssh_location: Option<String>,
    },

    /// Build a Nitro EIF from a given Dockerfile
    Build {
        // Dockerfile location
        #[arg(short,long)]
        dockerfile: String,
        // Output EIF location
        #[arg(short,long)]
        eif: String,
    },

    /// Deploy an EIF to a provisioned nitro ec2 instance
    Deploy {
        /// Name of the provisioned instance
        instance: String,
        // Filepath to EIF
        eif: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Launch{name, instance_type, port, ssh_location} => {
            println!("Launching instance...");
            println!("Name: {}", name);
            println!("Instance type: {}", instance_type);
            println!("Port: {}", port);
            let _ssh_location = ssh_location.unwrap_or("0.0.0.0/0");
            todo!("implement launch command logic");
        }
        Commands::Build{..} => {
            todo!("implement build command logic");
        }
        Commands::Deploy{..} => {
            todo!("implement deploy command logic");
        }
    }
}
