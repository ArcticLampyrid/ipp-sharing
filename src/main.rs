use clap::Parser;
use config::{read_config, OneOrMany};
use gethostname::gethostname;
use hyper::service::service_fn;
use ipp_service::MyIppService;
use ippper::handler::handle_ipp_via_http;
use ippper::server::{serve_adaptive_https, serve_http, tls_config_from_reader};
use log::{error, info};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;
mod attr;
mod config;
mod dnssd;
mod handler;
mod ipp_service;
mod print_options;
mod raster;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Opts {
    #[arg(short, long)]
    config: Option<String>,
}

fn default_config_file_path() -> anyhow::Result<PathBuf> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push("config.yaml");
    Ok(path)
}

async fn app_main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let config_path = match opts.config {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => default_config_file_path()
            .map_err(|e| anyhow::anyhow!("failed to get default config file path: {}", e))?,
    };
    let config = read_config(config_path.as_path()).await.map_err(|e| {
        anyhow::anyhow!(
            "failed to read config file {}: {}",
            config_path.display(),
            e
        )
    })?;

    info!("Config File: {}", config_path.display());

    if config.devices.is_empty() {
        return Err(anyhow::anyhow!("no printer device found in config file"));
    }

    let mut ipp_services = Vec::new();
    for device_config in config.devices {
        match MyIppService::new(&config.server, &device_config) {
            Ok(ipp_service) => {
                info!(
                    "Sharing {} as {} (UUID={})",
                    device_config.target, device_config.name, device_config.uuid
                );
                ipp_services.push(Arc::new(ipp_service))
            }
            Err(e) => {
                error!(
                    "Failed to create ipp service for {}: {}",
                    device_config.name, e
                );
            }
        }
    }

    let http_service = service_fn(move |req| {
        let ipp_services = ipp_services.clone();
        async move {
            let path = req.uri().path();
            if req.method() == hyper::Method::GET && path == "/" {
                return Ok(hyper::Response::builder()
                    .header("Content-Type", "text/plain")
                    .body(ippper::body::Body::from("Hello, IppSharing!"))
                    .unwrap());
            }
            for ipp_service in ipp_services {
                if ipp_service.matches(path) {
                    return handle_ipp_via_http(req, ipp_service.inner.as_ref()).await;
                }
            }
            Ok(hyper::Response::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(ippper::body::Body::from("404 Not Found"))
                .unwrap())
        }
    });

    let port = match &config.server.addr {
        OneOrMany::One(addr) => addr.port(),
        OneOrMany::Many(addrs) => addrs.first().map_or(631, |addr| addr.port()),
    };
    let host = config
        .server
        .host
        .clone()
        .unwrap_or_else(|| format!("{}.local:{}", gethostname().to_string_lossy(), port));
    info!("Host: {}", host);
    let addrs = Vec::<SocketAddr>::from(config.server.addr.clone());
    info!(
        "IppSharing is running at {}",
        addrs
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(tls) = &config.server.tls {
        info!("TLS Enabled, Cert: {}, Key: {}", tls.cert, tls.key);
        let cert = fs::read(tls.cert.as_str()).await?;
        let key = fs::read(tls.key.as_str()).await?;
        let tls_config = Arc::new(tls_config_from_reader(cert.as_slice(), key.as_slice())?);
        let all_tasks = addrs
            .into_iter()
            .map(|addr| serve_adaptive_https(addr, http_service.clone(), tls_config.clone()));
        futures::future::try_join_all(all_tasks).await?;
    } else {
        info!("TLS Disabled");
        let all_tasks = addrs
            .into_iter()
            .map(|addr| serve_http(addr, http_service.clone()));
        futures::future::try_join_all(all_tasks).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    flexi_logger::init();
    if let Err(e) = app_main().await {
        error!("Unhandled error: {}", e);
        std::process::exit(100);
    }
}
