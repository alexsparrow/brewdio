use serde::{Deserialize, Serialize};

/// Document type discriminator for the sync protocol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Recipe,
    Batch,
    Settings,
}

/// Wire protocol messages for Automerge sync over WebSocket.
/// Serialized as JSON and sent as WebSocket text frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    /// Startup handshake — each side sends its document IDs per type.
    Hello {
        recipe_ids: Vec<String>,
        batch_ids: Vec<String>,
        settings_ids: Vec<String>,
    },
    /// Automerge sync message for a specific document.
    SyncDoc {
        doc_type: DocType,
        doc_id: String,
        data: Vec<u8>,
    },
    /// Full doc for a document the other side doesn't have.
    NewDoc {
        doc_type: DocType,
        doc_id: String,
        am_data: Vec<u8>,
    },
}
