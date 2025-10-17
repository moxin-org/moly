use crate::{display_name_from_namespaced, protocol::*, utils::vec::*};

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

#[derive(Debug, Clone, PartialEq)]
pub enum ChatStateMutation {
    SetIsStreaming(bool),
    SetLoadStatus(Status),
    MutateMessages(VecMutation<Message>),
    MutateBots(VecMutation<Bot>),
}

impl ChatStateMutation {
    pub fn apply(self, state: &mut ChatState) {
        match self {
            ChatStateMutation::SetIsStreaming(is_streaming) => {
                state.is_streaming = is_streaming;
            }
            ChatStateMutation::SetLoadStatus(status) => {
                state.load_status = status;
            }
            ChatStateMutation::MutateMessages(mutation) => {
                mutation.apply(&mut state.messages);
            }
            ChatStateMutation::MutateBots(mutation) => {
                mutation.apply(&mut state.bots);
            }
        }
    }
}

impl From<VecMutation<Message>> for ChatStateMutation {
    fn from(mutation: VecMutation<Message>) -> Self {
        ChatStateMutation::MutateMessages(mutation)
    }
}

impl From<VecMutation<Bot>> for ChatStateMutation {
    fn from(mutation: VecMutation<Bot>) -> Self {
        ChatStateMutation::MutateBots(mutation)
    }
}
