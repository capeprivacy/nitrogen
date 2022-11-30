use crate::cf_utilities as utilities;
use aws_sdk_cloudformation::Client;
use failure::Error;
use serde_json::json;
use std::process::{Command, Stdio};
use std::str;
use tracing::{error, info, instrument};

#[instrument(level = "debug")]
pub async fn logs(client: &Client, stack_name: &str, ssh_key: &str) -> Result<(), Error> {
    let this_stack = utilities::get_stack(client, stack_name).await?;
    let url = utilities::get_instance_url(&this_stack).await?;

    let enclave = utilities::describe_enclave(ssh_key, &url)?;
    let enclave_name = match enclave.get("EnclaveName") {
        Some(name) => name,
        None => return Err(failure::err_msg("Enclave has no name.")),
    };

    if enclave.get("Flags") != Some(&json!("DEBUG_MODE")) {
        error!("Enclave is not in debug mode. Please redeploy with \"--debug-mode\" flag.");
        return Ok(());
    }

    info!("Getting logs from enclave console: {}", url);
    let console_out = Command::new("ssh")
        .args([
            "-i",
            ssh_key,
            &format!("ec2-user@{}", url),
            "nitro-cli",
            "console",
            "--enclave-name",
            &enclave_name.to_string(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !console_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to get enclave console{:?}",
            console_out
        )));
    }

    Ok(())
}
