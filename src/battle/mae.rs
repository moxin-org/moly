use crate::data::store::Store;
use makepad_widgets::{Actions, Cx, Scope};
use moxin_mae::{should_be_fake, MaeAgent, MaeAgentCommand, MaeAgentResponse};
use std::{
    sync::mpsc::{channel, Sender},
    thread::spawn,
};

/// Interface to MAE that takes adventage of the new `Cx::post_action` and provides
/// isolated message handling.
pub struct Mae {
    /// Identify this mae instance to handle responses in isolation.
    id: usize,

    /// Send messages here, to translate them from raw `mpsc` based communication
    /// to `Cx::post_action` with added isolation.
    ///
    /// See `new` implementation for details.
    ///
    /// This is just to be compatible with the current mae backend which requires
    /// to pass a `Receiver` and doesn't provide a way to filter responses based
    /// on the sender.
    response_sender: Sender<MaeAgentResponse>,

    /// Lazy initialization to be compatible with the store.
    ///
    /// See `ensure_initialized` implementation for details.
    command_sender: Option<Sender<MaeAgentCommand>>,
}

impl Mae {
    /// Creates a new `Mae` instance.
    /// You still need to call `ensure_initialized`.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        let (tx, rx) = channel();
        spawn(move || {
            // Break and dispose the thread if this instance is dropped.
            while let Ok(response) = rx.recv() {
                // Simulate some delay if using a fake backend.
                // This is handled here and not in the backend implementation so
                // the delay affects only this instance of the interface and not others.
                if should_be_fake() {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Cx::post_action((id, response))
            }
        });

        Self {
            id,
            response_sender: tx,
            command_sender: None,
        }
    }

    /// Compatibility trick to steal the already configured sender from the store.
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

    /// Sends a prompt to an agent.
    pub fn send_prompt(&self, agent: MaeAgent, prompt: String) {
        self.send(MaeAgentCommand::SendTask(
            prompt,
            agent,
            self.response_sender.clone(),
        ));
    }

    /// Handle responses sent from this specific mae instance.
    pub fn responses<'a>(
        &'a self,
        actions: &'a Actions,
    ) -> impl Iterator<Item = &'a MaeAgentResponse> {
        actions
            .iter()
            .filter_map(move |action| action.downcast_ref::<(usize, MaeAgentResponse)>())
            .filter(|(id, _)| *id == self.id)
            .map(|(_, response)| response)
    }
}
