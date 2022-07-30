extern crate dotenv;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::model::CapacityReservationInstancePlatform::LinuxUnix;
use aws_sdk_ec2::model::{
    filter, CapacityReservationInstancePlatform, Filter, InstanceType, OfferingClassType,
    ReservedInstancesOffering, RiProductDescription, Tenancy,
};
use aws_sdk_ec2::{Client, Region};
use aws_sdk_s3::types::ByteStream;
use bytes::Bytes;
use dotenv::dotenv;
use std::time::Duration;
use std::time::Instant;

const DATE_FORMAT: &str = "%Y%m%d-%H%M%S";

// Uploads a file to S3.
async fn upload_file(bucket: &str, key: &str, content: &str) -> Result<()> {
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

    println!("Uploaded {}", key);
    Ok(())
}

async fn show_state(region_name: &str) -> Result<Vec<ReservedInstancesOffering>> {
    let region = Region::new(region_name.to_string());
    let shared_config = aws_config::from_env().region(region).load().await;
    let client = Client::new(&shared_config);

    let filter = filter::Builder::default()
        .set_name(Some("marketplace".to_string()))
        .set_values(Some(vec!["true".to_string()]))
        .build();

    let mut next_token = None;
    let mut res = vec![];
    for count in 1.. {
        let resp = client
            .describe_reserved_instances_offerings()
            .filters(filter.clone())
            .include_marketplace(true)
            // .instance_type(InstanceType::C54xlarge)
            // .instance_tenancy(Tenancy::Default)
            // .product_description(CapacityReservationInstancePlatform::LinuxUnix.into())
            // .offering_class(OfferingClassType::Standard)
            // .instance_tenancy(Tenancy::Default)
            .set_next_token(next_token)
            .send()
            .await?;

        if let Some(mut offerings) = resp.reserved_instances_offerings {
            if offerings.len() > 0 {
                res.append(&mut offerings);
            }
        }

        next_token = resp.next_token;
        if next_token.is_none() {
            break;
        }
        println!("{:4}, {:?}", count, chrono::offset::Local::now());
    }

    println!("{:#?}", res);
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    // upload_file("ec2-scraper", "test.txt", "test").await?;

    // let now = Instant::now();
    // show_state("us-east-2").await;
    // println!("Elapsed {}", now.elapsed().as_secs());

    // return Ok(());

    let regions = ["us-east-1", "us-east-2", "us-west-1", "us-west-2"];
    let regions = ["us-east-1"];

    'foo: loop {
        let date = chrono::offset::Local::now().format(DATE_FORMAT).to_string();
        let mut reserved = vec![];
        for region in regions {
            println!("{}", region);
            let r = show_state(region).await;
            if let Ok(mut res) = r {
                reserved.append(&mut res);
            } else {
                continue 'foo;
            }
        }
        let content = format!("{:#?}", reserved);
        let file_name = format!("{date}-v1.txt");
        upload_file("ec2-scraper", &file_name, &content).await?;
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
