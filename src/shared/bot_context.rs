//! Copy paste of the legacy [`BotContext`] from Moly Kit.

use moly_kit::{
    controllers::chat::{ChatController, ChatStateMutation, Status},
    mcp::McpManagerClient,
    protocol::*,
    utils::vec::VecMutation,
};
use std::sync::{Arc, Mutex};

struct InnerBotContext {
    client: Box<dyn BotClient>,
    bots: Vec<Bot>,
    tool_manager: Option<McpManagerClient>,
    /// Status tracked for compatibility with [`ChatController`].
    status: Status,
}

/// A sharable wrapper around a [BotClient] that holds loadeed bots and provides
/// synchronous APIs to access them, mainly from the UI.
///
/// Passed down through widgets from this crate.
///
/// Separate chat widgets can share the same [BotContext] to avoid loading the same
/// bots multiple times.
pub struct BotContext(Arc<Mutex<InnerBotContext>>);

impl Clone for BotContext {
    fn clone(&self) -> Self {
        BotContext(self.0.clone())
    }
}

impl PartialEq for BotContext {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl BotContext {
    /// Differenciates [BotContext]s.
    ///
    /// Two [BotContext]s are equal and share the same underlying data if they have
    /// the same id.
    pub fn id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    /// Fetches the bots and keeps them to be read synchronously later.
    ///
    /// It errors with whatever the underlying client errors with.
    pub fn load(&mut self) -> BoxPlatformSendFuture<ClientResult<()>> {
        let future = async move {
            self.0.lock().unwrap().status = Status::Working;

            let result = self.client().bots().await;
            let (new_bots, errors) = result.into_value_and_errors();

            if let Some(new_bots) = new_bots {
                self.0.lock().unwrap().bots = new_bots;
            }

            if errors.is_empty() {
                self.0.lock().unwrap().status = Status::Success;
                ClientResult::new_ok(())
            } else {
                self.0.lock().unwrap().status = Status::Error;
                ClientResult::new_err(errors)
            }
        };

        Box::pin(future)
    }
    pub fn client(&self) -> Box<dyn BotClient> {
        self.0.lock().unwrap().client.clone_box()
    }

    pub fn bots(&self) -> Vec<Bot> {
        self.0.lock().unwrap().bots.clone()
    }

    pub fn get_bot(&self, id: &BotId) -> Option<Bot> {
        self.bots().into_iter().find(|bot| bot.id == *id)
    }

    pub fn tool_manager(&self) -> Option<McpManagerClient> {
        self.0.lock().unwrap().tool_manager.clone()
    }

    pub fn set_tool_manager(&mut self, tool_manager: McpManagerClient) {
        self.0.lock().unwrap().tool_manager = Some(tool_manager);
    }

    pub fn replace_tool_manager(&mut self, tool_manager: McpManagerClient) {
        self.0.lock().unwrap().tool_manager = Some(tool_manager);
    }

    /// Copies the data and status from this context into the controller.
    ///
    /// This is a glue function while migrating away from [`BotContext`].
    pub fn synchronize_to(&self, chat_controller: &mut ChatController) {
        chat_controller.set_tool_manager(self.tool_manager());
        chat_controller.set_client(Some(self.client()));
        chat_controller.dispatch_mutation(VecMutation::Set(self.bots().clone()));
        chat_controller.dispatch_mutation(ChatStateMutation::SetLoadStatus(
            self.0.lock().unwrap().status,
        ));
    }
}

impl<T: BotClient + 'static> From<T> for BotContext {
    fn from(client: T) -> Self {
        BotContext(Arc::new(Mutex::new(InnerBotContext {
            client: Box::new(client),
            bots: Vec::new(),
            tool_manager: None,
            status: Status::Idle,
        })))
    }
}
