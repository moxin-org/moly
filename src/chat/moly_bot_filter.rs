use crate::data::providers::ProviderBot;
use moly_kit::protocol::Bot;
use moly_kit::widgets::model_selector_list::BotFilter;
use moly_kit::BotId;
use std::collections::HashMap;

/// Moly-specific bot filter that filters by enabled status
pub struct MolyBotFilter {
    /// Map from BotId to ProviderBot (to check enabled status)
    available_bots: HashMap<BotId, ProviderBot>,
}

impl MolyBotFilter {
    pub fn new(available_bots: HashMap<BotId, ProviderBot>) -> Self {
        Self { available_bots }
    }
}

impl BotFilter for MolyBotFilter {
    fn should_show(&self, bot: &Bot) -> bool {
        // Only show enabled bots in the model selector
        // Disabled bots are still kept in ChatController.state.bots for message rendering
        if let Some(provider_bot) = self.available_bots.get(&bot.id) {
            provider_bot.enabled
        } else {
            // If bot not in available_bots, show it (defensive: let unknown bots through)
            true
        }
    }
}
