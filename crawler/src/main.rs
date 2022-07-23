extern crate dotenv;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::{Client, Error, Region, PKG_VERSION};
use dotenv::dotenv;

// Lists the state of an instance.
// snippet-start:[ec2.rust.describe-instances]
async fn show_state(client: &Client, ids: Option<Vec<String>>) -> Result<(), Error> {
    let resp = client.describe_instance_type_offerings().send().await?;

    println!("offerings: {resp:#?}");

    // for reservation in resp.instance_type_offerings.unwrap_or_default() {
    //     for instance in reservation.instances().unwrap_or_default() {
    //         println!("Instance ID: {}", instance.instance_id().unwrap());
    //         println!(
    //             "State:       {:?}",
    //             instance.state().unwrap().name().unwrap()
    //         );
    //         println!();
    //     }
    // }

    Ok(())
}
// snippet-end:[ec2.rust.describe-instances]

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenv();

    println!("Hello, world!");

    let instance_id = "m5.xlarge";
    let region = Region::new("us-east-1");

    // let region_provider = RegionProviderChain::first_try(region)
    //     .or_default_provider()
    //     .or_else(Region::new("us-west-2"));
    // println!();

    println!("EC2 client version: {}", PKG_VERSION);
    // println!(
    //     "Region:             {}",
    //     region_provider.region().await.unwrap().as_ref()
    // );

    println!("Instance ID:        {:?}", instance_id);

    println!();

    let shared_config = aws_config::from_env().region(region).load().await;
    let client = Client::new(&shared_config);

    let mut ids: Vec<String> = Vec::new();
    let id_opt: std::option::Option<std::vec::Vec<std::string::String>>;

    // match instance_id {
    //     None => id_opt = None,
    //     Some(i) => {
    ids.push(instance_id.to_string());
    id_opt = Some(ids);
    //     }
    // }

    show_state(&client, id_opt).await
}
