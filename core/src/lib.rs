use crate::config::ConfigRoot;
use config::OneOrMany;
use gethostname::gethostname;
use hyper::service::service_fn;
use ipp_service::MyIppService;
use ippper::handler::handle_ipp_via_http;
use ippper::server::{serve_adaptive_https, serve_http, tls_config_from_reader};
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs;
mod attr;
pub mod config;
mod dnssd;
mod handler;
mod ipp_service;
mod print_options;
mod raster;

pub async fn ipp_sharing(config: &ConfigRoot) -> anyhow::Result<()> {
    if config.devices.is_empty() {
        return Err(anyhow::anyhow!("no printer device found in config file"));
    }

    let mut ipp_services = Vec::new();
    for device_config in config.devices.iter() {
        match MyIppService::new(&config.server, device_config) {
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
