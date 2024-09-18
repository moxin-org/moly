use crate::data::store::Store;
use makepad_widgets::{Actions, Cx, Scope};
use moxin_mae::{MaeAgent, MaeAgentCommand, MaeAgentResponse};
use std::{
    sync::mpsc::{channel, Sender},
    thread::spawn,
};

pub struct Mae {
    /// Send messages here, to translate them from raw `mpsc` based communication
    /// to `Cx::post_action`.
    ///
    /// See `new` implementation for details.
    ///
    /// This is just to be compatible with the current mae backend which requires
    /// to pass a `Receiver`.
    response_sender: Sender<MaeAgentResponse>,

    /// Lazy initialization to be compatible with the store.
    command_sender: Option<Sender<MaeAgentCommand>>,
}

impl Mae {
    /// Creates a new `Mae` instance.
    /// You still need to call `ensure_initialized`.
    pub fn new() -> Self {
        let (tx, rx) = channel();
        spawn(move || {
            // Break and dispose the thread if this instance is dropped.
            while let Ok(response) = rx.recv() {
                Cx::post_action(response)
            }
        });

        Self {
            response_sender: tx,
            command_sender: None,
        }
    }

    /// Compatibility trick to steal the already initialized sender from the store.
    pub fn ensure_initialized(&mut self, scope: &mut Scope) {
        if self.command_sender.is_none() {
            let store = scope.data.get::<Store>().expect("store not found");
            self.command_sender = store.mae_backend.command_sender.clone().into();
        }
    }

    /// Unconditional accessor to the sender to overcome lazy initialization.
    fn sender(&self) -> &Sender<MaeAgentCommand> {
        self.command_sender.as_ref().unwrap()
    }

    /// Shortcut to send a message to `self.sender()`.
    fn send(&self, command: MaeAgentCommand) {
        self.sender()
            .send(command)
            .expect("can't communicate with mae's thread");
    }

    /// Sends prompt to an agent.
    pub fn send_prompt(&self, agent: MaeAgent, prompt: String) {
        self.send(MaeAgentCommand::SendTask(
            prompt,
            agent,
            self.response_sender.clone(),
        ));
    }
}

/// Handle global responses from mae.
///
/// Note: Could be improved to handle responses for a specific widget.
pub fn responses(actions: &Actions) -> impl Iterator<Item = &MaeAgentResponse> {
    actions
        .iter()
        .filter_map(move |action| action.downcast_ref())
}
