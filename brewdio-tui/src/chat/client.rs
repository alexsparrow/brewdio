use std::sync::mpsc;

use brewdio_core::beerjson_types::RecipeType;
use futures_util::StreamExt;
use serde_json::{json, Value as JsonValue};

use super::tools;
use super::{ChatEvent, ChatMessage, ChatRole};

const MAX_TOOL_ROUNDS: usize = 50;

const SYSTEM_PROMPT: &str = "\
You are a helpful brewing assistant. When using tools:
- Be efficient - complete tasks in as few steps as possible
- NEVER call the same tool multiple times in a row
- After calling getCurrentRecipe ONCE, you have the recipe data - use it directly, do NOT call it again
- After getting data from any tool, use it immediately - don't retrieve it again
- When you have enough information to answer, STOP calling tools and respond to the user";

/// Build the OpenAI messages array from our ChatMessage list.
fn build_messages(
    messages: &[ChatMessage],
    recipe_context: &Option<(String, RecipeType)>,
) -> Vec<JsonValue> {
    let has_recipe = recipe_context.is_some();
    let context_msg = if has_recipe {
        "The user is currently viewing a recipe. You can use getCurrentRecipe to read it and updateCurrentRecipe to modify it. \
         You can also search for ingredients and answer brewing questions.\n\n\
         IMPORTANT: Do not call the same tool multiple times with the same parameters. Complete tasks efficiently in as few steps as possible."
    } else {
        "The user is not viewing a specific recipe. You can create new recipes, search for ingredients, \
         list beer styles, and answer brewing questions. Navigate to a specific recipe to modify it.\n\n\
         IMPORTANT: Use tools efficiently - avoid calling the same tool repeatedly."
    };

    let mut api_messages = vec![
        json!({ "role": "system", "content": SYSTEM_PROMPT }),
        json!({ "role": "system", "content": context_msg }),
    ];

    for msg in messages {
        match msg.role {
            ChatRole::User => {
                api_messages.push(json!({ "role": "user", "content": msg.content }));
            }
            ChatRole::Assistant => {
                if msg.tool_calls.is_empty() {
                    api_messages.push(json!({ "role": "assistant", "content": msg.content }));
                } else {
                    // Assistant message with tool calls
                    let tool_calls: Vec<JsonValue> = msg
                        .tool_calls
                        .iter()
                        .map(|tc| {
                            let mut obj = json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                }
                            });
                            // Replay provider-specific fields (e.g. Gemini thought_signature)
                            if let Some(map) = obj.as_object_mut() {
                                for (k, v) in &tc.extra {
                                    map.insert(k.clone(), v.clone());
                                }
                            }
                            obj
                        })
                        .collect();

                    let mut assistant_msg =
                        json!({ "role": "assistant", "tool_calls": tool_calls });
                    if !msg.content.is_empty() {
                        assistant_msg["content"] = json!(msg.content);
                    }
                    api_messages.push(assistant_msg);

                    // Add tool result messages
                    for tc in &msg.tool_calls {
                        let content = if let Some(ref err) = tc.error {
                            format!("Error: {}", err)
                        } else if let Some(ref result) = tc.result {
                            serde_json::to_string(result).unwrap_or_default()
                        } else {
                            String::new()
                        };
                        api_messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tc.id,
                            "content": content,
                        }));
                    }
                }
            }
            ChatRole::System => {
                api_messages.push(json!({ "role": "system", "content": msg.content }));
            }
            ChatRole::Tool => {
                // Tool messages are handled inline with assistant messages above
            }
        }
    }

    api_messages
}

/// Accumulated state for a single tool call being streamed.
#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments_buf: String,
    /// Extra provider-specific fields to preserve across round-trips.
    extra: serde_json::Map<String, JsonValue>,
}

