use crate::{display_name_from_namespaced, protocol::*};

/// Represents a generic status in which an operation can be.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    Idle,
    Working,
    Error,
    Success,
}

impl Status {
    pub fn is_idle(&self) -> bool {
        matches!(self, Status::Idle)
    }

    pub fn is_working(&self) -> bool {
        matches!(self, Status::Working)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Status::Error)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Status::Success)
    }
}

/// State of the chat that you should reflect in your view component/widget/element.
// TODO: Makes sense? #[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChatState {
    /// The chat history sent as context to LLMs.
    pub messages: Vec<Message>,
    /// Indicates that the LLM is still streaming the response ("writing").
    pub is_streaming: bool,
    /// The bots that were loaded from the configured client.
    pub bots: Vec<Bot>,
    pub load_status: Status,
}

impl ChatState {
    pub fn get_bot(&self, bot_id: &BotId) -> Option<&Bot> {
        self.bots.iter().find(|b| &b.id == bot_id)
    }

    pub fn approve_tool_calls(&mut self, index: usize) {
        self.messages[index].update_content(|content| {
            for tool_call in &mut content.tool_calls {
                tool_call.permission_status = ToolCallPermissionStatus::Approved;
            }
        });
    }

    pub fn deny_tool_calls(&mut self, index: usize) {
        self.messages[index].update_content(|content| {
            for tool_call in &mut content.tool_calls {
                tool_call.permission_status = ToolCallPermissionStatus::Denied;
            }
        });

        // Create synthetic tool results indicating denial to maintain conversation flow
        let tool_results: Vec<ToolResult> = self.messages[index]
            .content
            .tool_calls
            .iter()
            .map(|tc| {
                let display_name = display_name_from_namespaced(&tc.name);
                ToolResult {
                    tool_call_id: tc.id.clone(),
                    content: format!(
                        "Tool execution was denied by the user. Tool '{}' was not executed.",
                        display_name
                    ),
                    is_error: true,
                }
            })
            .collect();

        // Add tool result message with denial results
        self.messages.push(Message {
            from: EntityId::Tool,
            content: MessageContent {
                text: "🚫 Tool execution was denied by the user.".to_string(),
                tool_results,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

pub enum ListMutation<T: Clone> {
    /// Adds NEW elements to the list.
    ///
    /// When applied, the length of the list will increase by the number of items inserted.
    Insert(usize, Vec<T>),
    /// Updates an EXISTING element in the list.
    ///
    /// When applied, the length of the list remains UNCHANGED.
    // WARNING: Do NOT change this to be able to insert new entries. `Update` is
    // assumed to leave the length of the list intact. There is a bug somewhere else
    // if an `Update` is being received to a non-existing index.
    Update(usize, T),
    /// Removes a range of elements from the list.
    ///
    /// When applied, the length of the list will decrease by `count`.
    Remove(usize, usize),
}

impl<T: Clone> ListMutation<T> {
    fn apply(&self, list: &mut Vec<T>) {
        match self {
            ListMutation::Insert(index, items) => {
                list.splice(*index..*index, items.clone());
            }
            ListMutation::Remove(index, count) => {
                list.drain(*index..(*index + *count));
            }
            ListMutation::Update(index, item) => {
                if let Some(elem) = list.get_mut(*index) {
                    *elem = item.clone();
                }
            }
        }
    }
}

pub(crate) enum ListMutatorMutation<T: Clone> {
    Base(ListMutation<T>),
    Extend(Vec<T>),
    Update(usize, Box<dyn FnOnce(&mut T)>),
    Set(Vec<T>),
}

impl<T: Clone> ListMutatorMutation<T> {
    pub(crate) fn into_list_mutations(self, list: &[T]) -> Vec<ListMutation<T>> {
        match self {
            ListMutatorMutation::Base(m) => vec![m],
            ListMutatorMutation::Extend(items) => {
                let index = list.len();
                vec![ListMutation::Insert(index, items)]
            }
            ListMutatorMutation::Update(index, f) => {
                let mut item = list[index].clone();
                f(&mut item);
                vec![ListMutation::Update(index, item)]
            }
            ListMutatorMutation::Set(items) => vec![
                ListMutation::Remove(0, list.len()),
                ListMutation::Insert(0, items),
            ],
        }
    }
}

pub struct ListMutator<T: Clone> {
    mutations: Vec<ListMutatorMutation<T>>,
}

impl<T: Clone> ListMutator<T> {
    pub(crate) fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn insert_many(&mut self, index: usize, items: Vec<T>) {
        self.mutations
            .push(ListMutatorMutation::Base(ListMutation::Insert(
                index, items,
            )));
    }

    pub fn insert(&mut self, index: usize, item: T) {
        self.insert_many(index, vec![item]);
    }

    pub fn extend(&mut self, items: Vec<T>) {
        self.mutations.push(ListMutatorMutation::Extend(items));
    }

    pub fn push(&mut self, item: T) {
        self.extend(vec![item]);
    }

    pub fn remove_range(&mut self, index: usize, count: usize) {
        self.mutations
            .push(ListMutatorMutation::Base(ListMutation::Remove(
                index, count,
            )));
    }

    pub fn remove(&mut self, index: usize) {
        self.remove_range(index, 1);
    }

    pub fn update(&mut self, index: usize, item: T) {
        self.mutations
            .push(ListMutatorMutation::Base(ListMutation::Update(index, item)));
    }

    pub fn update_with<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut T) + 'static,
    {
        self.mutations
            .push(ListMutatorMutation::Update(index, Box::new(f)));
    }

    pub fn set(&mut self, items: Vec<T>) {
        self.mutations.push(ListMutatorMutation::Set(items));
    }

    pub(crate) fn finish(self) -> Vec<ListMutatorMutation<T>> {
        self.mutations
    }
}

pub enum ChatStateMutation {
    SetIsStreaming(bool),
    SetLoadStatus(Status),
    MutateMessages(Vec<ListMutation<Message>>),
    MutateBots(Vec<ListMutation<Bot>>),
}

impl ChatStateMutation {
    pub fn apply(&self, state: &mut ChatState) {
        match self {
            ChatStateMutation::SetIsStreaming(is_streaming) => {
                state.is_streaming = *is_streaming;
            }
            ChatStateMutation::SetLoadStatus(status) => {
                state.load_status = status.clone();
            }
            ChatStateMutation::MutateMessages(mutations) => {
                for mutation in mutations {
                    mutation.apply(&mut state.messages);
                }
            }
            ChatStateMutation::MutateBots(mutations) => {
                for mutation in mutations {
                    mutation.apply(&mut state.bots);
                }
            }
        }
    }
}

pub(crate) enum ChatStateMutatorMutation {
    Base(ChatStateMutation),
    MutateMessages(ListMutatorMutation<Message>),
    MutateBots(ListMutatorMutation<Bot>),
}

impl ChatStateMutatorMutation {
    pub(crate) fn into_state_mutations(self, state: &ChatState) -> Vec<ChatStateMutation> {
        match self {
            ChatStateMutatorMutation::Base(m) => vec![m],
            ChatStateMutatorMutation::MutateMessages(m) => {
                let list_mutation = m.into_list_mutations(&state.messages);
                vec![ChatStateMutation::MutateMessages(vec![list_mutation])]
            }
            ChatStateMutatorMutation::MutateBots(m) => {
                let list_mutation = m.into_list_mutations(&state.bots);
                vec![ChatStateMutation::MutateBots(vec![list_mutation])]
            }
        }
    }
}

pub struct ChatStateMutator {
    mutations: Vec<ChatStateMutatorMutation>,
}

impl ChatStateMutator {
    pub(crate) fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn set_is_streaming(&mut self, is_streaming: bool) {
        self.mutations.push(ChatStateMutatorMutation::Base(
            ChatStateMutation::SetIsStreaming(is_streaming),
        ));
    }

    pub fn set_load_status(&mut self, status: Status) {
        self.mutations.push(ChatStateMutatorMutation::Base(
            ChatStateMutation::SetLoadStatus(status),
        ));
    }

    pub fn mutate_messages<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ListMutator<Message>),
    {
        let mut list_mutator = ListMutator::new();
        f(&mut list_mutator);
        let mutations = list_mutator.finish();
        for m in mutations {
            self.mutations
                .push(ChatStateMutatorMutation::MutateMessages(m));
        }
    }

    pub fn mutate_bots<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ListMutator<Bot>),
    {
        let mut list_mutator = ListMutator::new();
        f(&mut list_mutator);
        let mutations = list_mutator.finish();
        for m in mutations {
            self.mutations.push(ChatStateMutatorMutation::MutateBots(m));
        }
    }

    pub(crate) fn finish(self) -> Vec<ChatStateMutatorMutation> {
        self.mutations
    }
}

impl From<ListMutation<Message>> for ChatStateMutation {
    fn from(mutation: ListMutation<Message>) -> Self {
        ChatStateMutation::MutateMessages(vec![mutation])
    }
}

impl From<ListMutation<Bot>> for ChatStateMutation {
    fn from(mutation: ListMutation<Bot>) -> Self {
        ChatStateMutation::MutateBots(vec![mutation])
    }
}

impl From<ChatStateMutation> for Vec<ChatStateMutation> {
    fn from(mutation: ChatStateMutation) -> Self {
        vec![mutation]
    }
}
