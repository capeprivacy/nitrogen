use crate::commands::setup::get_stack;
use aws_sdk_cloudformation::{
    model::{Output as CloudOutput, Stack},
    Client,
};
use failure::Error;
use std::{
    fs,
    process::{Command, Output},
};
use tracing::{debug, info, instrument};

async fn get_instance_url(stack: &Stack) -> Result<String, Error> {
    let outputs: Vec<&CloudOutput> = stack
        .outputs()
        .unwrap_or_default()
        .iter()
        .filter(|x| {
            if x.output_key().unwrap_or_default() == "PublicDNS" {
                return true;
            }
            false
        })
        .collect();

    if outputs.is_empty() {
        return Err(failure::err_msg("unable to query public dns"));
    }
    let instance_url = outputs[0].output_value().unwrap_or_default();
    Ok(instance_url.to_string())
}

#[instrument(level = "debug")]
pub async fn deploy(
    client: &Client,
    instance: &str,
    eif: &String,
    ssh_key: &String,
    cpu_count: &u8,
    memory: Option<u64>,
) -> Result<Output, Error> {
    let this_stack = get_stack(&client, instance).await?;
    let url = get_instance_url(&this_stack).await?;

    // If eif is
    let metadata = fs::metadata(eif)?;
    let eif_size = metadata.len() / 1000000; // to mb
    let mem = if memory.is_none() {
        eif_size * 5
    } else {
        memory.unwrap()
    };

    info!("Terminating any existing enclaves");
    let terminate_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
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
    debug!(stdout=?terminate_out);

    info!(memory = mem, "Updating enclave allocator memory");
    let sed_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
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
    debug!(stdout=?sed_out);

    let systemctl_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
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
    debug!(std_out=?systemctl_out);

    info!(
        "Deploying {} to the instance (this may take some time, especially for larger files)",
        eif
    );
    let scp_out = Command::new("scp")
        .args(["-i", ssh_key, eif, format!("ec2-user@{}:~", &url).as_str()])
        .output()?;
    if !scp_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to copy eif to enclave host {:?}",
            scp_out
        )));
    }
    debug!(stdout=?scp_out);

    info!("Running enclave...");
    let run_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
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
