use anyhow::Result;
use memobot_env_vars::{required_var, required_var_parsed, var_parsed};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker},
    Id,
};

use crate::util::Sensitive;

#[derive(Debug)]
pub struct ParadiseConfig {
    id: Id<GuildMarker>,
    alert_channel_id: Id<ChannelMarker>,
    alert_role_id: Id<RoleMarker>,
    sanctuary_addr: Sensitive<String>,
    sanctuary_port: Sensitive<u16>,
}

impl ParadiseConfig {
    pub fn from_env() -> Result<Option<Self>> {
        let Some(id) = var_parsed("ROBOT_PARADISE_GUILD_ID")? else {
            return Ok(None);
        };

        let alert_channel_id = required_var_parsed("ROBOT_PARADISE_ALERT_CHANNEL_ID")?;
        let alert_role_id = required_var_parsed("ROBOT_PARADISE_ALERT_ROLE_ID")?;

        let sanctuary_addr = required_var("ROBOT_PARADISE_SANCTUARY_ADDR")?;
        let sanctuary_port = var_parsed("ROBOT_PARADISE_SANCTUARY_PORT")?.unwrap_or(25565);

        Ok(Some(Self {
            id,
            alert_channel_id,
            alert_role_id,
            sanctuary_addr: Sensitive::new(sanctuary_addr),
            sanctuary_port: Sensitive::new(sanctuary_port),
        }))
    }
}

impl ParadiseConfig {
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
        self.sanctuary_port.into_inner()
    }
}
