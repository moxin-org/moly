use crate::protocol::{Bot, Picture};
use std::cmp::Ordering;

/// Information about a group of bots (e.g., provider)
#[derive(Clone, Debug)]
pub struct GroupInfo {
    /// Unique identifier for this group (used for sorting/deduplication)
    pub id: String,
    /// Display label for the group header
    pub label: String,
    /// Optional icon to display next to the label
    pub icon: Option<Picture>,
}

/// Trait for customizing how bots are grouped in the model selector
///
/// The default implementation groups bots by their provider (extracted from BotId),
/// but applications can implement this trait to provide custom grouping logic,
/// display names, icons, etc.
pub trait BotGrouping: Send {
    /// Returns grouping information for a given bot
    fn get_group_info(&self, bot: &Bot) -> GroupInfo;

    /// Compare two group IDs for sorting (default: alphabetical)
    fn compare_groups(&self, a: &str, b: &str) -> Ordering {
        a.cmp(b)
    }
}

/// Default grouping implementation that groups bots by their provider
///
/// Extracts the provider information from BotId.provider() and uses it
/// as both the group ID and label. Uses the bot's avatar as the group icon.
pub struct DefaultBotGrouping;

impl BotGrouping for DefaultBotGrouping {
    fn get_group_info(&self, bot: &Bot) -> GroupInfo {
        let provider = bot.id.provider();
        GroupInfo {
            id: provider.to_string(),
            label: provider.to_string(),
            icon: Some(bot.avatar.clone()),
        }
    }
}
