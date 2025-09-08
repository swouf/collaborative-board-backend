use loro::{LoroValue, cursor::Cursor, loro_value};
// Import required modules from the RLLM library
use rllm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ChatRole, MessageType},
};
use tracing::{Level, event};

use crate::ws::{message::QueryAIMessage, room::Room, ws_codec::decode};

const SYSTEM_PROMPT: &str = "You are an helpful assistant providing feedback on ideas that are given to you. Always conclude your feedback with a question that steer the user to a more creative idea.";

pub async fn get_feedback(query: QueryAIMessage, room: &Room) -> Result<String, &'static str> {
    let err_msg: String;

    let responses = room.state.get_list("responses");

    let params = match query.parameters {
        Some(p) => p,
        None => return Err("No parameters in query."),
    };

    let cursor_ref = match params.get("cursor") {
        Some(c) => c,
        None => return Err("Couldn't find 'cursor' field in parameters."),
    };

    let cursor = match Cursor::decode(&decode(cursor_ref)) {
        Ok(c) => c,
        Err(_) => return Err("Error in decoding cursor."),
    };

    let position = match room.state.get_cursor_pos(&cursor) {
        Ok(pos_query) => pos_query.current.pos,
        Err(_) => return Err("Can't infer valid position."),
    };

    let mut response_struct = match responses.get(position) {
        Some(response) => response.into_value().unwrap().into_map().unwrap(),
        None => return Err("Couldn't find response at given position."),
    };

    let response_content = match response_struct.get("response") {
        Some(r) => r.clone().into_string().unwrap().unwrap(),
        None => return Err("Unable to get response content."),
    };

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
        content: format!("What do you think about this idea:\n{response_content}
        \n\n
        Try to be concise with your answer. Limit it to about 250 characters."),
        message_type: MessageType::Text,
    }];

    // Send chat request and handle the response
    let chat_response = match llm.chat(&messages).await {
        Ok(chat_response) => match chat_response.text() {
            Some(txt) => txt,
            None => return Err("No text in AI response."),
        },
        Err(err) => {
            err_msg = format!("LLM error:\n{err}");
            event!(Level::ERROR, "{}", err_msg.clone());
            return Err("LLM encountered an error.");
        }
    };
    let updated_response = response_struct.make_mut();
    updated_response.insert(String::from("feedback"), loro_value!(chat_response.clone()));

    event!(
        Level::DEBUG,
        "Here is the updated response: {:#?}",
        updated_response
    );

    responses.delete(position, 1).unwrap();
    responses
        .insert(position, LoroValue::Map(response_struct))
        .unwrap();

    event!(Level::DEBUG, "Responses updated:\n{:#?}", responses);

    room.state.commit();

    Ok(chat_response)
}
