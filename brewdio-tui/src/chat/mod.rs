pub mod client;
pub mod tools;
pub mod ui;

use brewdio_core::beerjson_types::RecipeType;
use ratatui::style::{Color, Style};
use serde_json::Value as JsonValue;
use tui_textarea::TextArea;

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
    /// Extra provider-specific fields (e.g. Gemini's thought_signature) to replay on round-trips.
    pub extra: serde_json::Map<String, JsonValue>,
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
    pub editor: TextArea<'static>,
    pub scroll_offset: usize,
    pub is_streaming: bool,
    pub user_scrolled: bool,
    pub recipe_context: Option<(String, RecipeType)>,
    pub fullscreen: bool,
}

impl ChatState {
    fn make_editor() -> TextArea<'static> {
        let mut editor = TextArea::default();
        editor.set_placeholder_text("Type your message...");
        editor.set_placeholder_style(Style::default().fg(Color::DarkGray));
        editor.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
        editor.set_cursor_line_style(Style::default());
        editor
    }

    pub fn new(recipe_context: Option<(String, RecipeType)>) -> Self {
        let has_recipe = recipe_context.is_some();
        let mut state = ChatState {
            messages: Vec::new(),
            editor: Self::make_editor(),
            scroll_offset: 0,
            is_streaming: false,
            user_scrolled: false,
            recipe_context,
            fullscreen: false,
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

    /// Get the current input text (single line).
    pub fn input_text(&self) -> String {
        self.editor.lines().join("")
    }

    /// Clear the editor content.
    pub fn clear_input(&mut self) {
        self.editor = Self::make_editor();
    }
}
