use failure::Error;
use home;
use std::path::PathBuf;
use std::env;
use std::process::ExitStatus;
use tokio::process::Command;
use tracing::{info, instrument};

#[instrument(level = "debug")]
pub async fn build(
    dockerfile_dir: &String,
    dockerfile_name: &String,
    eif_name: &String,
) -> Result<ExitStatus, Error> {
    let dockerdir = PathBuf::from(dockerfile_dir);
    let mut dockerfile_path = PathBuf::from(dockerfile_dir);
    dockerfile_path.push(dockerfile_name);

    let image_builder_process = Command::new("docker")
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
        .spawn()?
        .wait().await?;
    if !image_builder_process.success() {
        return Err(failure::err_msg("Docker nitrogen-build error."));
    }

    let h = home::home_dir().unwrap_or_default();
    let cwd = env::current_dir()?;
    let eif_dir = cwd.to_str().unwrap_or_default();
    let eif_builder_process = Command::new("docker")
        .args([
            "run",
            "-v",
            &format!("{}/.docker:/root/.docker", h.display()),
            "-v",
            "/var/run/docker.sock:/var/run/docker.sock",
            "-v",
            &format!("{}:/root/build", eif_dir,),
            "capeprivacy/eif-builder:latest",
            "build-enclave",
            "--docker-uri",
            "nitrogen-build",
            "--output-file",
            &format!("/root/build/{}", eif_name),
        ])
        .spawn()?
        .wait().await?;
    if !eif_builder_process.success() {
        return Err(failure::err_msg("Docker eif-builder error."));
    } else {
        let path_buf = cwd.join(eif_name);
        info!("EIF written to {}", path_buf.display());
    }
    Ok(eif_builder_process)
}
