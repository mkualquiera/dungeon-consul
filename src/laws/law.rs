use serde::{Deserialize, Serialize};
use serenity::{async_trait, model::prelude::GuildId, prelude::*};

/// Represents a single legal action, such as creating a channel, role,
/// banning a user, etc.
#[async_trait]
pub trait LegalActionT {
    async fn execute(&self, ctx: Context, guild: GuildId)
        -> Result<(), Box<dyn std::error::Error>>;

    fn natural_language(&self) -> String;
}

/// Action for creating a channel.
#[derive(Deserialize, Serialize, Debug)]
struct CreateChannelAction {
    name: String,
}

#[async_trait]
impl LegalActionT for CreateChannelAction {
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

    fn natural_language(&self) -> String {
        format!("Create a channel named {}", self.name)
    }
}

/// Enum that holds all possible legal actions.
#[derive(Deserialize, Serialize, Debug)]
pub enum LegalAction {
    CreateChannel(CreateChannelAction),
}

/// Represents a single law, which is a collection of legal actions.
#[derive(Deserialize, Serialize, Debug)]
pub struct Law {
    name: String,
    actions: Vec<LegalAction>,
}

impl Law {
    pub fn new(name: String, actions: Vec<LegalAction>) -> Self {
        Self { name, actions }
    }

    pub async fn execute(
        &self,
        ctx: Context,
        guild: GuildId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for action in &self.actions {
            match action {
                LegalAction::CreateChannel(action) => {
                    action.execute(ctx.clone(), guild).await?;
                }
            }
        }
        Ok(())
    }

    pub fn natural_language(&self) -> String {
        let mut result = format!("The law {} says to:\n", self.name);
        for action in &self.actions {
            match action {
                LegalAction::CreateChannel(action) => {
                    result.push_str(&action.natural_language());
                }
            }
            result.push('\n');
        }
        result.pop();
        result
    }
}
