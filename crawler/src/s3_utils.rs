use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use bytes::Bytes;
use tracing::info;

// Uploads a file to S3.
pub async fn upload_file_to_s3(bucket: &str, key: &str, content: &str) -> Result<()> {
    let region = RegionProviderChain::default_provider().or_else("us-east-1");
    let shared_config = aws_config::from_env().region(region).load().await;
    let client: aws_sdk_s3::Client = aws_sdk_s3::Client::new(&shared_config);

    let body = Bytes::copy_from_slice(content.as_bytes());
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.into())
        .send()
        .await?;

    info!("Uploaded {}", key);
    Ok(())
}
