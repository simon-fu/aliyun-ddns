

use anyhow::{Result, bail, Context};
use serde::Deserialize;
use serde_derive::Deserialize;
use tracing::debug;

pub struct AliyunCli {
    cli_path: String,
    region: String,
}

impl AliyunCli {
    pub fn new(cli_path: String, region: String, ) -> Self {
        Self { 
            cli_path,
            region,
        }
    }

    pub async fn update_domain_record_a(&self, record_id: &str, rr: &str, value: &str) -> Result<()> {
        // ./aliyun alidns UpdateDomainRecord --region cn-hangzhou --RecordId 831569755440133120 --RR 'simon.home' --Type A --Value '127.0.0.1'

        let mut cmd = tokio::process::Command::new(self.cli_path.as_str());
        cmd.args(&[
            "alidns", 
            "UpdateDomainRecord", 
            "--region", self.region.as_str(), 
            "--RecordId", record_id,
            "--RR", rr,
            "--Type", "A", 
            "--Value", value,
        ]);

        let _rsp: serde_json::Value = exec_cmd_json(cmd).await?;

        Ok(())
    }

    pub async fn get_domain_records(&self, domain: &str) -> Result<Vec<DomainRecord>> {
        // aliyun alidns DescribeDomainRecords --region cn-hangzhou --DomainName 'rtcsdk.com'

        let mut cmd = tokio::process::Command::new(self.cli_path.as_str());
        cmd.args(&[
            "alidns", 
            "DescribeDomainRecords", 
            "--region", self.region.as_str(), 
            "--DomainName", domain,
        ]);


        let output = cmd.output().await
        .with_context(||format!("fail to exec cmd [{}]", self.cli_path))?;
    
        if !output.status.success() {
            bail!("cmd output fail {:?}", output)
        }

        let std_output = String::from_utf8(output.stdout)
        .with_context(||"stdout not string")?;
        debug!("std_output [{}]", std_output);

        let rsp: GetDomainRecordsResponse = serde_json::from_str(&std_output)
        .with_context(||format!("invalid json [{}]", std_output))?;

        Ok(rsp.domain_records.record)
    }
}

async fn exec_cmd_json<T>(mut cmd: tokio::process::Command) -> Result<T> 
    where
        for<'a> T: Deserialize<'a>,
{
    let output = cmd.output().await
    .with_context(||format!("fail to exec cmd [{:?}]", cmd.as_std()))?;

    if !output.status.success() {
        bail!("cmd output fail {:?}", output)
    }

    let std_output = String::from_utf8(output.stdout)
    .with_context(||"stdout not string")?;
    debug!("std_output [{}]", std_output);

    let rsp: T = serde_json::from_str(&std_output)
    .with_context(||format!("invalid json [{}]", std_output))?;

    Ok(rsp)
}

#[derive(Debug, Deserialize)]
pub struct GetDomainRecordsResponse {
    #[serde(rename="DomainRecords")]
    domain_records: DomainRecords,
}

#[derive(Debug, Deserialize)]
pub struct DomainRecords {
    #[serde(rename="Record")]
    record: Vec<DomainRecord>,
}


#[derive(Debug, Deserialize)]
pub struct DomainRecord {
    
    #[serde(rename="DomainName")]
    pub domain_name: String, // "rtcsdk.com"

    #[serde(rename="RecordId")]
    pub record_id: String, // "831569755440133120"

    #[serde(rename="Type")]
    pub rtype: String, // "A"

    #[serde(rename="Value")]
    pub value: String, // "127.0.0.1"

    #[serde(rename="RR")]
    pub rr: String, // "simon.home"

    #[serde(rename="TTL")]
    pub ttl: i64, // 600


    #[serde(rename="Line")]
    pub line: String, // "default"

    #[serde(rename="Locked")]
    pub locked: bool, // false

    #[serde(rename="Status")]
    pub status: String, // "ENABLE"

    #[serde(rename="Weight")]
    pub weight: Option<i64>, // 1
}
