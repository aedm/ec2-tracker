extern crate dotenv;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::model::{filter, Filter};
use aws_sdk_ec2::{Client, Region};
use aws_sdk_s3::types::ByteStream;
use bytes::Bytes;
use dotenv::dotenv;

// Uploads a file to S3.
async fn upload_file(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    content: &str,
) -> Result<()> {
    let body = Bytes::copy_from_slice(content.as_bytes());
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.into())
        .send()
        .await?;
    Ok(())
}

async fn show_state(client: &Client) -> Result<()> {
    let filter = filter::Builder::default()
        .set_name(Some("marketplace".to_string()))
        .set_values(Some(vec!["true".to_string()]))
        .build();

    let resp = client
        .describe_reserved_instances_offerings()
        .filters(filter.clone())
        .include_marketplace(true)
        .send()
        .await?;

    println!("Offerings: {:?}", resp.reserved_instances_offerings);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    let region = Region::new("eu-central-1");

    let shared_config = aws_config::from_env().region(region).load().await;

    println!("Region: {:#?}", shared_config.region());

    let client = Client::new(&shared_config);

    show_state(&client).await
}
