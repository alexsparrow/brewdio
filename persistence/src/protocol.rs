use serde::{Deserialize, Serialize};

/// Wire protocol messages for Automerge sync over WebSocket.
/// Serialized as JSON and sent as WebSocket text frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    /// Startup handshake — each side sends its recipe IDs.
    Hello { recipe_ids: Vec<String> },
    /// Automerge sync message for a specific recipe.
    SyncDoc { recipe_id: String, data: Vec<u8> },
    /// Full doc for a recipe the other side doesn't have.
    NewDoc {
        recipe_id: String,
        am_data: Vec<u8>,
    },
}
