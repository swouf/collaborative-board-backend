// Import required modules from the RLLM library
use rllm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ChatRole, MessageType},
};
use tracing::{Level, event};

const SYSTEM_PROMPT: &str =
    "You are an helpful assistant providing feedback on ideas that are given to you.";

pub async fn get_feedback(response0: &String) -> Option<String> {
    // Get Ollama server URL from environment variable or use default localhost
    let base_url =
        std::env::var("OLLAMA_HOST").unwrap_or("http://host.docker.internal:11434".into());

    // Initialize and configure the LLM client
    let llm = LLMBuilder::new()
        .backend(LLMBackend::Ollama) // Use Ollama as the LLM backend
        .base_url(base_url) // Set the Ollama server URL
        .model("gemma3:4b") // Use the Mistral model
        .system(SYSTEM_PROMPT)
        .max_tokens(1000) // Set maximum response length
        .temperature(0.7) // Control response randomness (0.0-1.0)
        .stream(false) // Disable streaming responses
        .build()
        .expect("Failed to build LLM (Ollama)");

    // Prepare conversation history with example messages
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: format!("What do you think about this idea:\n{}", response0).into(),
        message_type: MessageType::Text,
    }];

    // Send chat request and handle the response
    match llm.chat(&messages).await {
        Ok(chat_response) => chat_response.text(),
        Err(err) => {
            event!(Level::ERROR, "LLM error:\n{}", err);
            None
        }
    }
}
