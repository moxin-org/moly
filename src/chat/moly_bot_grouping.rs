use crate::data::providers::ProviderBot;
use moly_kit::BotId;
use moly_kit::protocol::{Bot, Picture};
use moly_kit::widgets::model_selector_grouping::{BotGrouping, GroupInfo};
use std::collections::HashMap;

/// Moly-specific bot grouping that uses friendly provider names and icons
///
/// This maps provider IDs to human-readable names and provider icons.
pub struct MolyBotGrouping {
    /// Map from BotId to ProviderBot (to get provider_id for each bot)
    available_bots: HashMap<BotId, ProviderBot>,
    /// Map from provider ID to (friendly_name, icon)
    provider_info: HashMap<String, (String, Option<Picture>)>,
}

impl MolyBotGrouping {
    pub fn new(available_bots: HashMap<BotId, ProviderBot>) -> Self {
        Self {
            available_bots,
            provider_info: HashMap::new(),
        }
    }

    /// Add a provider mapping
    pub fn add_provider(&mut self, id: String, name: String, icon: Option<Picture>) {
        self.provider_info.insert(id, (name, icon));
    }
}

impl BotGrouping for MolyBotGrouping {
    fn get_group_info(&self, bot: &Bot) -> GroupInfo {
        // Look up the bot in available_bots to get its provider_id
        if let Some(provider_bot) = self.available_bots.get(&bot.id) {
            let provider_id = &provider_bot.provider_id;

            // Get provider info (name and icon) from provider_info
            if let Some((name, icon)) = self.provider_info.get(provider_id) {
                return GroupInfo {
                    id: provider_id.clone(),
                    label: name.clone(),
                    icon: icon.clone(),
                };
            }

            // Provider not in provider_info (maybe disabled), use provider_id as label
            return GroupInfo {
                id: provider_id.clone(),
                label: provider_id.clone(),
                icon: Some(bot.avatar.clone()),
            };
        }

        // Fallback: bot not found in available_bots, use bot's provider URL
        let bot_provider_url = bot.id.provider();
        GroupInfo {
            id: bot_provider_url.to_string(),
            label: bot_provider_url.to_string(),
            icon: Some(bot.avatar.clone()),
        }
    }
}
