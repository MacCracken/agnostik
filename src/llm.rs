use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// LLM provider identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LlmProvider {
    Ollama,
    LlamaCpp,
    OpenAI,
    Anthropic,
    Google,
    DeepSeek,
    Mistral,
    Grok,
    Groq,
    OpenRouter,
    LmStudio,
    LocalAI,
    Custom(String),
}

// ---------------------------------------------------------------------------
// Conversation / messages
// ---------------------------------------------------------------------------

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A block of content within a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ContentBlock {
    /// Plain text content.
    Text(String),
    /// A tool invocation produced by the model.
    ToolUse {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    /// The result of a tool invocation, fed back to the model.
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a simple text message.
    #[must_use]
    pub fn text(role: MessageRole, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentBlock::Text(text.into())],
        }
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

/// A tool that the model may invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    /// JSON Schema describing the tool's parameters.
    pub parameters: serde_json::Value,
}

/// A tool call returned by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// The result of executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

// ---------------------------------------------------------------------------
// Sampling parameters
// ---------------------------------------------------------------------------

/// Sampling parameters for inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub stop_sequences: Vec<String>,
    #[serde(default)]
    pub seed: Option<u64>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: Vec::new(),
            seed: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Token usage
// ---------------------------------------------------------------------------

/// Token usage for an inference request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Inference request / response
// ---------------------------------------------------------------------------

/// LLM inference request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model: String,
    /// Plain text prompt (for simple, single-turn requests).
    #[serde(default)]
    pub prompt: String,
    /// Structured conversation messages (preferred over `prompt`).
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub sampling: SamplingParams,
    #[serde(default)]
    pub stream: bool,
    /// Tools the model may invoke.
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
}

fn default_temperature() -> f32 {
    0.7
}

/// LLM inference response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    /// Tool calls extracted from content (convenience accessor).
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

/// Why inference stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolUse,
    Error,
}

// ---------------------------------------------------------------------------
// Streaming
// ---------------------------------------------------------------------------

