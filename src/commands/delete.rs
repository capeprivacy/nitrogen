use aws_sdk_cloudformation::{
    model::{Stack, StackStatus},
    Client,
};
use failure::Error;
use super::launch::get_stack;
use super::launch::check_stack_status;

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
