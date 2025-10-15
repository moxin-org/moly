use super::{state::*, task::*};
use crate::protocol::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Controls if remaining callbacks and default behavior should be executed.
pub enum ChatControl {
    Continue,
    Stop,
}

/// Allows to hook between dispatched events of any kind.
///
/// It's the fundamental building block for extending [`ChatController`] beyond
/// its default behavior and integrating it with other technologies.
pub trait ChatControllerPlugin: Send {
    /// Called when new state is available.
    ///
    /// Usually used to bind the controller to some view component/widget/element
    /// in your framework of choice.
    fn after_state_change(&mut self, _state: &ChatState, _mutations: &[ChatStateMutation]) {}

    fn on_task(&mut self, _event: &ChatTask) -> ChatControl {
        ChatControl::Continue
    }

    fn before_state_change(&mut self, _state: &ChatState, _mutations: &[ChatStateMutation]) {}

    fn on_upgrade(&mut self, upgrade: Upgrade, _bot_id: &BotId) -> Option<Upgrade> {
        Some(upgrade)
    }

    // attachment handling?
}

/// Unique identifier for a registered plugin. Can be used to unregister it later.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChatControllerPluginRegistrationId(u64);

impl ChatControllerPluginRegistrationId {
    pub(super) fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}
