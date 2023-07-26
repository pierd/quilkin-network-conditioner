#[allow(non_snake_case)]
mod proto {
    tonic::include_proto!("delay");
}

use quilkin::filters::prelude::*;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

// sync sleep if the duration is less than 2ms
const MAX_SYNC_DURATION: std::time::Duration = std::time::Duration::from_millis(2);

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

struct Sleeper {
    sync: bool,
    duration: Option<std::time::Duration>,
}

impl Sleeper {
    async fn sleep(&self) {
        if let Some(duration) = self.duration {
            if self.sync {
                std::thread::sleep(duration);
            } else {
                tokio::time::sleep(duration).await;
            }
        }
    }
}

impl From<Option<Duration>> for Sleeper {
    fn from(duration: Option<Duration>) -> Self {
        let duration: Option<std::time::Duration> = duration.map(Into::into);
        Self {
            sync: duration.map(|d| d <= MAX_SYNC_DURATION).unwrap_or(false),
            duration,
        }
    }
}

pub struct Delay {
    on_read: Sleeper,
    on_write: Sleeper,
}

#[async_trait::async_trait]
impl Filter for Delay {
    async fn read(&self, _ctx: &mut ReadContext) -> Result<(), FilterError> {
        self.on_read.sleep().await;
        Ok(())
    }

    async fn write(&self, _ctx: &mut WriteContext) -> Result<(), FilterError> {
        self.on_write.sleep().await;
        Ok(())
    }
}

impl StaticFilter for Delay {
    const NAME: &'static str = "delay";
    type Configuration = Config;
    type BinaryConfiguration = proto::Delay;

    fn try_from_config(config: Option<Self::Configuration>) -> Result<Self, CreationError> {
        let config = Self::ensure_config_exists(config)?;
        Ok(Self {
            on_read: config.on_read.into(),
            on_write: config.on_write.into(),
        })
    }
}
