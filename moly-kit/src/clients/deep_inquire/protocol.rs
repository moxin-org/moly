impl Message {
    /// Update the message with a delta
    pub fn apply_delta(&mut self, delta: MessageDelta) {
        match (&mut self.content, &delta.content) {
            // PlainText + PlainText -> append text and add citations
            (
                MessageContent::PlainText { text, citations },
                MessageContent::PlainText {
                    text: delta_text,
                    citations: delta_citations,
                },
            ) => {
                text.push_str(delta_text);
                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }

            // MultiStage + MultiStage -> update stages and append text
            (
                MessageContent::MultiStage {
                    text,
                    stages,
                    citations,
                },
                MessageContent::MultiStage {
                    text: delta_text,
                    stages: delta_stages,
                    citations: delta_citations,
                },
            ) => {
                // Append text if not empty
                if !delta_text.is_empty() {
                    text.push_str(delta_text);
                }

                // Merge stages from delta into existing stages
                for new_stage in delta_stages {
                    if let Some(existing_stage) = stages.iter_mut().find(|s| s.id == new_stage.id) {
                        // Update existing stage
                        if new_stage.thinking.is_some() {
                            existing_stage.thinking = new_stage.thinking.clone();
                        }
                        if new_stage.writing.is_some() {
                            existing_stage.writing = new_stage.writing.clone();
                        }
                        if new_stage.completed.is_some() {
                            existing_stage.completed = new_stage.completed.clone();
                            // Mark message as completed when we get a completed stage
                            self.is_writing = false;
                        }
                    } else {
                        // Add new stage
                        stages.push(new_stage.clone());
                    }
                }

                // Add new citations
                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }

            // PlainText + MultiStage -> convert to MultiStage and update
            (
                MessageContent::PlainText {
                    text: existing_text,
                    citations: existing_citations,
                },
                MessageContent::MultiStage {
                    text: delta_text,
                    stages: delta_stages,
                    citations: delta_citations,
                },
            ) => {
                let mut combined_text = existing_text.clone();
                if !delta_text.is_empty() {
                    combined_text.push_str(delta_text);
                }

                let mut combined_citations = existing_citations.clone();
                for citation in delta_citations {
                    if !combined_citations.contains(citation) {
                        combined_citations.push(citation.clone());
                    }
                }

                // Convert to MultiStage
                self.content = MessageContent::MultiStage {
                    text: combined_text,
                    stages: delta_stages.clone(),
                    citations: combined_citations,
                };
            }

            // MultiStage + PlainText -> just append text and citations
            (
                MessageContent::MultiStage {
                    text, citations, ..
                },
                MessageContent::PlainText {
                    text: delta_text,
                    citations: delta_citations,
                },
            ) => {
                text.push_str(delta_text);

                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }
        }
    }

    /// Gets the visible text content to display, regardless of the content type
    pub fn visible_text(&self) -> String {
        match &self.content {
            MessageContent::PlainText { text, .. } => text.clone(),
            MessageContent::MultiStage { text, .. } => text.clone(),
        }
    }

    /// Gets the citations/sources regardless of the content type
    pub fn sources(&self) -> Vec<String> {
        match &self.content {
            MessageContent::PlainText { citations, .. } => citations.clone(),
            MessageContent::MultiStage { citations, .. } => citations.clone(),
        }
    }

    /// Checks if this message has multi-stage content
    pub fn has_stages(&self) -> bool {
        match &self.content {
            MessageContent::PlainText { .. } => false,
            MessageContent::MultiStage { stages, .. } => !stages.is_empty(),
        }
    }

    /// Gets the stages if this message has multi-stage content
    pub fn get_stages(&self) -> Vec<MessageStage> {
        match &self.content {
            MessageContent::PlainText { .. } => Vec::new(),
            MessageContent::MultiStage { stages, .. } => stages.clone(),
        }
    }
}

// ---

/// Factory methods for creating properly formatted MessageDelta objects
pub trait MessageDeltaFactory {
    /// Create a text-only delta with optional citations
    fn text_delta(text: String, citations: Vec<String>) -> MessageDelta;

    /// Create a stage-based delta
    fn stage_delta(text: String, stage: MessageStage, citations: Vec<String>) -> MessageDelta;
}

impl MessageDeltaFactory for MessageDelta {
    fn text_delta(text: String, citations: Vec<String>) -> Self {
        MessageDelta {
            content: MessageContent::PlainText { text, citations },
        }
    }

    fn stage_delta(text: String, stage: MessageStage, citations: Vec<String>) -> Self {
        MessageDelta {
            content: MessageContent::MultiStage {
                text,
                stages: vec![stage],
                citations,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageStage {
    /// Stage identifier
    pub id: usize,
    /// Thinking block content
    pub thinking: Option<MessageBlockContent>,
    /// Writing block content
    pub writing: Option<MessageBlockContent>,
    /// Completed block content
    pub completed: Option<MessageBlockContent>,
}

impl MessageStage {
    /// Check if this stage has completed content
    pub fn is_completed(&self) -> bool {
        self.completed.is_some()
    }

    /// Get the text content of the most advanced stage (completed > writing > thinking)
    pub fn latest_content(&self) -> Option<&str> {
        if let Some(completed) = &self.completed {
            Some(&completed.content)
        } else if let Some(writing) = &self.writing {
            Some(&writing.content)
        } else if let Some(thinking) = &self.thinking {
            Some(&thinking.content)
        } else {
            None
        }
    }
}
