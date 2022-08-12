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
