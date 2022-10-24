use aws_sdk_cloudformation::{
    model::{Parameter, Stack, StackStatus},
    Client,
};
use failure::Error;

async fn delete_stack(client: &Client, name: &String) -> Result<(), Error> {
    // TODO tokio tracing, consider instrument
    println!("Deleting instance...");
    println!("Instance Name: {}", name);

    client.delete_stack()
        .stack_name(name)
        .parameters(lift_to_param("InstanceName", name))
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

    let resp = match delete_output {
        Ok() => Ok(),
        Err(error) => {
            return Err(failure::err_msg(
                "Deleting stack failed, please check CloudFormation \
                logs to determine the source of the error.",
            ))
        }
    };
    let (stack_status, stack_status_reason) = loop {
        let (status, status_reason) = check_stack_status(client, stack_id).await?;
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
