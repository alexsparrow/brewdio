use crate::automerge::{hydrate_from_automerge, reconcile_to_automerge};
use crate::connection::{Connection, DbError, Value};
use crate::protocol::DocType;

/// Trait for Automerge-reconciled documents that support soft-delete.
/// Implemented by RecipeDocument, BatchDocument, and EquipmentProfileDocument.
/// Settings is excluded (no `is_deleted` field).
pub trait SyncDocument: autosurgeon::Reconcile + autosurgeon::Hydrate {
    fn id(&self) -> &str;
    fn is_deleted(&self) -> bool;
    fn set_is_deleted(&mut self, deleted: bool);
}

/// Trait for database rows that correspond to a SyncDocument.
/// Provides the bridge between SQLite rows and Automerge documents.
pub trait SyncRow {
    type Document: SyncDocument;
    fn doc_type() -> DocType;
    fn id(&self) -> &str;
    fn am_data(&self) -> &[u8];
    fn into_document(&self) -> Result<Self::Document, DbError>;
}

/// Trait for document types that can be applied from remote Automerge merges.
/// Implemented by all 4 document types (including Settings).
pub trait RemoteMergeable: autosurgeon::Hydrate + Sized {
    fn apply_hydrated(
        conn: &(impl Connection + ?Sized),
        id: &str,
        doc: &Self,
        am_data: &[u8],
        exists: bool,
    ) -> Result<(), DbError>;

    fn apply_fallback(
        conn: &(impl Connection + ?Sized),
        id: &str,
        am_data: &[u8],
        exists: bool,
    ) -> Result<(), DbError>;
}

/// Generic soft-delete/undelete for any sync row type.
pub fn set_deleted<R: SyncRow>(
    conn: &(impl Connection + ?Sized),
    existing: Option<R>,
    id: &str,
    deleted: bool,
) -> Result<(), DbError> {
    if let Some(row) = existing {
        let mut doc = row.into_document()?;
        doc.set_is_deleted(deleted);
        let am_data =
            reconcile_to_automerge(&doc, Some(row.am_data())).map_err(|e| DbError(e))?;

        conn.execute(
            &format!(
                "UPDATE {} SET is_deleted = ?1, is_dirty = TRUE, am_data = ?2 WHERE id = ?3",
                R::doc_type().table_name()
            ),
            &[
                Value::Bool(deleted),
                Value::Blob(&am_data),
                Value::Text(id),
            ],
        )?;
    }
    Ok(())
}

/// Generic remote merge application for any RemoteMergeable document type.
pub fn apply_remote_merge_generic<D: RemoteMergeable>(
    conn: &(impl Connection + ?Sized),
    id: &str,
    am_data: &[u8],
    exists: bool,
) -> Result<(), DbError> {
    match hydrate_from_automerge::<D>(am_data) {
        Ok(doc) => D::apply_hydrated(conn, id, &doc, am_data, exists),
        Err(_) => D::apply_fallback(conn, id, am_data, exists),
    }
}
