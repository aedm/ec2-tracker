extern crate dotenv;

use anyhow::{anyhow, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::model::CapacityReservationInstancePlatform::LinuxUnix;
use aws_sdk_ec2::model::OfferingClassType::{Convertible, Standard};
use aws_sdk_ec2::model::OfferingTypeValues::{AllUpfront, NoUpfront, PartialUpfront};
use aws_sdk_ec2::model::{
    filter, CapacityReservationInstancePlatform, Filter, InstanceType, OfferingClassType,
    OfferingTypeValues, ReservedInstancesOffering, RiProductDescription, Tenancy,
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
// static REGIONS: &[&str] = &[
//     "us-east-2",
//     "us-east-1",
//     "us-west-1",
//     "us-west-2",
//     "eu-central-1",
//     "eu-west-1",
//     "eu-west-2",
//     "eu-west-3",
//     "eu-north-1",
// ];
static REGIONS: &[&str] = &["us-east-2"];

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

async fn show_state(
    region_name: String,
    offering_class: OfferingClassType,
    offering_type: OfferingTypeValues,
) -> Result<Vec<ReservedInstancesOffering>> {
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
            .instance_type(InstanceType::C54xlarge)
            // .product_description(CapacityReservationInstancePlatform::LinuxUnix.into())
            .offering_class(offering_class.clone())
            // .instance_tenancy(Tenancy::Default)
            .offering_type(offering_type.clone())
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
    pub id: String,
    pub region: String,
    pub count: i32,
    pub instance_type: String,
    pub price: f64,
    pub recurring_charge: f64,
    pub duration: i64,
    pub fixed_price: f32,

    pub availability_zone: String,
    pub product_description: String,
    pub usage_price: f32,
    pub instance_tenancy: String,
    pub offering_class: String,
    pub offering_type: String,
    pub scope: String,
}

// Convert option to string
fn option_to_string<T: AsRef<str>>(opt: &Option<T>) -> String {
    match opt {
        Some(x) => x.as_ref().to_string(),
        None => "".to_string(),
    }
}

impl MarketplaceReservationOffer {
    fn from(
        item: &ReservedInstancesOffering,
        region: &str,
        price: f64,
        recurring_charge: f64,
        count: i32,
    ) -> Self {
        MarketplaceReservationOffer {
            price,
            recurring_charge,
            region: region.to_string(),
            count,
            availability_zone: option_to_string(&item.availability_zone),
            duration: item.duration.unwrap_or(-1),
            fixed_price: item.fixed_price.unwrap_or(-1.0),
            instance_type: option_to_string(&item.instance_type),
            product_description: item
                .product_description
                .as_ref()
                .and_then(|x| Some(x.as_str().to_string()))
                .unwrap_or_default(),
            id: item
                .reserved_instances_offering_id
                .clone()
                .unwrap_or_default(),
            usage_price: item.usage_price.unwrap_or(-1.0),
            instance_tenancy: option_to_string(&item.instance_tenancy),
            offering_class: option_to_string(&item.offering_class),
            offering_type: option_to_string(&item.offering_type),
            scope: option_to_string(&item.scope),
        }
    }
}

fn add_offerings(
    reserved: &mut Vec<MarketplaceReservationOffer>,
    list: &[ReservedInstancesOffering],
    region: &str,
) {
    for item in list {
        // AWS API is full of Options for some reason.
        if let Some(pricing_details) = &item.pricing_details {
            if let Some(charges) = &item.recurring_charges {
                if pricing_details.len() != 1 || charges.len() != 1 {
                    continue;
                }
                for pricing_detail in pricing_details {
                    if let Some(price) = pricing_detail.price {
                        if let Some(count) = pricing_detail.count {
                            for charge in charges {
                                if let Some(recurring_charge) = charge.amount {
                                    let offer = MarketplaceReservationOffer::from(
                                        &item,
                                        region,
                                        price,
                                        recurring_charge,
                                        count,
                                    );
                                    reserved.push(offer);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    let offering_classes = [Standard, Convertible];
    let offering_types = [AllUpfront, PartialUpfront, NoUpfront];
    let offering_classes = [Standard];
    let offering_types = [AllUpfront];

    'foo: loop {
        let date = chrono::offset::Local::now().format(DATE_FORMAT).to_string();

        // Start crawler tasks
        let mut tasks = vec![];
        for &region in REGIONS {
            for offering_class in &offering_classes {
                for offering_type in &offering_types {
                    let region_clone = region.to_string();
                    let offering_class_clone = offering_class.clone();
                    let offering_type_clone = offering_type.clone();
                    let res = tokio::spawn(async {
                        show_state(region_clone, offering_class_clone, offering_type_clone).await
                    });
                    tasks.push((region.to_string(), res));
                }
            }
        }

        // let tasks = REGIONS
        //     .iter()
        //     .map(|&region| {
        //         let region_clone = region.to_string();
        //         let res = tokio::spawn(async { show_state(region_clone).await });
        //         (region.to_string(), res)
        //     })
        //     .collect_vec();

        // Collect results
        let mut reserved = vec![];
        for task in tasks {
            let (region, handle) = task;
            let result = handle.await?;

            if let Ok(mut list) = result {
                add_offerings(&mut reserved, &list, &region);
            } else {
                println!("Err: {}, {:?}", region, result);
                continue 'foo;
            }
        }

        // Write to file
        let json = serde_json::to_string(&reserved)?;
        let file_name = format!("{date}-v2.json");
        upload_file("ec2-scraper", &file_name, &json).await?;

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