/// Main async entry point: streams chat with an OpenAI-compatible API, handles tool calls, sends events via tx.
pub async fn stream_chat(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<ChatMessage>,
    recipe_context: Option<(String, RecipeType)>,
    conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    tx: mpsc::Sender<ChatEvent>,
) {
    let base_url = if base_url.is_empty() {
        "https://api.openai.com/v1".to_string()
    } else {
        base_url.trim_end_matches('/').to_string()
    };
    let model = if model.is_empty() {
        "gpt-4o-mini".to_string()
    } else {
        model
    };
    let has_recipe = recipe_context.is_some();
    let tool_defs = tools::tool_definitions(has_recipe);
    let mut recipe_ctx = recipe_context;
    let mut all_messages = messages;

    let client = reqwest::Client::new();

    for _round in 0..MAX_TOOL_ROUNDS {
        let api_messages = build_messages(&all_messages, &recipe_ctx);

        let mut body = json!({
            "model": &model,
            "messages": api_messages,
            "stream": true,
        });
        if !tool_defs.is_empty() {
            body["tools"] = json!(tool_defs);
        }

        log::debug!("[chat] round {} sending {} messages", _round + 1, api_messages.len());

        let response = match client
            .post(format!("{}/chat/completions", &base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("[chat] request error: {}", e);
                let msg = if e.is_connect() {
                    "Connection error. Check your internet connection.".to_string()
                } else {
                    format!("Network error: {}", e)
                };
                let _ = tx.send(ChatEvent::Error(msg));
                return;
            }
        };

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[chat] API error {}: {}", status, error_text);
            let msg = match status.as_u16() {
                401 => "Invalid API key. Update it in Settings.".to_string(),
                429 => "Rate limited. Please wait a moment and try again.".to_string(),
                _ => format!("API error ({}): {}", status, error_text),
            };
            let _ = tx.send(ChatEvent::Error(msg));
            return;
        }

        // Parse SSE stream
        let mut content_buf = String::new();
        let mut pending_tool_calls: Vec<PendingToolCall> = Vec::new();
        let mut stream = response.bytes_stream();

        let mut line_buf = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(ChatEvent::Error(format!("Stream error: {}", e)));
                    return;
                }
            };

            let text = String::from_utf8_lossy(&chunk);
            line_buf.push_str(&text);

            // Process complete lines
            while let Some(newline_pos) = line_buf.find('\n') {
                let line = line_buf[..newline_pos].trim_end_matches('\r').to_string();
                line_buf = line_buf[newline_pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }

                    let parsed: JsonValue = match serde_json::from_str(data) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if let Some(choices) = parsed.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            let delta = match choice.get("delta") {
                                Some(d) => d,
                                None => continue,
                            };

                            // Content delta
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                content_buf.push_str(content);
                                let _ = tx.send(ChatEvent::TokenDelta(content.to_string()));
                            }

                            // Tool calls delta
                            if let Some(tool_calls) =
                                delta.get("tool_calls").and_then(|tc| tc.as_array())
                            {
                                for tc_delta in tool_calls {
                                    let index =
                                        tc_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0)
                                            as usize;

                                    // Extend pending_tool_calls if needed
                                    while pending_tool_calls.len() <= index {
                                        pending_tool_calls.push(PendingToolCall::default());
                                    }

                                    let pending = &mut pending_tool_calls[index];

                                    if let Some(id) = tc_delta.get("id").and_then(|i| i.as_str()) {
                                        pending.id = id.to_string();
                                    }
                                    if let Some(func) = tc_delta.get("function") {
                                        if let Some(name) =
                                            func.get("name").and_then(|n| n.as_str())
                                        {
                                            pending.name = name.to_string();
                                        }
                                        if let Some(args) =
                                            func.get("arguments").and_then(|a| a.as_str())
                                        {
                                            pending.arguments_buf.push_str(args);
                                        }
                                    }

                                    // Capture extra provider-specific fields
                                    if let Some(obj) = tc_delta.as_object() {
                                        for (k, v) in obj {
                                            match k.as_str() {
                                                "index" | "id" | "type" | "function" => {}
                                                _ => { pending.extra.insert(k.clone(), v.clone()); }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Stream finished — check if we have tool calls to execute
        if pending_tool_calls.is_empty() {
            // No tool calls — we're done
            let _ = tx.send(ChatEvent::Done);
            return;
        }

        // Execute tool calls
        let mut tool_call_infos = Vec::new();

        for pending in &pending_tool_calls {
            let arguments: JsonValue = serde_json::from_str(&pending.arguments_buf)
                .unwrap_or(json!({}));

            log::debug!("[chat] tool call: {} args={}", pending.name, arguments);
            let _ = tx.send(ChatEvent::ToolCallStart {
                id: pending.id.clone(),
                name: pending.name.clone(),
                arguments: arguments.clone(),
            });

            let result = {
                let c = conn.lock().unwrap_or_else(|e| e.into_inner());
                tools::execute_tool(&pending.name, &arguments, &*c, &mut recipe_ctx)
            };

            match result {
                Ok(result_val) => {
                    let _ = tx.send(ChatEvent::ToolCallResult {
                        id: pending.id.clone(),
                        result: result_val.clone(),
                    });
                    tool_call_infos.push(super::ToolCallInfo {
                        id: pending.id.clone(),
                        name: pending.name.clone(),
                        arguments,
                        result: Some(result_val),
                        error: None,
                        extra: pending.extra.clone(),
                    });
                }
                Err(err) => {
                    log::error!("[chat] tool {} error: {}", pending.name, err);
                    let _ = tx.send(ChatEvent::ToolCallError {
                        id: pending.id.clone(),
                        error: err.clone(),
                    });
                    tool_call_infos.push(super::ToolCallInfo {
                        id: pending.id.clone(),
                        name: pending.name.clone(),
                        arguments,
                        result: None,
                        error: Some(err),
                        extra: pending.extra.clone(),
                    });
                }
            }
        }

        // Add assistant message with tool calls to the conversation
        all_messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: content_buf.clone(),
            tool_calls: tool_call_infos,
        });

        // Clear content for next round
        content_buf.clear();

        // Loop back to call OpenAI again with tool results
    }

    // Exhausted tool rounds
    let _ = tx.send(ChatEvent::Error(
        "Reached maximum tool call rounds.".to_string(),
    ));
}
