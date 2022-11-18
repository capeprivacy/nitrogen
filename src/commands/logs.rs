use crate::commands::deploy::describe_enclave;
use crate::commands::deploy::get_instance_url;
use crate::commands::setup::get_stack;
use aws_sdk_cloudformation::Client;
use failure::Error;
use std::process::{Command, Output, Stdio};
use std::str;
use tracing::{debug, info, instrument};

#[instrument(level = "debug")]
pub async fn logs(client: &Client, stack_name: &str, ssh_key: &str) -> Result<Output, Error> {
    let this_stack = get_stack(client, stack_name).await?;
    let url = get_instance_url(&this_stack).await?;

    let enclave = describe_enclave(ssh_key, &url)?;
    let enclave_name = match enclave.get("EnclaveName") {
        Some(name) => name,
        None => return Err(failure::err_msg("Enclave has no name.")),
    };

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

    debug!(stdout=?console_out);

    if !console_out.status.success() {
        return Err(failure::err_msg(format!(
            "failed to get enclave console{:?}",
            console_out
        )));
    }

    Ok(console_out)
}
