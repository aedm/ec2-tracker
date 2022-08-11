extern crate dotenv;

use anyhow::{anyhow, Result};
use aws_sdk_ec2::error::DescribeReservedInstancesOfferingsError;
use aws_sdk_ec2::model::OfferingClassType::{Convertible, Standard};
use aws_sdk_ec2::model::OfferingTypeValues::{AllUpfront, NoUpfront, PartialUpfront};
use aws_sdk_ec2::model::{
    filter, Filter, OfferingClassType, OfferingTypeValues, ReservedInstancesOffering,
};
use aws_sdk_ec2::output::DescribeReservedInstancesOfferingsOutput;
use aws_sdk_ec2::types::SdkError;
use aws_sdk_ec2::{Client, Region};
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error, info, warn};

static REGIONS: &[&str] = &[
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

async fn get_single_offering(
    client: &Client,
    filter: &Filter,
    offering_class: &OfferingClassType,
    offering_type: &OfferingTypeValues,
    next_token: &Option<String>,
) -> Result<
    DescribeReservedInstancesOfferingsOutput,
    SdkError<DescribeReservedInstancesOfferingsError>,
> {
    let mut retry_count = 3;
    loop {
        let response = client
            .describe_reserved_instances_offerings()
            .filters(filter.clone())
            .include_marketplace(true)
            .offering_class(offering_class.clone())
            .offering_type(offering_type.clone())
            .set_next_token(next_token.clone())
            .send()
            .await;
        if response.is_ok() {
            return response;
        }
        if format!("{:?}", response).contains("RequestLimitExceeded") {
            tokio::time::sleep(Duration::from_millis(100)).await;
            debug!("RequestLimitExceeded, retrying.");
            continue;
        }
        debug!("Failed to get offerings: {:?}", response);
        retry_count -= 1;
        if retry_count == 0 {
            error!("Out of retries, giving up. Last error: {:?}", response);
            return response;
        }
    }
}

async fn make_ec2_client(region: String) -> Result<Client> {
    let shared_config = aws_config::from_env()
        .region(Region::new(region))
        .load()
        .await;
    let client = Client::new(&shared_config);
    Ok(client)
}

async fn fetch_marketplace_offers_once(
    region_name: String,
    offering_class: OfferingClassType,
    offering_type: OfferingTypeValues,
) -> Result<Vec<ReservedInstancesOffering>> {
    let client = make_ec2_client(region_name.clone()).await?;
    let filter = filter::Builder::default()
        .set_name(Some("marketplace".to_string()))
        .set_values(Some(vec!["true".to_string()]))
        .build();

    let mut next_token = None;
    let mut res = vec![];
    for count in 1.. {
        let resp = get_single_offering(
            &client,
            &filter,
            &offering_class,
            &offering_type,
            &next_token,
        )
        .await?;

        if let Some(mut offerings) = resp.reserved_instances_offerings {
            res.append(&mut offerings);
        }

        next_token = resp.next_token;
        if next_token.is_none() {
            break;
        }
        debug!("Iteration {count}");
    }

    debug!("Finished, result length: {}", res.len());
    Ok(res)
}

#[tracing::instrument]
async fn fetch_marketplace_offers(
    region_name: String,
    offering_class: OfferingClassType,
    offering_type: OfferingTypeValues,
) -> Result<Vec<ReservedInstancesOffering>> {
    let mut result = Ok(vec![]);
    for i in 0..3 {
        result = fetch_marketplace_offers_once(
            region_name.clone(),
            offering_class.clone(),
            offering_type.clone(),
        )
        .await;
        if result.is_ok() {
            return result;
        } else {
            warn!("Iteration {} failed: {:?}", i, result);
        }
    }
    result
}

#[derive(Serialize)]
pub struct MarketplaceReservationOffer {
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

pub async fn fetch_offerings() -> Result<Vec<MarketplaceReservationOffer>> {
    info!("Start fetching offers");

    let offering_classes = [Standard, Convertible];
    let offering_types = [AllUpfront, PartialUpfront, NoUpfront];
    // let offering_classes = [Standard];
    // let offering_types = [AllUpfront];

    // Start crawler tasks
    let mut tasks = vec![];
    for &region in REGIONS {
        for offering_class in &offering_classes {
            for offering_type in &offering_types {
                let region_clone = region.to_string();
                let offering_class_clone = offering_class.clone();
                let offering_type_clone = offering_type.clone();
                let res = tokio::spawn(async {
                    fetch_marketplace_offers(
                        region_clone,
                        offering_class_clone,
                        offering_type_clone,
                    )
                    .await
                });
                tasks.push((region.to_string(), res));
            }
        }
    }

    // Collect results
    let mut offers = vec![];
    let mut has_error = false;
    for task in tasks {
        let (region, handle) = task;
        let handle_result = handle.await;
        if let Ok(result) = handle_result {
            if let Ok(list) = result {
                add_offerings(&mut offers, &list, &region);
                info!("Added {} offerings from {}", list.len(), region);
            } else {
                error!("{}, {:?}", region, result);
                has_error = true;
            }
        } else {
            error!("Can't join task. {}, {:?}", region, handle_result);
            has_error = true;
        }
    }

    if has_error {
        return Err(anyhow!("Can't fetch offerings"));
    }

    Ok(offers)
}

struct InstanceType {
    pub name: String,
    pub vcpu: i32,
    pub memory: i32,
    pub storage: i32,
}

pub async fn fetch_instance_type_list() -> Result<Vec<InstanceType>> {
    let mut result = vec![];
    for &region in REGIONS {
        let client = make_ec2_client(region.to_string()).await?;
        let response = client.describe_instance_type_offerings().send().await?;
        response.instance_type_offerings
    }
    Ok(result)
}
