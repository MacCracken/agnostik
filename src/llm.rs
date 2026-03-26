use serde::{Deserialize, Serialize};

/// LLM provider identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Token usage for an inference request.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM inference request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub stream: bool,
}

fn default_temperature() -> f32 { 0.7 }

/// LLM inference response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}

/// Why inference stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Error,
}

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
            max_tokens: Some(100),
            temperature: 0.7,
            stream: false,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InferenceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.model, "llama3");
    }

    #[test]
    fn finish_reason_variants() {
        assert_ne!(FinishReason::Stop, FinishReason::Length);
    }
}
