use std::env;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;
use derive_builder::Builder;
use merge::Merge;
use tracing::{debug, info};

#[derive(Builder)]
#[builder(derive(serde::Deserialize, merge::Merge))]
pub struct SharpConfig {
    #[builder(default = "IpAddr::from([127, 0, 0, 1])")]
    pub address: IpAddr,
    pub port: u16,
    pub upstream: SocketAddr,
    #[builder(default = "vec![\"/favicon.ico\".to_string(), \"/robots.txt\".to_string(), \"/sitemap.xml\".to_string()]")]
    pub exceptions: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("failed to parse config file: {0}")]
    ParseToml(#[from] toml::de::Error),
    #[error("failed to parse config file: {0}")]
    Builder(#[from] SharpConfigBuilderError),
    #[error("failed to parse address: {0}")]
    ParseAddress(#[from] AddrParseError),
    #[error("failed to parse port as int: {0}")]
    ParsePort(#[from] ParseIntError),
}

impl SharpConfigBuilder {
    fn from_env() -> Result<Self, ConfigError> {
        info!("parsing config from environment variables");
        let address = match env::var("SHARP_ADDRESS").map(|s| s.parse()) {
            Ok(p) => Some(p?),
            Err(_) => None,
        };
        let port = match env::var("SHARP_PORT").map(|s| u16::from_str(&s)) {
            Ok(p) => Some(p?),
            Err(_) => None,
        };
        let upstream = match env::var("SHARP_UPSTREAM").map(|s| s.parse()) {
            Ok(p) => Some(p?),
            Err(_) => None,
        };
        Ok(Self {
            address,
            port,
            upstream,
            exceptions: None,
        })
    }
}

pub async fn read_config<P: AsRef<Path>>(path: P) -> Result<SharpConfig, ConfigError> {
    let mut config_builder = SharpConfigBuilder::from_env()?;
    match config_builder.build() {
        Ok(config) => Ok(config),
        Err(e) => {
            debug!("missing environment variables ({e})");
            info!("parsing config file at `{}`", path.as_ref().display());
            let str = tokio::fs::read_to_string(path).await?;
            let file_config: SharpConfigBuilder = toml::from_str(&str)?;
            config_builder.merge(file_config);
            Ok(config_builder.build()?)
        }
    }
}
