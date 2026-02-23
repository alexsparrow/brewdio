CREATE TABLE IF NOT EXISTS recipe (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    recipe TEXT NOT NULL,
    am_data BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    is_dirty BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS sync_state (
    recipe_id TEXT NOT NULL,
    peer_id TEXT NOT NULL DEFAULT 'server',
    state BLOB NOT NULL,
    PRIMARY KEY (recipe_id, peer_id),
    FOREIGN KEY (recipe_id) REFERENCES recipe(id)
);
