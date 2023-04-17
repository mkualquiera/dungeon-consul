mod laws {
    pub mod law;
}

mod database;
mod state;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Reaction;
use serenity::prelude::*;
use std::env;

use lazy_static::lazy_static;
use regex::Regex;

use crate::laws::law::Law;

pub struct ConsulHandler {
    database: database::Database<state::ConsulState>,
}

impl ConsulHandler {
    fn new() -> Self {
        Self {
            database: database::Database::new("consul".into()),
        }
    }
    async fn handle_proposal(&self, ctx: Context, msg: &Message, lawcode: &str) {
        // Try to deserialize the law code
        let law = serde_yaml::from_str::<Law>(lawcode);
        let law = match law {
            Ok(law) => law,
            Err(e) => {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, format!("Error parsing law: {e}"))
                    .await
                {
                    println!("Error sending message: {why:?}");
                }
                return;
            }
        };

        // Get the law-proposals channel
        let guild_id = msg.guild_id.unwrap();
        let guild = guild_id.to_partial_guild(&ctx.http).await.unwrap();
        let channels = guild.channels(&ctx.http).await.unwrap();
        let channel = channels
            .iter()
            .find(|c| c.1.name == "law-proposals")
            .unwrap()
            .1;

        // Send the law to the law-proposals channel
        let message = channel
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&law.name);
                    e.description(&law.natural_language());
                    e
                });
                m
            })
            .await
            .unwrap();

        // Create database entry
        let mut state = self.database.lock();

        let guild_id = msg.guild_id.unwrap();

        // Add the law proposal to the database
        state
            .law_proposals
            .entry(guild_id)
            .or_insert_with(Vec::new)
            .push(state::LawProposal {
                law,
                voting_message_id: message.id,
            });
    }
}

#[async_trait]
impl EventHandler for ConsulHandler {
    /// Handles a law proposal

    // Set a ConsulHandler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event ConsulHandlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Message received: {:?}", msg.content);
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
            return;
        }
        lazy_static! {
            static ref PROPOSE_RE: Regex = Regex::new(r"!propose ```(yaml)?([^`]+)```").unwrap();
        }
        if let Some(caps) = PROPOSE_RE.captures(&msg.content) {
            let lawcode = caps.get(2).unwrap().as_str();
            self.handle_proposal(ctx, &msg, lawcode).await;
        }
    }

    // Handle the reaction add event
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        // Get the message that was reacted to
        let message = reaction.message(&ctx.http).await.unwrap();

        // Get the guild that the message was sent in
        let guild_id = reaction.guild_id.unwrap();

        fn get_law(
            handler: &ConsulHandler,
            guild_id: serenity::model::id::GuildId,
            message_id: serenity::model::id::MessageId,
        ) -> Option<Law> {
            // Get the state
            let mut state = handler.database.lock();

            // Get the law proposals for the guild
            let proposals = state.law_proposals.get_mut(&guild_id).unwrap();

            // Find the proposal that the message is for
            let proposal = proposals
                .iter_mut()
                .find(|p| p.voting_message_id == message_id);

            // If the message is not a proposal, return
            proposal.as_ref()?;

            // Get the proposal
            let proposal = proposal.unwrap();

            Some(proposal.law.clone())
        }

        // Get the law
        let law = get_law(self, guild_id, message.id).unwrap();

        // Handle the proposal voting
        law.execute(ctx, guild_id, self)
            .await
            .expect("Error executing law");
    }

    // Set a ConsulHandler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(ConsulHandler::new())
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
