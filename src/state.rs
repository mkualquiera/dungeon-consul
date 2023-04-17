use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{GuildId, MessageId, UserId};

use crate::laws::law::Law;

#[derive(Serialize, Deserialize)]
pub struct LawProposal {
    pub law: Law,
    pub voting_message_id: MessageId,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ConsulState {
    pub law_proposals: HashMap<GuildId, Vec<LawProposal>>,
}
