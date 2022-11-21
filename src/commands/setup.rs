use crate::commands::utilities;
use aws_sdk_cloudformation::{
    model::{Parameter, StackStatus},
    output::CreateStackOutput,
    Client,
};
use failure::Error;
use std::fs;
use tracing::{info, instrument};

fn lift_to_param(key: impl Into<String>, value: impl Into<String>) -> Parameter {
    Parameter::builder()
        .parameter_key(key)
        .parameter_value(value)
        .build()
}

async fn setup_stack(
    client: &Client,
    setup_template: &String,
    name: &String,
    instance_type: &String,
    port: &usize,
    public_key: &String,
    ssh_location: &String,
) -> Result<CreateStackOutput, Error> {
    let stack = client
        .create_stack()
        .stack_name(name)
        .template_body(setup_template)
        .parameters(lift_to_param("InstanceName", name))
        .parameters(lift_to_param("InstanceType", instance_type))
        .parameters(lift_to_param("Port", port.to_string()))
        .parameters(lift_to_param("PublicKey", public_key))
        .parameters(lift_to_param("SSHLocation", ssh_location));
    let stack_output = stack.send().await?;
    Ok(stack_output)
}

#[instrument(level = "debug", skip(client, setup_template))]
pub async fn setup(
    client: &Client,
    setup_template: &String,
    name: &String,
    instance_type: &String,
    port: &usize,
    public_key_file: &String,
    ssh_location: &String,
) -> Result<Vec<(String, String)>, Error> {
    let public_key = fs::read_to_string(public_key_file)?;

    let stack_output = setup_stack(
        client,
        setup_template,
        name,
        instance_type,
        port,
        &public_key,
        ssh_location,
    )
    .await?;
    let stack_id = match stack_output.stack_id() {
        Some(x) => x,
        None => {
            return Err(failure::err_msg(
                "Missing `stack_id` in CreateStackOutput, please check CloudFormation \
                logs to determine the source of the error.",
            ))
        }
    };
    let (stack_status, stack_status_reason) = loop {
        let (status, status_reason) = utilities::check_stack_status(client, stack_id).await?;
        tokio::time::sleep(tokio::time::Duration::new(4, 0)).await;
        if status != StackStatus::CreateInProgress {
            break (status, status_reason);
        }
    };
    match stack_status {
        StackStatus::CreateComplete => {
            let stack_id = stack_output.stack_id().unwrap();
            info!(stack_id, "Successfully created enclave instance.");
        }
        StackStatus::CreateFailed => {
            return Err(failure::err_msg(
                "Received CreateFailed status from CloudFormation stack, please check \
                AWS console or AWS logs for more information.",
            ))
        }
        other_status => {
            return Err(failure::err_msg(format!(
                "{:#?}: {}",
                other_status, stack_status_reason
            )))
        }
    }
    // Stack was created successfully, report outputs to stdout
    let this_stack = utilities::get_stack(client, stack_id).await?;
    // TODO handle missing outputs in this unwrap, maybe w/ warning instead of error?
    let outputs: Vec<(String, String)> = this_stack
        .outputs()
        .unwrap()
        .iter()
        .map(|o| {
            let k = o.output_key().unwrap().to_string();
            let v = o.output_value().unwrap().to_string();
            (k, v)
        })
        .collect();
    Ok(outputs)
}
