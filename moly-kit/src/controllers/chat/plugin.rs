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
///
/// Note: While a method of a plugin is running, the controller is locked and you
/// don't get mutable access to it so if you need to do something to the controller
/// you may need to schedule it for later.
pub trait ChatControllerPlugin: Send {
    /// Called when a new FULL state is READY to be presented.
    ///
    /// This is called after ALL mutations given to [`dispatch_mutations`](super::ChatController::dispatch_mutations)
    /// have been applied.
    ///
    /// So you get a final view of the state. You also get the list of all mutations
    /// that were applied together, although, since they are already applied, they
    /// are mostly useful for reacting to specific kinds of mutations.
    ///
    /// This is always called after [`on_state_mutation`](ChatControllerPlugin::on_state_mutation)
    /// has been called for each individual mutation.
    ///
    /// Usually used to bind the controller to some view component/widget/element
    /// in your framework of choice.
    ///
    /// > Note: State is the main focus of this method, so it is given as the first parameter.
    fn on_state_ready(&mut self, _state: &ChatState, _mutations: &[ChatStateMutation]) {}

    /// Called right before a task is going to be executed.
    ///
    /// You can cancel it to handle the behavior yourself.
    fn on_task(&mut self, _task: &ChatTask) -> ChatControl {
        ChatControl::Continue
    }

    /// Called for every INDIVIDUAL mutation passed to [`dispatch_mutations`](super::ChatController::dispatch_mutations).
    ///
    /// The received state is the state without the mutation applied. This state
    /// may be modified further by next batched mutations so it's not safe to read
    /// it for logic based on the final state. For this purpose, wait for [`on_state_ready`](ChatControllerPlugin::on_state_ready)
    /// to be called. The state received here is useful for data replication and for
    /// reconstructing and anayzing the effects of the mutation before they happen.
    ///
    /// > Note: Mutations are the focus of this method, so they are given as the first parameter.
    fn on_state_mutation(&mut self, _mutation: &ChatStateMutation, _state: &ChatState) {}

    // TODO: Remove this very specific method later.
    fn on_upgrade(&mut self, upgrade: Upgrade, _bot_id: &BotId) -> Option<Upgrade> {
        Some(upgrade)
    }
}

/// Unique identifier for a registered plugin. Can be used to unregister it later.
// TODO: Consider identifying plugins just by their type for simplicity on most use cases.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChatControllerPluginRegistrationId(u64);

impl ChatControllerPluginRegistrationId {
    pub(super) fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}
