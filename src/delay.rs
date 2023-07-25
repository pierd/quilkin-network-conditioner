#[allow(non_snake_case)]
mod proto {
    tonic::include_proto!("delay");
}

use quilkin::filters::prelude::*;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, schemars::JsonSchema)]
pub struct Duration {
    #[serde(default)]
    secs: u64,
    #[serde(default)]
    millis: u64,
    #[serde(default)]
    micros: u64,
    #[serde(default)]
    nanos: u64,
}

impl TryFrom<proto::Duration> for Duration {
    type Error = ConvertProtoConfigError;

    fn try_from(p: proto::Duration) -> Result<Self, Self::Error> {
        Ok(Self {
            secs: p.secs,
            millis: p.millis,
            micros: p.micros,
            nanos: p.nanos,
        })
    }
}

impl From<Duration> for proto::Duration {
    fn from(config: Duration) -> Self {
        Self {
            secs: config.secs,
            millis: config.millis,
            micros: config.micros,
            nanos: config.nanos,
        }
    }
}

impl From<Duration> for tokio::time::Duration {
    fn from(value: Duration) -> Self {
        let Duration {
            secs,
            millis,
            micros,
            nanos,
        } = value;
        Self::from_secs(secs)
            + Self::from_millis(millis)
            + Self::from_micros(micros)
            + Self::from_nanos(nanos)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, schemars::JsonSchema)]
pub struct Config {
    on_read: Option<Duration>,
    on_write: Option<Duration>,
}

impl TryFrom<proto::Delay> for Config {
    type Error = ConvertProtoConfigError;

    fn try_from(p: proto::Delay) -> Result<Self, Self::Error> {
        Ok(Self {
            on_read: p.on_read.map(Duration::try_from).transpose()?,
            on_write: p.on_write.map(Duration::try_from).transpose()?,
        })
    }
}

impl From<Config> for proto::Delay {
    fn from(config: Config) -> Self {
        Self {
            on_read: config.on_read.map(Into::into),
            on_write: config.on_write.map(Into::into),
        }
    }
}

pub struct Delay {
    config: Config,
}

#[async_trait::async_trait]
impl Filter for Delay {
    async fn read(&self, _ctx: &mut ReadContext) -> Result<(), FilterError> {
        if let Some(delay) = self.config.on_read {
            tokio::time::sleep(delay.into()).await;
        }
        Ok(())
    }

    async fn write(&self, _ctx: &mut WriteContext) -> Result<(), FilterError> {
        if let Some(delay) = self.config.on_write {
            tokio::time::sleep(delay.into()).await;
        }
        Ok(())
    }
}

impl StaticFilter for Delay {
    const NAME: &'static str = "delay.v1";
    type Configuration = Config;
    type BinaryConfiguration = proto::Delay;

    fn try_from_config(config: Option<Self::Configuration>) -> Result<Self, CreationError> {
        Ok(Self {
            config: Self::ensure_config_exists(config)?,
        })
    }
}
