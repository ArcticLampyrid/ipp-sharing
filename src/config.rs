use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
};
use tokio::fs;
use uuid::Uuid;

fn default_addr() -> OneOrMany<SocketAddr> {
    OneOrMany::Many(vec![
        SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 631),
        SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 631),
    ])
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl From<OneOrMany<SocketAddr>> for Vec<SocketAddr> {
    fn from(v: OneOrMany<SocketAddr>) -> Vec<SocketAddr> {
        match v {
            OneOrMany::One(v) => vec![v],
            OneOrMany::Many(v) => v,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigRoot {
    pub server: ServerConfig,
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_addr")]
    pub addr: OneOrMany<SocketAddr>,
    pub host: Option<String>,
    pub tls: Option<TlsConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}

fn default_make_and_model() -> String {
    "IppSharing via ippper".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceConfig {
    pub name: String,
    pub info: String,
    pub target: String,
    pub uuid: Uuid,
    pub basepath: String,
    #[serde(default)]
    pub dnssd: bool,
    #[serde(default = "default_make_and_model")]
    pub make_and_model: String,
}

pub async fn read_config(path: impl AsRef<Path>) -> anyhow::Result<ConfigRoot> {
    let content = fs::read_to_string(path).await?;
    Ok(serde_yml::from_str::<ConfigRoot>(content.as_str())?)
}