/// A streaming event from an LLM inference response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StreamEvent {
    /// Incremental text delta.
    Delta { text: String },
    /// Incremental tool call arguments.
    ToolCallDelta {
        id: String,
        name: Option<String>,
        arguments_delta: String,
    },
    /// Token usage update (may arrive mid-stream or at end).
    Usage(TokenUsage),
    /// Stream completed.
    Done { finish_reason: FinishReason },
    /// Stream error.
    Error { message: String },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_usage_default() {
        let u = TokenUsage::default();
        assert_eq!(u.total_tokens, 0);
    }

    #[test]
    fn provider_variants() {
        assert_ne!(LlmProvider::Ollama, LlmProvider::OpenAI);
        let custom = LlmProvider::Custom("my-provider".into());
        assert_ne!(custom, LlmProvider::Ollama);
    }

    #[test]
    fn inference_request_serde() {
        let r = InferenceRequest {
            model: "llama3".into(),
            prompt: "hello".into(),
            messages: vec![],
            max_tokens: Some(100),
            sampling: SamplingParams::default(),
            stream: false,
            tools: vec![],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InferenceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.model, "llama3");
    }

    #[test]
    fn finish_reason_variants() {
        assert_ne!(FinishReason::Stop, FinishReason::Length);
        assert_ne!(FinishReason::ToolUse, FinishReason::Stop);
    }

    #[test]
    fn token_usage_serde_roundtrip() {
        let u = TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        let json = serde_json::to_string(&u).unwrap();
        let back: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.prompt_tokens, 10);
        assert_eq!(back.total_tokens, 30);
    }

    #[test]
    fn finish_reason_serde_roundtrip() {
        for variant in [
            FinishReason::Stop,
            FinishReason::Length,
            FinishReason::ContentFilter,
            FinishReason::ToolUse,
            FinishReason::Error,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FinishReason = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn inference_response_serde_roundtrip() {
        let r = InferenceResponse {
            model: "llama3".into(),
            content: vec![ContentBlock::Text("Hello world".into())],
            usage: TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 10,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
            tool_calls: vec![],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn llm_provider_serde_roundtrip() {
        for variant in [
            LlmProvider::Ollama,
            LlmProvider::LlamaCpp,
            LlmProvider::OpenAI,
            LlmProvider::Anthropic,
            LlmProvider::Google,
            LlmProvider::DeepSeek,
            LlmProvider::Mistral,
            LlmProvider::Grok,
            LlmProvider::Groq,
            LlmProvider::OpenRouter,
            LlmProvider::LmStudio,
            LlmProvider::LocalAI,
            LlmProvider::Custom("my-provider".into()),
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let _back: LlmProvider = serde_json::from_str(&json).unwrap();
        }
    }

    // --- New type tests ---

    #[test]
    fn message_role_serde_roundtrip() {
        for variant in [
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: MessageRole = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn message_text_helper() {
        let m = Message::text(MessageRole::User, "hello");
        assert_eq!(m.role, MessageRole::User);
        assert_eq!(m.content.len(), 1);
        assert!(matches!(&m.content[0], ContentBlock::Text(t) if t == "hello"));
    }

    #[test]
    fn content_block_serde_roundtrip() {
        let blocks = vec![
            ContentBlock::Text("hello".into()),
            ContentBlock::ToolUse {
                id: "call_1".into(),
                name: "search".into(),
                arguments: serde_json::json!({"query": "rust"}),
            },
            ContentBlock::ToolResult {
                tool_use_id: "call_1".into(),
                content: "found 10 results".into(),
                is_error: false,
            },
        ];
        for block in &blocks {
            let json = serde_json::to_string(block).unwrap();
            let back: ContentBlock = serde_json::from_str(&json).unwrap();
            assert_eq!(block, &back);
        }
    }

    #[test]
    fn message_serde_roundtrip() {
        let m = Message {
            role: MessageRole::Assistant,
            content: vec![
                ContentBlock::Text("Let me search.".into()),
                ContentBlock::ToolUse {
                    id: "call_1".into(),
                    name: "search".into(),
                    arguments: serde_json::json!({"q": "test"}),
                },
            ],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back.role, MessageRole::Assistant);
        assert_eq!(back.content.len(), 2);
    }

    #[test]
    fn tool_definition_serde_roundtrip() {
        let t = ToolDefinition {
            name: "search".into(),
            description: "Search the web".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {"query": {"type": "string"}}
            }),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "search");
    }

    #[test]
    fn tool_call_serde_roundtrip() {
        let tc = ToolCall {
            id: "call_1".into(),
            name: "search".into(),
            arguments: serde_json::json!({"query": "rust"}),
        };
        let json = serde_json::to_string(&tc).unwrap();
        let back: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "call_1");
        assert_eq!(back.name, "search");
    }

    #[test]
    fn tool_result_serde_roundtrip() {
        let tr = ToolResult {
            tool_use_id: "call_1".into(),
            content: "result".into(),
            is_error: true,
        };
        let json = serde_json::to_string(&tr).unwrap();
        let back: ToolResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tool_use_id, "call_1");
        assert!(back.is_error);
    }

    #[test]
    fn sampling_params_default() {
        let s = SamplingParams::default();
        assert!((s.temperature - 0.7).abs() < f32::EPSILON);
        assert!(s.top_p.is_none());
        assert!(s.stop_sequences.is_empty());
    }

    #[test]
    fn sampling_params_serde_roundtrip() {
        let s = SamplingParams {
            temperature: 0.5,
            top_p: Some(0.9),
            top_k: Some(40),
            frequency_penalty: Some(0.1),
            presence_penalty: None,
            stop_sequences: vec!["END".into()],
            seed: Some(42),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SamplingParams = serde_json::from_str(&json).unwrap();
        assert!((back.temperature - 0.5).abs() < f32::EPSILON);
        assert_eq!(back.top_k, Some(40));
        assert_eq!(back.seed, Some(42));
        assert_eq!(back.stop_sequences, vec!["END"]);
    }

    #[test]
    fn stream_event_serde_roundtrip() {
        let events = vec![
            StreamEvent::Delta {
                text: "hello".into(),
            },
            StreamEvent::ToolCallDelta {
                id: "call_1".into(),
                name: Some("search".into()),
                arguments_delta: "{\"q\":".into(),
            },
            StreamEvent::Usage(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
            StreamEvent::Done {
                finish_reason: FinishReason::Stop,
            },
            StreamEvent::Error {
                message: "timeout".into(),
            },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let _back: StreamEvent = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn inference_request_with_messages_and_tools() {
        let r = InferenceRequest {
            model: "gpt-4".into(),
            prompt: String::new(),
            messages: vec![
                Message::text(MessageRole::System, "You are helpful."),
                Message::text(MessageRole::User, "Search for rust"),
            ],
            max_tokens: Some(1000),
            sampling: SamplingParams {
                temperature: 0.0,
                seed: Some(42),
                ..SamplingParams::default()
            },
            stream: false,
            tools: vec![ToolDefinition {
                name: "search".into(),
                description: "Web search".into(),
                parameters: serde_json::json!({"type": "object"}),
            }],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InferenceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.messages.len(), 2);
        assert_eq!(back.tools.len(), 1);
        assert_eq!(back.sampling.seed, Some(42));
    }

    #[test]
    fn inference_response_with_tool_calls() {
        let r = InferenceResponse {
            model: "gpt-4".into(),
            content: vec![ContentBlock::ToolUse {
                id: "call_1".into(),
                name: "search".into(),
                arguments: serde_json::json!({"query": "rust"}),
            }],
            usage: TokenUsage::default(),
            finish_reason: FinishReason::ToolUse,
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "search".into(),
                arguments: serde_json::json!({"query": "rust"}),
            }],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.finish_reason, FinishReason::ToolUse);
        assert_eq!(back.tool_calls.len(), 1);
    }
}
