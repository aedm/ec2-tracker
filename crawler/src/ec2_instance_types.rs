use crate::ec2_utils::{make_ec2_client, REGIONS};
use anyhow::{anyhow, Result};

pub struct InstanceType {
    pub name: String,
    pub vcpu: i32,
    pub memory: i32,
    pub storage: i32,
}

pub async fn fetch_instance_type_list() -> Result<Vec<InstanceType>> {
    let result = vec![];
    let client = make_ec2_client(REGIONS[0].to_string()).await?;
    let response = client.describe_instance_type_offerings().send().await?;
    println!("{:#?}", response.instance_type_offerings);
    let list = response
        .instance_type_offerings
        .ok_or(anyhow!("Can't get instance type offerings"))?;
    let instance_type = list[0]
        .instance_type
        .clone()
        .ok_or(anyhow!("Instance type not found"))?;
    println!("{:#?}", instance_type);

    let res2 = client
        .describe_instance_types()
        .instance_types(instance_type)
        .send()
        .await?;
    println!("{:#?}", res2);

    // client.describe_instance_types().instance_types(response.instance_type_offerings[0])
    // for &region in REGIONS {
    //     let client = make_ec2_client(region.to_string()).await?;
    //     let response = client.describe_instance_type_offerings().send().await?;
    //     println!("{:#?}", response.instance_type_offerings);
    // }
    Ok(result)
}
