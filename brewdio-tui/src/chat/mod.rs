pub mod client;
pub mod tools;
pub mod ui;

use brewdio_core::beerjson_types::RecipeType;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: JsonValue,
    pub result: Option<JsonValue>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub tool_calls: Vec<ToolCallInfo>,
}

pub enum ChatEvent {
    TokenDelta(String),
    ToolCallStart {
        id: String,
        name: String,
        arguments: JsonValue,
    },
    ToolCallResult {
        id: String,
        result: JsonValue,
    },
    ToolCallError {
        id: String,
        error: String,
    },
    Done,
    Error(String),
}

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_offset: usize,
    pub is_streaming: bool,
    pub user_scrolled: bool,
    pub recipe_context: Option<(String, RecipeType)>,
}

impl ChatState {
    pub fn new(recipe_context: Option<(String, RecipeType)>) -> Self {
        let has_recipe = recipe_context.is_some();
        let mut state = ChatState {
            messages: Vec::new(),
            input: String::new(),
            scroll_offset: 0,
            is_streaming: false,
            user_scrolled: false,
            recipe_context,
        };

        // Add initial assistant greeting
        let greeting = if has_recipe {
            "Hi! I can help you with this recipe. I can:\n\
             - View and modify this recipe's ingredients, batch size, etc.\n\
             - Search for fermentables, hops, and yeast cultures\n\
             - Answer brewing questions\n\n\
             What would you like to do?"
        } else {
            "Hi! I can help you with brewing recipes. I can:\n\
             - Create new recipes\n\
             - Search for fermentables, hops, and yeast cultures\n\
             - List beer styles\n\
             - Answer questions about brewing\n\n\
             Navigate to a specific recipe to modify it."
        };

        state.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: greeting.to_string(),
            tool_calls: Vec::new(),
        });

        state
    }
}
