use anyhow::Result;

use aws_sdk_ec2::{Client, Region};

pub static REGIONS: &[&str] = &[
    "us-east-2",
    "us-east-1",
    "us-west-1",
    "us-west-2",
    "eu-central-1",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-north-1",
];
// static REGIONS: &[&str] = &["us-east-2"];

pub async fn make_ec2_client(region: String) -> Result<Client> {
    let shared_config = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;
    let client = Client::new(&shared_config);
    Ok(client)
}
