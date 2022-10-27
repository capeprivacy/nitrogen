use crate::commands::setup::check_stack_status;
use crate::commands::setup::get_stack;
use aws_sdk_cloudformation::{model::StackStatus, Client};
use failure::Error;
use tracing::{info, instrument};

async fn delete_stack(client: &Client, name: &String) -> Result<(), Error> {
    client.delete_stack().stack_name(name).send().await?;
    Ok(())
}

#[instrument(level = "debug", skip(client))]
pub async fn delete(client: &Client, name: &String) -> Result<(), Error> {
    let this_stack = get_stack(client, name).await?;
    let stack_id = this_stack.stack_id().unwrap_or_default();

    delete_stack(client, name).await?;

    let (stack_status, stack_status_reason) = loop {
        let (status, status_reason) = check_stack_status(client, stack_id).await?;
        tokio::time::sleep(tokio::time::Duration::new(4, 0)).await;
        if status != StackStatus::DeleteInProgress {
            break (status, status_reason);
        }
    };
    match stack_status {
        StackStatus::DeleteComplete => {
            info!("Successfully deleted stack '{}'.", name);
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
