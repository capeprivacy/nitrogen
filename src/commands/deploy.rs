use crate::commands::setup::get_stack;
use aws_sdk_cloudformation::{
    model::{Output as CloudOutput, Stack},
    Client,
};
use failure::Error;
use serde_json::Value;
use std::str;
use std::{
    fs,
    process::{Command, Output},
};
use tracing::{debug, error, info, instrument};

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

fn terminate_existing_enclaves(ssh_key: &str, url: &str) -> Result<(), Error> {
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

    debug!(stdout=?terminate_out);

    if !terminate_out.status.success() {
        Err(failure::err_msg(format!(
            "failed to terminate any currently running enclaves {:?}",
            terminate_out
        )))
    } else {
        Ok(())
    }
}

fn update_allocator_memory(memory: u64, ssh_key: &str, url: &str) -> Result<(), Error> {
    info!(memory, "Updating enclave allocator memory (in MB).");
    let sed_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
            "sudo",
            "sed",
            "-i",
            format!("'s/memory_mib: .*/memory_mib: {}/g'", memory).as_str(),
            "/etc/nitro_enclaves/allocator.yaml",
        ])
        .output()?;

    debug!(stdout=?sed_out);
    if !sed_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to update allocator config with sed {:?}",
            sed_out
        )));
    }

    info!("Restarting enclave allocator service.");
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

    debug!(std_out=?systemctl_out);
    if !systemctl_out.status.success() {
        Err(failure::err_msg(format!(
            "failed to restart allocator after reconfig {:?}",
            systemctl_out
        )))
    } else {
        Ok(())
    }
}

fn deploy_eif(eif_path: &str, ssh_key: &str, url: &str) -> Result<(), Error> {
    info!(
        "Deploying {} to the instance http://{} (this may take some time, especially for larger files)",
        eif_path,
        url
    );
    let scp_out = Command::new("scp")
        .args([
            "-i",
            ssh_key,
            eif_path,
            format!("ec2-user@{}:~", url).as_str(),
        ])
        .output()?;
    debug!(stdout=?scp_out);

    if !scp_out.status.success() {
        Err(failure::err_msg(format!(
            "failed to copy eif to enclave host {:?}",
            scp_out
        )))
    } else {
        Ok(())
    }
}

fn run_eif(
    eif_path: &str,
    cpu_count: u64,
    mem: &u64,
    ssh_key: &str,
    url: &str,
) -> Result<Output, Error> {
    info!("Running EIF in enclave.");
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
            format!("~/{}", eif_path).as_str(),
            "--cpu-count",
            cpu_count.to_string().as_str(),
            "--memory",
            mem.to_string().as_str(),
        ])
        .output()?;
    debug!(stdout=?run_out);

    if !run_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to run enclave{:?}",
            run_out
        )));
    }

    match check_enclave_status(ssh_key, url) {
        Ok(_) => info!("Enclave up and running!"),
        Err(err) => error!("Error: something went wrong with deployment. {}", err),
    }

    Ok(run_out)
}

fn check_enclave_status(ssh_key: &str, url: &str) -> Result<(), Error> {
    info!("Check enclave status...");
    let describe_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            format!("ec2-user@{}", url).as_str(),
            "nitro-cli",
            "describe-enclaves",
        ])
        .output()?;
    debug!(stdout=?describe_out);

    if !describe_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to get enclave info{:?}",
            describe_out
        )));
    };

    let json: Value = serde_json::from_slice(&describe_out.stdout)
        .expect("Could not parse response from AWS as json.");

    let enclave_list = json.as_array().unwrap();

    let enclave = match enclave_list.get(0) {
        None => return Err(failure::err_msg("No running enclaves detected.")),
        Some(enclave) => enclave,
    };

    let state = match enclave.get("State") {
        None => {
            return Err(failure::err_msg(
                "Enclave detected, but has no state description.",
            ))
        }
        Some(current_state) => current_state.as_str(),
    };

    match state {
        Some("RUNNING") => Ok(()),
        _ => Err(failure::err_msg(
            "Enclave detected, but not in running state.",
        )),
    }
}

#[instrument(level = "debug")]
pub async fn deploy(
    client: &Client,
    stack_name: &str,
    eif: &String,
    ssh_key: &String,
    cpu_count: u64,
    memory: Option<u64>,
) -> Result<Output, Error> {
    let this_stack = get_stack(client, stack_name).await?;
    let url = get_instance_url(&this_stack).await?;

    // If enclave memory not specified, default to 5x eif size
    let metadata = fs::metadata(eif)?;
    let eif_size = metadata.len() / 1000000; // to mb
    let mem = if memory.is_none() {
        eif_size * 5
    } else {
        memory.unwrap()
    };

    info!("Using instance URL {}...", url);
    terminate_existing_enclaves(ssh_key, &url)?;
    update_allocator_memory(mem, ssh_key, &url)?;
    deploy_eif(eif, ssh_key, &url)?;
    run_eif(eif, cpu_count, &mem, ssh_key, &url)
}
