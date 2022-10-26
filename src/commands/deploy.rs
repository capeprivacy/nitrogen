use failure::Error;
use std::{
    fs,
    process::{Command, Output},
};

pub async fn deploy(
    instance: &String,
    eif: &String,
    ssh_key: &String,
    cpu_count: &u8,
    memory: u64,
) -> Result<Output, Error> {
    let metadata = fs::metadata(eif)?;
    let eif_size = metadata.len() / 1000000; // to mb

    let mut mem = memory;
    if mem == 0 {
        mem = eif_size * 5;
    }

    let terminate_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", instance).as_str(),
            "nitro-cli",
            "terminate-enclave",
            "--all",
        ])
        .output()?;
    if !terminate_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to terminate any currently running enclaves {:?}",
            terminate_out
        )));
    }
    println!("{:?}", terminate_out);

    let sed_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", instance).as_str(),
            "sudo",
            "sed",
            "-i",
            format!("'s/memory_mib: .*/memory_mib: {}/g'", mem).as_str(),
            "/etc/nitro_enclaves/allocator.yaml",
        ])
        .output()?;
    if !sed_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to update allocator config with sed {:?}",
            sed_out
        )));
    }
    println!("{:?}", sed_out);

    let systemctl_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", instance).as_str(),
            "sudo",
            "systemctl",
            "restart",
            "nitro-enclaves-allocator.service",
        ])
        .output()?;
    if !systemctl_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to restart allocator after reconfig {:?}",
            systemctl_out
        )));
    }
    println!("{:?}", systemctl_out);

    println!(
        "Deploying {} to the instance...\n(this may take some time, especially for larger files)",
        eif
    );
    let scp_out = Command::new("scp")
        .args([
            "-i",
            ssh_key,
            eif,
            format!("ec2-user@{}:~", &instance).as_str(),
        ])
        .output()?;
    if !scp_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to copy eif to enclave host {:?}",
            scp_out
        )));
    }
    println!("{:?}", scp_out);

    println!("Running enclave...");
    let run_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", instance).as_str(),
            "nitro-cli",
            "run-enclave",
            "--enclave-cid",
            "16",
            "--eif-path",
            format!("~/{}", eif).as_str(),
            "--cpu-count",
            cpu_count.to_string().as_str(),
            "--memory",
            mem.to_string().as_str(),
        ])
        .output()?;
    if !run_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to run enclave{:?}",
            run_out
        )));
    }
    Ok(run_out)
}
