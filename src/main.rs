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
        name: String,
        /// EC2-instance type. Must be Nitro compatible.
        instance_type: String,
        /// EC2-instance port for socat enclave connection
        port: usize,
    },

    /// Build a Nitro EIF from a given Dockerfile
    Build {
        // Dockerfile location
        dockerfile: String,
        // Output EIF location
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
        Commands::Launch{name, instance_type, port} => {
            println!("Launching instance...");
            println!("Name: {}", name);
            println!("Instance type: {}", instance_type);
            println!("Port: {}", port);
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
