use derive_builder::Builder;
use merge::Merge;
use std::{
    env,
    fmt::{Debug, Display, Formatter},
    future::Future,
    net::{AddrParseError, IpAddr, SocketAddr},
    num::ParseIntError,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct CustomCss(String);

impl Deref for CustomCss {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CustomCss {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl CustomCss {
    fn from_path_option<P: AsRef<Path>>(path: &Option<P>) -> Result<Option<Self>, std::io::Error> {
        match path {
            None => Ok(None),
            Some(p) => Ok(Some(Self(std::fs::read_to_string(p)?))),
        }
    }
}

impl From<std::io::Error> for SharpConfigBuilderError {
    fn from(e: std::io::Error) -> Self {
        Self::ValidationError(format!("failed to read custom css file: {e}"))
    }
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(serde::Deserialize, merge::Merge))]
pub struct SharpConfig {
    #[builder(default = "IpAddr::from([127, 0, 0, 1])")]
    pub address: IpAddr,
    pub port: u16,
    pub upstream: SocketAddr,
    pub database_url: String,
    #[builder(
        default = "vec![\"/favicon.ico\".to_string(), \"/robots.txt\".to_string(), \"/sitemap.xml\".to_string()]"
    )]
    pub exceptions: Vec<String>,
    #[builder(field(
        type = "Option<PathBuf>",
        build = "CustomCss::from_path_option(&self.custom_css)?"
    ))]
    pub custom_css: Option<CustomCss>,
    #[builder(default = "String::from(\"/\")")]
    pub redirect_url: String,
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
    pub fn from_env() -> Result<Self, ConfigError> {
        info!("parsing config from environment variables");
        let address = match env::var("SHARP_ADDRESS").map(|s| s.parse()) {
            Ok(a) => Some(a?),
            Err(_) => None,
        };
        let port = match env::var("SHARP_PORT").map(|s| u16::from_str(&s)) {
            Ok(p) => Some(p?),
            Err(_) => None,
        };
        let upstream = match env::var("SHARP_UPSTREAM").map(|s| s.parse()) {
            Ok(u) => Some(u?),
            Err(_) => None,
        };
        Ok(Self {
            address,
            port,
            upstream,
            database_url: env::var("SHARP_DATABASE_URL").ok(),
            exceptions: None,
            custom_css: None,
            redirect_url: None,
        })
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        info!("parsing config file at `{}`", path.as_ref().display());
        let str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&str)?)
    }
}

pub async fn read_run_config<P: AsRef<Path>>(path: P) -> Result<SharpConfig, ConfigError> {
    read_config_with_fallback(
        || async { SharpConfigBuilder::from_env() },
        || SharpConfigBuilder::from_file(path),
    )
    .await
}

pub async fn read_config<F, Fut>(main: F) -> Result<SharpConfig, ConfigError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SharpConfigBuilder, ConfigError>>,
{
    Ok(main().await?.build()?)
}

pub async fn read_config_with_fallback<FM, MFut, FF, FFut>(
    main: FM,
    fallback: FF,
) -> Result<SharpConfig, ConfigError>
where
    FM: FnOnce() -> MFut,
    MFut: Future<Output = Result<SharpConfigBuilder, ConfigError>>,
    FF: FnOnce() -> FFut,
    FFut: Future<Output = Result<SharpConfigBuilder, ConfigError>>,
{
    let mut config_builder = main().await?;
    match config_builder.build() {
        Ok(config) => Ok(config),
        Err(e) => {
            debug!("missing environment variables ({e})");
            config_builder.merge(fallback().await?);
            Ok(config_builder.build()?)
        }
    }
}
