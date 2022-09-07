use crate::ec2_utils::{make_ec2_client, REGIONS};
use anyhow::{anyhow, Context, Result};
use aws_sdk_ec2::model::InstanceTypeInfo;
use aws_sdk_ec2::output::DescribeInstanceTypesOutput;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, serde::Deserialize)]
pub struct InstanceType {
    pub name: String,
    pub v_cpu_count: i32,
    pub memory: i64,
}

fn make_instance_type_map(instance_types: Vec<InstanceType>) -> HashMap<String, InstanceType> {
    let mut map = HashMap::new();
    for instance_type in instance_types {
        map.insert(instance_type.name.clone(), instance_type);
    }
    map
}

fn instance_type_from_info(result: &DescribeInstanceTypesOutput) -> Option<InstanceType> {
    let info = result.instance_types.as_ref()?.first()?;
    Some(InstanceType {
        name: info.instance_type.as_ref()?.as_str().to_string(),
        v_cpu_count: info.v_cpu_info.as_ref()?.default_v_cpus?,
        memory: info.memory_info.as_ref()?.size_in_mi_b?,
    })
}

pub async fn fetch_instance_type_list() -> Result<HashMap<String, InstanceType>> {
    let mut result = vec![];
    let client = make_ec2_client(REGIONS[0].to_string()).await?;
    let response = client.describe_instance_type_offerings().send().await?;
    let list = response
        .instance_type_offerings
        .ok_or(anyhow!("Can't get instance type offerings"))?;
    info!("Got {} instance type offerings", list.len());

    let mut c = 0;
    for item in list {
        info!("{c}");
        c += 1;
        let instance_type = item
            .instance_type
            .clone()
            .ok_or(anyhow!("Instance type not found"))?;
        let res2 = client
            .describe_instance_types()
            // .instance_types(instance_type)
            .send()
            .await?;
        info!("len {:?}", res2.instance_types.as_ref().unwrap().len());
        if let Some(instance_type) = instance_type_from_info(&res2) {
            result.push(instance_type);
        } else {
            println!("Can't get instance type info for {:?}", res2);
        }
    }
    // println!("{:#?}", res2);

    // client.describe_instance_types().instance_types(response.instance_type_offerings[0])
    // for &region in REGIONS {
    //     let client = make_ec2_client(region.to_string()).await?;
    //     let response = client.describe_instance_type_offerings().send().await?;
    //     println!("{:#?}", response.instance_type_offerings);
    // }

    Ok(make_instance_type_map(result))
}
