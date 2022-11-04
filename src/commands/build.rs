use failure::Error;
use home;
use std::env;
use std::path::PathBuf;
use std::process::Output;
use tokio::process::Command;
use tracing::instrument;

#[instrument(level = "debug")]
pub async fn build(dockerfile_dir: &String, eif_name: &String) -> Result<Output, Error> {
    let dockerdir = PathBuf::from(dockerfile_dir);
    let mut dockerfile_path = PathBuf::from(dockerfile_dir);
    dockerfile_path.push("Dockerfile");

    let out = Command::new("docker")
        .args([
            "build",
            "-t",
            "nitrogen-build",
            "--platform",
            "linux/amd64",
            dockerdir.to_str().unwrap(),
            "-f",
            dockerfile_path.to_str().unwrap(),
        ])
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
