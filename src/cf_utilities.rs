use aws_sdk_cloudformation::{
    model::{Output as CloudOutput, Stack, StackStatus},
    Client,
};
use failure::Error;
use serde_json::{from_slice, json, Value};
use std::process::Command;
use tracing::{debug, info};

pub(crate) async fn get_stack(client: &Client, stack_id: &str) -> Result<Stack, Error> {
    let resp = client.describe_stacks().stack_name(stack_id).send().await?;
    let this_stack = resp.stacks().unwrap_or_default().first().unwrap();
    Ok(this_stack.clone())
}

pub(crate) async fn check_stack_status(
    client: &Client,
    stack_id: &str,
) -> Result<(StackStatus, String), Error> {
    let this_stack = get_stack(client, stack_id).await?;
    let stack_status = this_stack.stack_status().unwrap();
    let stack_status_reason = this_stack.stack_status_reason().unwrap_or("");
    Ok((stack_status.clone(), stack_status_reason.to_string()))
}

pub(crate) async fn get_instance_url(stack: &Stack) -> Result<String, Error> {
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

pub(crate) fn describe_enclave(ssh_key: &str, url: &str) -> Result<Value, Error> {
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

    let json: Value = match from_slice(&describe_out.stdout) {
        Ok(json) => json,
        Err(_) => return Err(failure::err_msg("Could not parse AWS response.")),
    };

    let description = match json.as_array() {
        Some(enclaves) => match enclaves.get(0) {
            Some(enclave) => enclave,
            None => return Err(failure::err_msg("Enclave not created.")),
        },
        None => return Err(failure::err_msg("Enclave not created.")),
    };

    Ok(description.clone())
}

pub(crate) fn check_enclave_status(ssh_key: &str, url: &str) -> Result<(), Error> {
    info!("Check enclave status...");

    match describe_enclave(ssh_key, url)?.get("State") {
        // According to the docs, the state is either "running" or "terminating"
        // https://docs.aws.amazon.com/enclaves/latest/user/cmd-nitro-describe-enclaves.html
        Some(x) if x.eq(&json!("RUNNING")) => Ok(()),
        Some(x) => Err(failure::err_msg(format!("Enclave created, but is {}.", x))),
        None => Err(failure::err_msg("Enclave created, but unknown state.")),
    }
}
