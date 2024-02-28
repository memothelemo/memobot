use derive_more::Display;
use error_stack::{Result, ResultExt};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug)]
pub struct ApiConfig {
    address: IpAddr,
    port: u16,
}

#[derive(Debug, Display)]
#[display(fmt = "Could not load memobot API configuration")]
pub struct ApiConfigLoadError;
impl error_stack::Context for ApiConfigLoadError {}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiConfigLoadError> {
        let address = memobot_env_vars::var_parsed("MEMOBOT_API_ADDRESS")
            .change_context(ApiConfigLoadError)?
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));

        let port = memobot_env_vars::var_parsed("MEMOBOT_API_PORT")
            .change_context(ApiConfigLoadError)?
            .unwrap_or(6500);

        Ok(Self { address, port })
    }
}

impl ApiConfig {
    #[must_use]
    pub fn address(&self) -> IpAddr {
        self.address
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
}
