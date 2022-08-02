extern crate dotenv;

use anyhow::{anyhow, Result};
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
use itertools::Itertools;
use serde::Serialize;
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

async fn show_state(region_name: String) -> Result<Vec<ReservedInstancesOffering>> {
    let region = Region::new(region_name.clone());
    let shared_config = aws_config::from_env().region(region).load().await;
    let client = Client::new(&shared_config);
    // return Ok(vec![]);

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
            .await;

        let resp = if let Ok(x) = resp {
            x
        } else {
            return Err(anyhow!("{:?} {:#?}", region_name, resp));
        };

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

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[derive(Serialize)]
struct MarketplaceReservationOffer {
    pub availability_zone: String,
    /// <p>The duration of the Reserved Instance, in seconds.</p>
    pub duration: i64,
    /// <p>The purchase price of the Reserved Instance.</p>
    pub fixed_price: f32,
    /// <p>The instance type on which the Reserved Instance can be used.</p>
    pub instance_type: String,
    /// <p>The Reserved Instance product platform description.</p>
    pub product_description: String,
    /// <p>The ID of the Reserved Instance offering. This is the offering ID used in <code>GetReservedInstancesExchangeQuote</code> to confirm that an exchange can be made.</p>
    pub reserved_instances_offering_id: String,
    /// <p>The usage price of the Reserved Instance, per hour.</p>
    pub usage_price: f32,
    /// <p>The tenancy of the instance.</p>
    pub instance_tenancy: String,
    /// <p>If <code>convertible</code> it can be exchanged for Reserved Instances of the same or higher monetary value, with different configurations. If <code>standard</code>, it is not possible to perform an exchange.</p>
    pub offering_class: String,
    /// <p>The Reserved Instance offering type.</p>
    pub offering_type: String,
    /// <p>The pricing details of the Reserved Instance offering.</p>
    pub pricing_details: String,
    /// <p>The recurring charge tag assigned to the resource.</p>
    pub recurring_charges: String,
    /// <p>Whether the Reserved Instance is applied to instances in a Region or an Availability Zone.</p>
    pub scope: String,

    pub region: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    let regions = [
        "us-east-2",
        "us-east-1",
        "us-west-1",
        "us-west-2",
        // "af-south-1",
        "ap-east-1",
        // "ap-southeast-3",
        "ap-south-1",
        "ap-northeast-3",
        "ap-northeast-2",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-northeast-1",
        "ca-central-1",
        "eu-central-1",
        "eu-west-1",
        "eu-west-2",
        // "eu-south-1",
        "eu-west-3",
        "eu-north-1",
        // "me-south-1",
        "sa-east-1",
    ];
    let regions = ["us-east-1", "us-east-2", "us-west-1", "us-west-2"];
    let regions = [
        "us-east-2",
        "us-east-1",
        "us-west-1",
        "us-west-2",
        "eu-central-1",
        "eu-west-1",
        "eu-west-2",
        // "eu-south-1",
        "eu-west-3",
        "eu-north-1",
    ];
    // let regions = ["us-east-1"];

    // async fn foo(i: u32) -> u32 {
    //     i
    // }
    //
    // let futures = vec![foo(1), foo(2), foo(3)];
    // let res = futures::future::join_all(futures).await;
    // print_type_of(&res);
    // assert_eq!(res, [1, 2, 3]);

    'foo: loop {
        let date = chrono::offset::Local::now().format(DATE_FORMAT).to_string();
        // Start crawler tasks
        let tasks = regions
            .iter()
            .map(|&region| {
                let region_clone = region.to_string();
                let res = tokio::spawn(async { show_state(region_clone).await });
                (region.to_string(), res)
            })
            .collect_vec();

        // Collect results
        let mut reserved = vec![];
        for task in tasks {
            let (region, handle) = task;
            let result = handle.await?;
            if let Ok(mut list) = result {
                for item in list {
                    reserved.push(MarketplaceReservationOffer {
                        availability_zone: item.availability_zone.unwrap_or("".to_string()),
                        duration: item.duration.unwrap_or(-1),
                        fixed_price: item.fixed_price.unwrap_or(-1.0),
                        instance_type: item
                            .instance_type
                            .unwrap_or(InstanceType::Unknown("".to_string()))
                            .as_str()
                            .to_string(),
                        product_description: item
                            .product_description
                            .and_then(|x| Some(x.as_str().to_string()))
                            .unwrap_or("".to_string()),
                        reserved_instances_offering_id: item
                            .reserved_instances_offering_id
                            .and_then(|x| Some(x.as_str().to_string()))
                            .unwrap_or("".to_string()),
                        usage_price: item.usage_price.unwrap_or(-1.0),
                        instance_tenancy: item
                            .instance_tenancy
                            .and_then(|x| Some(x.as_str().to_string()))
                            .unwrap_or("".to_string()),

                        offering_class: item
                            .offering_class
                            .and_then(|x| Some(x.as_str().to_string()))
                            .unwrap_or("".to_string()),

                        offering_type: item
                            .offering_type
                            .and_then(|x| Some(x.as_str().to_string()))
                            .unwrap_or("".to_string()),

                        pricing_details: item
                            .pricing_details
                            .and_then(|x| Some(format!("{:?}", x)))
                            .unwrap_or("".to_string()),
                        recurring_charges: item
                            .recurring_charges
                            .and_then(|x| Some(format!("{:?}", x)))
                            .unwrap_or("".to_string()),
                        scope: item
                            .scope
                            .and_then(|x| Some(format!("{:?}", x)))
                            .unwrap_or("".to_string()),
                        region: region.clone(),
                    });
                }
            } else {
                println!("Err: {}, {:?}", region, result);
                continue 'foo;
            }
        }

        // Write to file
        let json = serde_json::to_string(&reserved)?;
        let file_name = format!("{date}-v2.json");
        upload_file("ec2-scraper", &file_name, &json).await?;

        // let res = futures::future::try_join_all(tasks).await;
        // print_type_of(&res);
        // if let Ok(res2) = res {
        //     println!("{:?}", res2);
        // } else {
        //     println!("Err: {:?}", res);
        // }

        // if let Ok(res) = futures::future::try_join_all(tasks).await {
        //     // let res2 = res.into_iter().flatten().collect::<Vec<_>>();
        //     for r in res {
        //         println!("{:?}", r);
        //         // if let Ok(mut res) = r {
        //         //     reserved.append(&mut res);
        //         // } else {
        //         //     println!("Err: {:?}", r);
        //         //     continue 'foo;
        //         // }
        //     }
        // }
        // let res2 = res.into_iter().flatten().collect::<Vec<_>>();
        // if let Ok(reserved) = res2 {
        //     let content = format!("{:#?}", reserved);
        //     let file_name = format!("{date}-v1.txt");
        //     upload_file("ec2-scraper", &file_name, &content).await?;
        // }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
