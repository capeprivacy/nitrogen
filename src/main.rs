use clap::{Parser, Subcommand};
use std::process::Command;


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
        Commands::Deploy{instance, eif, ssh_key, cpu_count, memory} => {
            loop {
                let _result = Command::new("ssh")
                    .args(["-i", &ssh_key, &("ec2-user@".to_owned()+&instance)])
                    .args(["nitro-cli", "run-enclave"])
                    .args(["--encalve-cid", "16"])
                    .args(["--eif-path", &eif, "--cpu-count", &cpu_count.to_string()])
                    .args(["--memory", &memory.to_string()])
                    .output()
                    .expect("command failed");
                // io::stdout().write_all(&result.stdout).unwrap();
                // io::stderr().write_all(&result.stderr).unwrap();
            }
        }
    }
}
