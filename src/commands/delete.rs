use aws_sdk_cloudformation::{
    model::StackStatus,
    Client,
};
use failure::Error;

fn lift_to_param(key: impl Into<String>, value: impl Into<String>) -> Parameter {
    Parameter::builder()
        .parameter_key(key)
        .parameter_value(value)
        .build()
}

async fn get_stack(client: &Client, name: &str) -> Result<Stack, Error> {
    let resp = client.describe_stacks().stack_name(name).send().await?;
    let this_stack = resp.stacks().unwrap_or_default().first().unwrap();
    Ok(this_stack.clone())
}

async fn check_stack_status(
    client: &Client,
    name: &str,
) -> Result<(StackStatus, String), Error> {
    let this_stack = get_stack(client, name).await?;
    let stack_status = this_stack.stack_status().unwrap();
    let stack_status_reason = this_stack.stack_status_reason().unwrap_or("");
    Ok((stack_status.clone(), stack_status_reason.to_string()))
}

async fn delete_stack(client: &Client, name: &String) -> Result<(), Error> {
    // TODO tokio tracing, consider instrument
    println!("Deleting instance...");
    println!("Instance Name: {}", name);

    client.delete_stack()
        .stack_name(name)
        .send()
        .await?;

    Ok(())
}

pub async fn delete(
    client: &Client,
    name: &String,
) -> Result<(), Error> {

    // TODO get to check if it exists
    let delete_output = delete_stack(client, name).await?;

    let success = match delete_output {
        Ok(()) => (),
        Err(..) => {
            return Err(failure::err_msg(
                "Deleting stack failed, please check CloudFormation \
                logs to determine the source of the error.",
            ))
        }
    };
    let (stack_status, stack_status_reason) = loop {
        let (status, status_reason) = check_stack_status(client, name).await?;
        tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
        if status != StackStatus::DeleteInProgress {
            break (status, status_reason);
        }
    };
    match stack_status {
        StackStatus::DeleteComplete => {
            println!(
                "Successfully deleted stack {:?}", name
            );
        }
        StackStatus::DeleteFailed => {
            return Err(failure::err_msg(
                "Received DeleteFailed status from CloudFormation stack, please check \
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

    Ok(())
}
