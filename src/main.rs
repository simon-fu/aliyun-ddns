
use std::time::Duration;
use anyhow::{Result, Context};
use time::macros::format_description;
use tracing::{info, debug, warn, Instrument, metadata::LevelFilter};

pub mod aliyun_cli;
use aliyun_cli::AliyunCli;

pub mod get_my_ip;
use get_my_ip::get_my_ip;
use tracing_subscriber::EnvFilter;
// use tracing_subscriber::fmt::time;
// use tracing_subscriber::EnvFilter;


#[tokio::main]
async fn main() -> Result<()> {
    const ALIYUN_CLI: &str = "/Users/simon/simon/myhome/mini/aliyun/aliyun";
    const REGION: &str = "cn-hangzhou";
    const DOMAIN: &str = "rtcsdk.com";
    const RR: &str = "simon.home";

    // %m-%d %H:%M:%S%.3f
    let timer = tracing_subscriber::fmt::time::LocalTime::new(
        format_description!("[month]-[day]T[hour]:[minute]:[second]")
    );
    
    tracing_subscriber::fmt::fmt()
    .with_timer(timer)
    .with_env_filter(
        EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
    )
    .init();


    let cli = AliyunCli::new(
        ALIYUN_CLI.into(), 
        REGION.into(),
    );

    // ./aliyun alidns UpdateDomainRecord --region cn-hangzhou --RecordId 831569755440133120 --RR 'simon.home' --Type A --Value '114.249.210.247'
    // cli.update_domain_record_a("831569755440133120", RR, "114.249.210.247").await?;

    let h1 = tokio::spawn(async move {

        let r = run_update(&cli, DOMAIN, RR).await;
        info!("run result [{:?}]", r);

    }.instrument(tracing::info_span!("update")));

    h1.await?;

    Ok(())
}

async fn run_update(cli: &AliyunCli, domain: &str, rr: &str) -> Result<()> {
    let (my_ip, updated) = update_aliyun_ddns(cli, domain, rr).await?;
    if updated {
        info!("update domain record [{}] -> [{}]", rr, my_ip);
    } else {
        info!("exist domain record [{}] = [{}]", rr, my_ip);
    }

    loop {
        let r = update_aliyun_ddns(cli, domain, rr).await;
        match r {
            Ok((my_ip, updated)) => {
                if updated {
                    info!("update domain record [{}] -> [{}]", rr, my_ip);
                }
            },
            Err(e) => {
                warn!("update but [{:?}]", e);
            },
        }

        tokio::time::sleep(Duration::from_millis(60*1000)).await;
    }
}

async fn update_aliyun_ddns(cli: &AliyunCli, domain: &str, rr: &str) -> Result<(String, bool)> {
    let my_ip = get_my_ip().await
    .with_context(||"get_my_ip fail")?;

    debug!("my ip [{}]", my_ip);

    let records = cli.get_domain_records(domain).await
    .with_context(||"get_domain_records fail")?;
    debug!("records {:#?}", records);

    let record = records.iter()
    .find(|x|x.rr == rr)
    .with_context(||format!("Not found RR [{}]", rr))?;

    if record.value != my_ip {
        cli.update_domain_record_a(&record.record_id, &record.rr, &my_ip).await
        .with_context(||"update_domain_record_a fail")?;
        return Ok((my_ip, true))
    } else {
        return Ok((my_ip, false))
    }
}





