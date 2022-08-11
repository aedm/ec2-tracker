mod ec2_utils;
mod s3_utils;

extern crate dotenv;

use crate::ec2_utils::{fetch_offerings, MarketplaceReservationOffer};
use crate::s3_utils::upload_file_to_s3;
use anyhow::Result;
use dotenv::dotenv;
use std::time::Duration;
use tracing::{error, info};

const DATE_FORMAT: &str = "%Y%m%d-%H%M%S";

async fn run_cycle() -> Result<()> {
    let date = chrono::offset::Local::now().format(DATE_FORMAT).to_string();
    info!("Starting crawler cycle {}", date);

    let offerings = fetch_offerings().await?;
    let json = serde_json::to_string(&offerings)?;
    let file_name = format!("db/{date}-v3.json");
    upload_file_to_s3("ec2-scraper", &file_name, &json).await?;
    upload_file_to_s3("ec2-scraper", "latest.txt", &file_name).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    tracing_subscriber::fmt::init();

    loop {
        if let Err(e) = run_cycle().await {
            error!("{}", e);
        }
    }
}
