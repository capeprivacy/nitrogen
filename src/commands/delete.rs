use super::setup::check_stack_status;
use aws_sdk_cloudformation::{model::StackStatus, Client};
use failure::Error;

async fn delete_stack(client: &Client, name: &String) -> Result<(), Error> {
    // TODO tokio tracing, consider instrument
    println!("Deleting instance...");
    println!("Instance Name: {}", name);

    client.delete_stack().stack_name(name).send().await?;

    Ok(())
}

pub async fn delete(client: &Client, name: &String) -> Result<(), Error> {
    delete_stack(client, name).await?;

    let (stack_status, stack_status_reason) = loop {
        let (status, status_reason) = check_stack_status(client, name).await?;
        tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
        if status != StackStatus::DeleteInProgress {
            break (status, status_reason);
        }
    };
    match stack_status {
        StackStatus::DeleteComplete => {
            println!("Successfully deleted stack {:?}", name);
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
