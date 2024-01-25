use std::net::{IpAddr, Ipv4Addr};

use anyhow::Result;
use serde_derive::Deserialize;


pub async fn get_my_ip() -> Result<String> {
    get_ip_jsonip().await
}


async fn get_ip_jsonip() -> Result<String> {
    const URL: &str = "http://jsonip.com";

    let bind_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let client = reqwest::Client::builder()
        .local_address(bind_addr) 
        .build()?;

    let body = client
        .get(URL)
        .send()
        .await?
        .text()
        .await?;


    // let body = reqwest::get(URL)
    // .await?
    // .text()
    // .await?;

    let rsp: IpResponse = serde_json::from_str(body.as_str())?;
    check_ip(&rsp.ip)?;

    Ok(rsp.ip)
}

fn check_ip(ip: &str) -> Result<()> {
    let _ip: IpAddr = ip.parse()?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct IpResponse {
    ip: String,
}


// async fn get_ip_3322() -> Result<String> {
//     const URL: &str = "http://www.3322.org/dyndns/getip";
//     let body = reqwest::get(URL)
//     .await?
//     .text()
//     .await?;

//     Ok(body)
// }
