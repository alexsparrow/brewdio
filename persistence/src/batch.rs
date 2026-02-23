use crate::connection::{Connection, DbError, Value};

pub struct BatchRow {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub data: String,
    pub am_data: Vec<u8>,
    pub is_deleted: bool,
    pub is_dirty: bool,
}

fn row_from_query(row: &dyn crate::connection::Row) -> BatchRow {
    BatchRow {
        id: row.get_text(0),
        name: row.get_text(1),
        recipe_id: row.get_text(2),
        data: row.get_text(3),
        am_data: row.get_blob(4),
        is_deleted: row.get_bool(5),
        is_dirty: row.get_bool(6),
    }
}

pub fn create_batch(
    conn: &(impl Connection + ?Sized),
    name: &str,
    recipe_id: &str,
    data: &str,
) -> Result<BatchRow, DbError> {
    let id = ulid::Ulid::new().to_string();
    conn.execute(
        "INSERT INTO batch (id, name, recipe_id, data) VALUES (?1, ?2, ?3, ?4)",
        &[
            Value::Text(&id),
            Value::Text(name),
            Value::Text(recipe_id),
            Value::Text(data),
        ],
    )?;
    Ok(BatchRow {
        id,
        name: name.to_string(),
        recipe_id: recipe_id.to_string(),
        data: data.to_string(),
        am_data: Vec::new(),
        is_deleted: false,
        is_dirty: true,
    })
}

pub fn get_batch(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<Option<BatchRow>, DbError> {
    conn.query_one(
        "SELECT id, name, recipe_id, data, am_data, is_deleted, is_dirty FROM batch WHERE id = ?1",
        &[Value::Text(id)],
        row_from_query,
    )
}

pub fn list_batches(conn: &(impl Connection + ?Sized)) -> Result<Vec<BatchRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe_id, data, am_data, is_deleted, is_dirty FROM batch WHERE is_deleted = FALSE",
        &[],
        row_from_query,
    )
}

pub fn update_batch(
    conn: &(impl Connection + ?Sized),
    id: &str,
    name: &str,
    data: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE batch SET name = ?1, data = ?2, is_dirty = TRUE WHERE id = ?3",
        &[Value::Text(name), Value::Text(data), Value::Text(id)],
    )
}

pub fn delete_batch(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE batch SET is_deleted = TRUE, is_dirty = TRUE WHERE id = ?1",
        &[Value::Text(id)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    #[test]
    fn batch_crud() {
        let conn = test_conn();

        // Create
        let row = create_batch(&conn, "Batch 1", "recipe-123", r#"{"notes":"test"}"#).unwrap();
        assert_eq!(row.name, "Batch 1");
        assert!(!row.is_deleted);
        assert!(row.is_dirty);

        // Get
        let fetched = get_batch(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Batch 1");
        assert_eq!(fetched.recipe_id, "recipe-123");
        assert!(fetched.is_dirty);

        // List
        let batches = list_batches(&conn).unwrap();
        assert_eq!(batches.len(), 1);

        // Update
        update_batch(&conn, &row.id, "Updated Batch", r#"{"notes":"updated"}"#).unwrap();
        let fetched = get_batch(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Batch");

        // Soft-delete
        delete_batch(&conn, &row.id).unwrap();
        let batches = list_batches(&conn).unwrap();
        assert_eq!(batches.len(), 0);

        // Still exists
        let fetched = get_batch(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.is_deleted);
        assert!(fetched.is_dirty);
    }
}
