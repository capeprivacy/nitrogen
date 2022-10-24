use failure::Error;
use std::process::Output;
use tokio::process::Command;

pub async fn deploy(
    instance: &String,
    eif: &String,
    ssh_key: &String,
    cpu_count: &usize,
    memory: &usize,
) -> Result<Output, Error> {
    let scp_out = Command::new("scp")
        .args([
            "-i",
            ssh_key,
            eif,
            format!("ec2-user@{}:~", instance).as_str(),
        ])
        .output()
        .await?;
    println!("{:?}", scp_out);
    println!("Running enclave...");
    let ssh_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", instance).as_str(),
            "nitro-cli",
            "run-enclave",
            "--enclave-cid",
            "16",
            "--eif-path",
            format!("~/{}", &eif).as_str(),
            "--cpu-count",
            &cpu_count.to_string(),
            "--memory",
            &memory.to_string(),
        ])
        .output()
        .await?;
    Ok(ssh_out)
}
