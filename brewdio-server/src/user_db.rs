use brewdio_persistence::connection::{Connection, DbError, Value};
use brewdio_persistence::protocol::DocType;

/// List all document IDs for a given type owned by a specific user (including deleted).
pub fn list_all_doc_ids_for_user(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    user_id: &str,
) -> Result<Vec<String>, DbError> {
    conn.query_map(
        &format!("SELECT id FROM {} WHERE user_id = ?1", doc_type.table_name()),
        &[Value::Text(user_id)],
        |row| row.get_text(0),
    )
}

/// List all dirty documents owned by a specific user.
pub fn list_dirty_docs_for_user(
    conn: &(impl Connection + ?Sized),
    user_id: &str,
) -> Result<Vec<(DocType, String)>, DbError> {
    let mut result = Vec::new();
    for &doc_type in DocType::ALL {
        let ids: Vec<String> = conn.query_map(
            &format!(
                "SELECT id FROM {} WHERE is_dirty = TRUE AND user_id = ?1",
                doc_type.table_name()
            ),
            &[Value::Text(user_id)],
            |row| row.get_text(0),
        )?;
        for id in ids {
            result.push((doc_type, id));
        }
    }
    Ok(result)
}

/// Check if a document is owned by a specific user.
pub fn doc_owned_by_user(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    doc_id: &str,
    user_id: &str,
) -> Result<bool, DbError> {
    let count = conn.query_one(
        &format!(
            "SELECT COUNT(*) FROM {} WHERE id = ?1 AND user_id = ?2",
            doc_type.table_name()
        ),
        &[Value::Text(doc_id), Value::Text(user_id)],
        |row| row.get_text(0).parse::<i32>().unwrap_or(0),
    )?;
    Ok(count.unwrap_or(0) > 0)
}

/// Set user_id on a newly created document (from NewDoc sync).
pub fn set_doc_user_id(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    doc_id: &str,
    user_id: &str,
) -> Result<(), DbError> {
    conn.execute(
        &format!(
            "UPDATE {} SET user_id = ?1 WHERE id = ?2",
            doc_type.table_name()
        ),
        &[Value::Text(user_id), Value::Text(doc_id)],
    )
}
