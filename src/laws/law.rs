use serde::Deserialize;
use serenity::{async_trait, model::prelude::GuildId, prelude::*};

/// Represents a single legal action, such as creating a channel, role,
/// banning a user, etc.
#[async_trait]
pub trait LegalAction {
    async fn execute(&self, ctx: Context, guild: GuildId)
        -> Result<(), Box<dyn std::error::Error>>;
}

/// Action for creating a channel.
#[derive(Deserialize)]
struct CreateChannelAction {
    name: String,
}

/// Action for creating a role.
#[async_trait]
impl LegalAction for CreateChannelAction {
    async fn execute(
        &self,
        ctx: Context,
        guild: GuildId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        guild
            .create_channel(&ctx, |c| {
                c.name(&self.name);
                c
            })
            .await?;
        Ok(())
    }
}

/// Enum that holds all possible legal actions.
#[derive(Deserialize)]
enum LegalActionEnum {
    CreateChannel(CreateChannelAction),
}

/// Represents a single law, which is a collection of legal actions.
pub struct Law {
    name: String,
    actions: Vec<LegalActionEnum>,
}
