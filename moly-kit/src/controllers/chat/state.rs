use super::plugin::*;
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

pub struct ListMutator<'a, T: Clone> {
    list: &'a [T],
    mutations: Vec<ListMutation<T>>,
    len: usize,
}

impl<'a, T: Clone> ListMutator<'a, T> {
    pub(crate) fn new(list: &'a [T]) -> Self {
        Self {
            list,
            mutations: Vec::new(),
            len: list.len(),
        }
    }

    pub fn extend(&mut self, index: usize, items: Vec<T>) {
        if index > self.len {
            panic!(
                "Invalid insert index: {}. Maximum allowed at this point is {}",
                index, self.len
            );
        }

        self.len += items.len();
        self.mutations.push(ListMutation::Insert(index, items));
    }

    pub fn remove_range(&mut self, index: usize, count: usize) {
        if index + count > self.len {
            panic!(
                "Invalid remove range: {}..{}. Maximum allowed at this point is {}",
                index,
                index + count,
                self.len
            );
        }

        self.len = self.len.saturating_sub(count);
        self.mutations.push(ListMutation::Remove(index, count));
    }

    pub fn update(&mut self, index: usize, item: T) {
        if index >= self.len {
            panic!(
                "Invalid update index: {}. Maximum allowed at this point is {}",
                index,
                self.len.saturating_sub(1)
            );
        }

        self.mutations.push(ListMutation::Update(index, item));
    }

    pub fn insert(&mut self, index: usize, item: T) {
        self.extend(index, vec![item]);
    }

    pub fn remove(&mut self, index: usize) {
        self.remove_range(index, 1);
    }

    pub fn push(&mut self, item: T) {
        self.extend(self.len, vec![item]);
    }

    pub(crate) fn finish(self) -> Vec<ListMutation<T>> {
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

pub struct ChatStateMutator<'a> {
    state: &'a ChatState,
    mutations: Vec<ChatStateMutation>,
}

impl<'a> ChatStateMutator<'a> {
    pub(crate) fn new(state: &'a ChatState) -> Self {
        Self {
            state,
            mutations: Vec::new(),
        }
    }

    pub fn set_is_streaming(&mut self, is_streaming: bool) {
        self.mutations
            .push(ChatStateMutation::SetIsStreaming(is_streaming));
    }

    pub fn set_load_status(&mut self, status: Status) {
        self.mutations
            .push(ChatStateMutation::SetLoadStatus(status));
    }

    pub(crate) fn finish(self) -> Vec<ChatStateMutation> {
        self.mutations
    }
}
