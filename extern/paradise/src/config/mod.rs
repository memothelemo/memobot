use derive_more::Display;
use error_stack::{Result, ResultExt};
use memobot_kernel::Sensitive;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker},
    Id,
};

use memobot_env_vars::{required_var, required_var_parsed, var_parsed};

#[derive(Debug)]
pub struct Config {
    id: Id<GuildMarker>,
    alert_channel_id: Id<ChannelMarker>,
    alert_role_id: Id<RoleMarker>,
    sanctuary_addr: String,
    sanctuary_port: u16,
    // token to get access from the api
    token: Sensitive<String>,
}

#[derive(Debug, Display)]
#[display(fmt = "Could not load Paradise configuration")]
pub struct ConfigLoadError;
impl error_stack::Context for ConfigLoadError {}

impl Config {
    pub fn from_env() -> Result<Option<Self>, ConfigLoadError> {
        let id = var_parsed("MEMOBOT_PARADISE_GUILD_ID").change_context(ConfigLoadError);
        let Some(id) = id? else {
            return Ok(None);
        };

        let alert_channel_id = required_var_parsed("MEMOBOT_PARADISE_ALERT_CHANNEL_ID")
            .change_context(ConfigLoadError)?;

        let alert_role_id = required_var_parsed("MEMOBOT_PARADISE_ALERT_ROLE_ID")
            .change_context(ConfigLoadError)?;

        let sanctuary_addr =
            required_var("MEMOBOT_PARADISE_SANCTUARY_ADDR").change_context(ConfigLoadError)?;

        let sanctuary_port = var_parsed("MEMOBOT_PARADISE_SANCTUARY_PORT")
            .change_context(ConfigLoadError)?
            .unwrap_or(25565);

        let token =
            required_var_parsed("MEMOBOT_PARADISE_API_TOKEN").change_context(ConfigLoadError)?;

        Ok(Some(Self {
            id,
            alert_channel_id,
            alert_role_id,
            sanctuary_addr,
            sanctuary_port,
            token: Sensitive::new(token),
        }))
    }
}

impl Config {
    #[must_use]
    pub fn id(&self) -> Id<GuildMarker> {
        self.id
    }

    #[must_use]
    pub fn alert_channel_id(&self) -> Id<ChannelMarker> {
        self.alert_channel_id
    }

    #[must_use]
    pub fn alert_role_id(&self) -> Id<RoleMarker> {
        self.alert_role_id
    }

    #[must_use]
    pub fn sanctuary_addr(&self) -> &str {
        &self.sanctuary_addr
    }

    #[must_use]
    pub fn sanctuary_port(&self) -> u16 {
        self.sanctuary_port
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }
}
