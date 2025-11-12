use crate::protocol::*;

/// Represents complex (mostly async) operations that may cause multiple mutations
/// over time.
#[derive(Clone, Debug, PartialEq)]
pub enum ChatTask {
    /// Causes the whole list of messages to be sent to the currently selected bot
    /// (from state.bot_id) and starts the streaming response work in the background.
    Send,
    /// Calls the given MCP tools. If a bot is specified, successful tool calls
    /// will be processed by that bot.
    Execute(Vec<ToolCall>, Option<BotId>),
    /// Interrupts the streaming started by `Send`.
    Stop,
    /// Should be triggered to start fetching async data (e.g. bots).
    ///
    /// Eventually, the state will contain the list of bots or errors as messages.
    Load,
}
