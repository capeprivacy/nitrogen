use aws_sdk_kms::{model::KeySpec, model::KeyUsageType, Client};
use failure::Error;
use gtmpl::template;
use gtmpl_derive::Gtmpl;
use home;
use serde::Deserialize;
use std::env;
use tokio::process::Command;
use tracing::instrument;
use uuid::Uuid;

pub const KEY_POLICY_TEMPLATE: &str = r##"{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "Allow access for Key Administrators",
            "Effect": "Allow",
            "Principal": {
                "AWS": "{{ .principal }}"
            },
            "Action": [
                "kms:ScheduleKeyDeletion",
                "kms:Revoke*",
                "kms:List*",
                "kms:GetKeyPolicy",
                "kms:Encrypt",
                "kms:GetPublicKey",
                "kms:Enable*",
                "kms:Disable*",
                "kms:Describe*",
                "kms:Delete*",
                "kms:Create*",
                "kms:CancelKeyDeletion"
            ],
            "Resource": "*"
        },
        {
            "Sid": "Enabled from enclave ONLY",
            "Effect": "Allow",
            "Principal": {
                "AWS": "{{ .principal }}"
            },
            "Action": "kms:Decrypt",
            "Resource": "*",
            "Condition": {
                "ForAnyValue:StringEqualsIgnoreCase": {
                    "kms:RecipientAttestation:PCR0": ["{{ .pcr0 }}"],
                    "kms:RecipientAttestation:PCR1": ["{{ .pcr1 }}"],
                    "kms:RecipientAttestation:PCR2": ["{{ .pcr2 }}"]
                }
            }
        },
        {
            "Sid": "Enabled from anywhere",
            "Effect": "Allow",
            "Principal": {
                "AWS": "{{ .principal }}"
            },
            "Action": [
                "kms:GetKeyPolicy",
                "kms:Encrypt",
                "kms:GetPublicKey",
                "kms:TagResource"
            ],
            "Resource": "*"
        }
    ]
}
"##;

#[derive(Gtmpl)]
struct Context {
    principal: String,
    pcr0: String,
    pcr1: String,
    pcr2: String,
}

#[derive(Debug)]
pub struct KeyOutput {
    pub arn: String,
    pub alias: String,
}

#[instrument(level = "debug")]
pub async fn key(client: &Client, principal: &str, eif: &str) -> Result<KeyOutput, Error> {
    let pcrs = get_pcrs(eif).await?;

    let context = Context {
        principal: principal.to_string(),
        pcr0: pcrs.pcr0,
        pcr1: pcrs.pcr1,
        pcr2: pcrs.pcr2,
    };

    let key_policy = template(KEY_POLICY_TEMPLATE, context)?;

    let output = client
        .create_key()
        .policy(key_policy)
        .key_spec(KeySpec::Rsa4096)
        .key_usage(KeyUsageType::EncryptDecrypt)
        .bypass_policy_lockout_safety_check(true)
        .send()
        .await?;

    let metadata = output.key_metadata().unwrap();
    let key_id = metadata.key_id().unwrap();
    let arn = metadata.arn().unwrap();

    let id = Uuid::new_v4();
    let alias = format!("alias/nitrogen-{}", id);

    client
        .create_alias()
        .alias_name(&alias)
        .target_key_id(key_id)
        .send()
        .await?;

    Ok(KeyOutput {
        arn: arn.to_string(),
        alias,
    })
}

// "Measurements": {
//     "HashAlgorithm": "Sha384 { ... }",
//     "PCR0": "6e5f9f840dd17f3ab4deaf1954e65302642ac4ee4365382afa5ec970045d2a3448f222431208494daa1fa59d78b8b3f8",
//     "PCR1": "bcdf05fefccaa8e55bf2c8d6dee9e79bbff31e34bf28a99aa19e6b29c37ee80b214a414b7607236edf26fcb78654e63f",
//     "PCR2": "d8afbe78d624566500651d1abd46c87c0b32c6ae309690dcaa26d87f8069a4828a9a95b4ea5c05f765ae8571728becaa"
//   },

#[derive(Deserialize, Debug)]
struct PCRs {
    #[serde(rename = "PCR0")]
    pcr0: String,

    #[serde(rename = "PCR1")]
    pcr1: String,

    #[serde(rename = "PCR2")]
    pcr2: String,
}

#[derive(Deserialize, Debug)]
struct EIFInfo {
    #[serde(rename = "Measurements")]
    measurements: PCRs,
}

async fn get_pcrs(eif: &str) -> Result<PCRs, Error> {
    let cwd = env::current_dir()?;
    let h = home::home_dir().unwrap_or_default();

    let out = Command::new("docker")
        .args([
            "run",
            "-v",
            &format!("{}/.docker:/root/.docker", h.display()),
            "-v",
            "/var/run/docker.sock:/var/run/docker.sock",
            "-v",
            &format!("{}:/root/build", cwd.to_str().unwrap_or_default()),
            "capeprivacy/eif-builder:latest",
            "describe-eif",
            "--eif-path",
            &format!("/root/build/{}", eif),
        ])
        .output()
        .await?;

    let info: EIFInfo = serde_json::from_slice(&out.stdout)?;

    Ok(info.measurements)
}
