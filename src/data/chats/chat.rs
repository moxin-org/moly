use crate::shared::utils::filesystem;
use anyhow::{Result, anyhow};
use moly_kit::{BotId, Message, utils::asynchronous::spawn};
use moly_protocol::data::FileID;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub type ChatID = u128;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
enum TitleState {
    #[default]
    Default,
    Updated,
}

#[derive(Serialize, Deserialize)]
struct ChatData {
    id: ChatID,
    associated_bot: Option<BotId>,
    system_prompt: Option<String>,
    messages: Vec<Message>,
    title: String,
    #[serde(default)]
    title_state: TitleState,
    #[serde(default)]
    accessed_at: chrono::DateTime<chrono::Utc>,

    // Legacy field, it can be removed in the future.
    last_used_file_id: Option<FileID>,
}

#[derive(Debug, Clone)]
pub struct ChatInferenceParams {
    pub frequency_penalty: f32,
    pub max_tokens: u32,
    pub presence_penalty: f32,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
    pub stop: String,
}

impl Default for ChatInferenceParams {
    fn default() -> Self {
        Self {
            frequency_penalty: 0.0,
            max_tokens: 2048,
            presence_penalty: 0.0,
            temperature: 1.0,
            top_p: 1.0,
            stream: true,
            stop: "".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chat {
    /// Unix timestamp in ms.
    pub id: ChatID,

    /// This is the model or agent that is currently "active" on the chat
    /// For models it is the most recent model used or loaded in the context of this chat session.
    /// For agents it is the agent that originated the chat.
    pub associated_bot: Option<BotId>,

    pub messages: Vec<Message>,
    pub inferences_params: ChatInferenceParams,
    pub system_prompt: Option<String>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub has_unread_messages: bool,

    title: String,
    title_state: TitleState,
    chats_dir: PathBuf,
}

impl Chat {
    pub fn new(chats_dir: PathBuf) -> Self {
        // Get Unix timestamp in ms for id.
        let id = chrono::Utc::now().timestamp_millis() as u128;

        Self {
            id,
            title: String::from("New Chat"),
            messages: vec![],
            associated_bot: None,
            title_state: TitleState::default(),
            chats_dir,
            inferences_params: ChatInferenceParams::default(),
            system_prompt: None,
            accessed_at: chrono::Utc::now(),
            has_unread_messages: false,
        }
    }

    pub async fn load(path: &Path) -> Result<Self> {
        let fs = filesystem::global();
        let dir = path
            .parent()
            .ok_or_else(|| anyhow!("Invalid chat file path"))?;

        match fs.read_json::<ChatData>(path).await {
            Ok(data) => {
                let chat = Chat {
                    id: data.id,
                    associated_bot: data.associated_bot,
                    messages: data.messages,
                    title: data.title,
                    title_state: data.title_state,
                    chats_dir: dir.to_path_buf(),
                    inferences_params: ChatInferenceParams::default(),
                    system_prompt: data.system_prompt,
                    accessed_at: data.accessed_at,
                    has_unread_messages: false,
                };

                Ok(chat)
            }
            Err(_) => Err(anyhow!("Couldn't read chat file from path")),
        }
    }

    pub async fn save(&self) {
        let path = self.chats_dir.join(self.file_name());
        let data = ChatData {
            id: self.id,
            associated_bot: self.associated_bot.clone(),
            system_prompt: self.system_prompt.clone(),
            messages: self.messages.clone(),
            title: self.title.clone(),
            title_state: self.title_state,
            accessed_at: self.accessed_at,

            // Legacy field, it can be removed in the future.
            last_used_file_id: None,
        };

        filesystem::global()
            .queue_write_json(path, &data)
            .await
            .unwrap();
    }

    pub fn save_and_forget(&self) {
        let self_clone = self.clone();
        spawn(async move {
            self_clone.save().await;
        });
    }

    pub fn remove_saved_file_and_forget(&self) {
        let path = self.chats_dir.join(self.file_name());
        spawn(async move {
            filesystem::global().remove(&path).await.unwrap();
        });
    }

    fn file_name(&self) -> String {
        format!("{}.chat.json", self.id)
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.title_state = TitleState::Updated;
    }

    pub fn update_title_based_on_first_message(&mut self) {
        // If it hasnt been updated, and theres at least one message, use the first
        // one as title. Else we just return the default one.
        if matches!(self.title_state, TitleState::Default) {
            if let Some(message) = self.messages.first() {
                let max_char_length = 25;
                let ellipsis = "...";

                let title = if message.content.text.len() > max_char_length {
                    let mut truncated = message
                        .content
                        .text
                        .chars()
                        .take(max_char_length)
                        .collect::<String>()
                        .replace('\n', " ");
                    truncated.push_str(ellipsis);
                    truncated
                } else {
                    message.content.text.clone()
                };

                self.set_title(title);
            }
        }
    }

    pub fn delete_message(&mut self, message_index: usize) {
        self.messages.remove(message_index);
    }

    pub fn update_accessed_at(&mut self) {
        self.accessed_at = chrono::Utc::now();
    }
}
