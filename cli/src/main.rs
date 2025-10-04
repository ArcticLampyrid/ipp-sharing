use clap::Parser;
use ipp_sharing_core::config::read_config;
use ipp_sharing_core::ipp_sharing;
use log::{error, info};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

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
    ipp_sharing(&config).await?;

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
