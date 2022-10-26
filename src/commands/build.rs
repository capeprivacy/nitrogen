use failure::Error;
use home;
use std::env;
use std::process::Output;
use tokio::process::Command;

pub async fn build(
    dockerfile: &String,
    context: &String,
    eif_name: &String,
) -> Result<Output, Error> {
    let out = Command::new("docker")
        .args(["build", "-t", "nitrogen-build", context, "-f", dockerfile])
        .output()
        .await?;
    if !out.status.success() {
        return Err(failure::err_msg(format!(
            "unable to build docker image {:?}",
            out
        )));
    }

    let h = home::home_dir().unwrap_or_default();
    let out = Command::new("docker")
        .args([
            "run",
            "-v",
            &format!("{}/.docker:/root/.docker", h.display()),
            "-v",
            "/var/run/docker.sock:/var/run/docker.sock",
            "-v",
            &format!(
                "{}:/root/build",
                env::current_dir()?.to_str().unwrap_or_default()
            ),
            "capeprivacy/eif-builder:latest",
            "build-enclave",
            "--docker-uri",
            "nitrogen-build",
            "--output-file",
            &format!("/root/build/{}", eif_name),
        ])
        .output()
        .await?;
    Ok(out)
}
