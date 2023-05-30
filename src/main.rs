
use std::{time::Duration, net::SocketAddr, borrow::Cow};
use anyhow::{Result, Context, bail};
use clap::Parser;
use reqwest::Url;
use time::macros::format_description;
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::{info, debug, warn, Instrument, metadata::LevelFilter};

pub mod aliyun_cli;
use aliyun_cli::AliyunCli;

pub mod get_my_ip;
use get_my_ip::get_my_ip;
use tracing_subscriber::EnvFilter;

// cargo run -- --domain rtcsdk.com --rr simon.home --cli "/Users/simon/simon/myhome/mini/aliyun/aliyun" --ping "udp://39.105.43.146:5000?line=aaa"

#[tokio::main]
async fn main() -> Result<()> {
    let r = run_me().await;
    println!("final: [{:?}]", r);
    r
}

async fn run_me() -> Result<()> {
// const ALIYUN_CLI: &str = "/Users/simon/simon/myhome/mini/aliyun/aliyun";
    // const REGION: &str = "cn-hangzhou";
    // const DOMAIN: &str = "rtcsdk.com";
    // const RR: &str = "simon.home";

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

    info!("runing...");
    
    let args = CmdArgs::parse();

    if let Some(ping) = args.ping.as_ref() {
        kick_ping(ping).await?;
    }

    let cli = AliyunCli::new(
        args.cli.clone(), // ALIYUN_CLI.into(), 
        args.region.clone(), // REGION.into(),
    );

    // ./aliyun alidns UpdateDomainRecord --region cn-hangzhou --RecordId 831569755440133120 --RR 'simon.home' --Type A --Value '114.249.210.247'
    // cli.update_domain_record_a("831569755440133120", RR, "114.249.210.247").await?;

    let h1 = tokio::spawn(async move {

        let r = run_update(&cli, &args.domain, &args.rr).await;
        info!("run result [{:?}]", r);

    }.instrument(tracing::info_span!("update")));


    h1.await?;

    Ok(())
}

async fn kick_ping(ping: &str) -> Result<JoinHandle<()>> {
    let url: Url = Url::parse(ping).with_context(||"invalid ping url")?;
    if url.scheme() != "udp" {
        bail!("only support udp ping")
    }

    let addr = format!(
        "{}:{}", 
        url.host_str().with_context(||"expect ping host")?, 
        url.port().with_context(||"expect ping port")?,
    );

    let target: SocketAddr = addr.parse().with_context(||"invalid ping addr")?;

    let socket = UdpSocket::bind("0.0.0.0:0").await
    .with_context(||"fail to bind udp")?;

    let line = url.query_pairs()
    .find(|x|x.0 == "line")
    .map(|x|x.1)
    .unwrap_or(Cow::Borrowed("hello ddns"));

    let line = format!("{}\r\n", line);

    let h = tokio::spawn(async move {
        loop {
            let r = socket.send_to(line.as_bytes(), target).await;
            if let Err(e) = r {
                warn!("ping fail [{:?}]", e);
            }
            tokio::time::sleep(Duration::from_millis(60*1000)).await;
        }
        
    }.instrument(tracing::info_span!("ping")));
    Ok(h)
}

async fn run_update(cli: &AliyunCli, domain: &str, rr: &str) -> Result<()> {
    update_one(cli, domain, rr, true).await;

    loop {
        update_one(cli, domain, rr, false).await;
        tokio::time::sleep(Duration::from_millis(60*1000)).await;
    }
}

async fn update_one(cli: &AliyunCli, domain: &str, rr: &str, first: bool ) {
    let r = update_aliyun_ddns(cli, domain, rr).await;
    match r {
        Ok((my_ip, updated)) => {
            if updated {
                info!("update domain record [{}] -> [{}]", rr, my_ip);
            } else {
                if first {
                    info!("exist domain record [{}] = [{}]", rr, my_ip);
                }
            }
        },
        Err(e) => {
            warn!("update but [{:?}]", e);
        },
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



#[derive(Parser, Debug)]
#[clap(name = "aliyun-ddns", author, version, about = "update domain record")]
pub struct CmdArgs {

    #[clap(long = "cli", long_help = "aliyun cli path", default_value = "aliyun")]
    pub cli: String,

    #[clap(long = "region", long_help = "aliyun region", default_value="cn-hangzhou")]
    pub region: String,

    #[clap(long = "domain", long_help = "target domain")]
    pub domain: String,

    #[clap(long = "rr", long_help = "for example: www")]
    pub rr: String,


    #[clap(long = "ping", long_help = "target url to ping, for example: udp://127.0.0.1:5000?line=abc")]
    pub ping: Option<String>,

}





