use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio::time::{sleep, timeout, Duration};

use brewdio_core::beerjson_types::RecipeType;
use brewdio_persistence::automerge::reconcile_to_automerge;
use brewdio_persistence::batch;
use brewdio_persistence::recipe::RecipeDocument;
use brewdio_persistence::settings;
use brewdio_persistence::{connection_native, db, sync_worker};

fn sample_recipe() -> RecipeType {
    serde_json::from_str(
        r#"{
            "name": "Test IPA",
            "type": "all grain",
            "author": "Tester",
            "batch_size": { "unit": "l", "value": 20.0 },
            "efficiency": {
                "brewhouse": { "unit": "%", "value": 72.0 }
            },
            "ingredients": {
                "fermentable_additions": [{
                    "name": "Pale Malt",
                    "type": "grain",
                    "amount": { "unit": "kg", "value": 5.0 },
                    "color": { "unit": "SRM", "value": 2.0 },
                    "yield": { "fine_grind": { "unit": "%", "value": 80.0 } }
                }],
                "hop_additions": [{
                    "name": "Cascade",
                    "timing": {
                        "duration": { "unit": "min", "value": 60 },
                        "use": "add_to_boil"
                    },
                    "amount": { "unit": "g", "value": 30.0 },
                    "form": "pellet",
                    "alpha_acid": { "unit": "%", "value": 5.5 }
                }]
            }
        }"#,
    )
    .unwrap()
}

#[tokio::test]
async fn client_to_server_sync() {
    let server_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));
    let client_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));

    {
        let c = client_conn.lock().unwrap();
        db::create_recipe(&*c, "Client IPA", &sample_recipe(), None, None).unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let _server = brewdio_server::start_server(listener, server_conn.clone());

    let url = format!("ws://{}/ws", addr);
    let _client = sync_worker::spawn_sync(client_conn.clone(), url, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    let recipes = timeout(Duration::from_secs(10), async {
        loop {
            let recipes = {
                let c = server_conn.lock().unwrap();
                db::list_recipes(&*c).unwrap()
            };
            if !recipes.is_empty() {
                return recipes;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Timed out waiting for recipe to sync to server");

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].name, "Client IPA");
    assert!(!recipes[0].is_deleted, "Synced recipe should not be soft-deleted");

    // Also verify client-side recipe is still not deleted
    {
        let c = client_conn.lock().unwrap();
        let client_recipes = db::list_recipes(&*c).unwrap();
        assert_eq!(client_recipes.len(), 1);
        assert!(!client_recipes[0].is_deleted, "Client recipe should not be soft-deleted after sync");
    }
}

#[tokio::test]
async fn server_to_client_sync() {
    let server_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));
    let client_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));

    {
        let c = server_conn.lock().unwrap();
        db::create_recipe(&*c, "Server Stout", &sample_recipe(), None, None).unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let _server = brewdio_server::start_server(listener, server_conn.clone());

    let url = format!("ws://{}/ws", addr);
    let _client = sync_worker::spawn_sync(client_conn.clone(), url, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    let recipes = timeout(Duration::from_secs(10), async {
        loop {
            let recipes = {
                let c = client_conn.lock().unwrap();
                db::list_recipes(&*c).unwrap()
            };
            if !recipes.is_empty() {
                return recipes;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Timed out waiting for recipe to sync to client");

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].name, "Server Stout");
    assert!(!recipes[0].is_deleted, "Synced recipe should not be soft-deleted");

    // Also verify server-side recipe is still not deleted
    {
        let c = server_conn.lock().unwrap();
        let server_recipes = db::list_recipes(&*c).unwrap();
        assert_eq!(server_recipes.len(), 1);
        assert!(!server_recipes[0].is_deleted, "Server recipe should not be soft-deleted after sync");
    }
}

/// Both sides have the same recipe (same ID, independently created AM docs).
/// This exercises the SyncDoc path for shared recipes and verifies is_deleted
/// stays false through the merge.
#[tokio::test]
async fn shared_recipe_sync_preserves_is_deleted() {
    let server_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));
    let client_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));

    let recipe_id = "shared-recipe-1";
    let recipe = sample_recipe();

    // Create the same recipe on both sides with the same ID but different AM docs (different actors)
    {
        let doc = RecipeDocument {
            id: recipe_id.to_string(),
            name: "Server Version".to_string(),
            recipe: recipe.clone(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None).unwrap();
        let recipe_json = serde_json::to_string(&recipe).unwrap();

        let c = server_conn.lock().unwrap();
        c.execute(
            "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, FALSE, TRUE)",
            rusqlite::params![recipe_id, "Server Version", recipe_json, am_data],
        ).unwrap();
    }
    {
        let doc = RecipeDocument {
            id: recipe_id.to_string(),
            name: "Client Version".to_string(),
            recipe: recipe.clone(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None).unwrap();
        let recipe_json = serde_json::to_string(&recipe).unwrap();

        let c = client_conn.lock().unwrap();
        c.execute(
            "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, FALSE, TRUE)",
            rusqlite::params![recipe_id, "Client Version", recipe_json, am_data],
        ).unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let _server = brewdio_server::start_server(listener, server_conn.clone());

    let url = format!("ws://{}/ws", addr);
    let _client = sync_worker::spawn_sync(client_conn.clone(), url, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    // Wait for sync to converge (names should match on both sides)
    timeout(Duration::from_secs(10), async {
        loop {
            let server_name = {
                let c = server_conn.lock().unwrap();
                db::get_recipe(&*c, recipe_id).unwrap().map(|r| r.name.clone())
            };
            let client_name = {
                let c = client_conn.lock().unwrap();
                db::get_recipe(&*c, recipe_id).unwrap().map(|r| r.name.clone())
            };
            if server_name == client_name {
                return;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Timed out waiting for shared recipe to converge");

    // Verify neither side has the recipe soft-deleted
    {
        let c = server_conn.lock().unwrap();
        let row = db::get_recipe(&*c, recipe_id).unwrap().unwrap();
        assert!(!row.is_deleted, "Server recipe should not be soft-deleted after shared sync");
        let listed = db::list_recipes(&*c).unwrap();
        assert_eq!(listed.len(), 1, "Server should still list 1 recipe");
    }
    {
        let c = client_conn.lock().unwrap();
        let row = db::get_recipe(&*c, recipe_id).unwrap().unwrap();
        assert!(!row.is_deleted, "Client recipe should not be soft-deleted after shared sync");
        let listed = db::list_recipes(&*c).unwrap();
        assert_eq!(listed.len(), 1, "Client should still list 1 recipe");
    }
}

#[tokio::test]
async fn batch_client_to_server_sync() {
    let server_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));
    let client_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));

    // Create a batch on the client
    {
        let c = client_conn.lock().unwrap();
        let profile = &brewdio_core::data::equipment()[0];
        batch::create_batch_from_recipe(
            &*c,
            "Test Batch",
            "recipe-1",
            &sample_recipe(),
            &profile.equipment,
            &profile.id,
        )
        .unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let _server = brewdio_server::start_server(listener, server_conn.clone());

    let url = format!("ws://{}/ws", addr);
    let _client = sync_worker::spawn_sync(client_conn.clone(), url, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    let batches = timeout(Duration::from_secs(10), async {
        loop {
            let batches = {
                let c = server_conn.lock().unwrap();
                batch::list_batches(&*c).unwrap()
            };
            if !batches.is_empty() {
                return batches;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Timed out waiting for batch to sync to server");

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].name, "Test Batch");
    assert!(!batches[0].is_deleted);
}

#[tokio::test]
async fn settings_client_to_server_sync() {
    let server_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));
    let client_conn = Arc::new(Mutex::new(connection_native::open(":memory:").unwrap()));

    // Save settings on the client
    {
        let c = client_conn.lock().unwrap();
        settings::save_settings(&*c, r#"{"vimMode":true}"#).unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let _server = brewdio_server::start_server(listener, server_conn.clone());

    let url = format!("ws://{}/ws", addr);
    let _client = sync_worker::spawn_sync(client_conn.clone(), url, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    let synced_settings = timeout(Duration::from_secs(10), async {
        loop {
            let s = {
                let c = server_conn.lock().unwrap();
                settings::get_settings(&*c).unwrap()
            };
            if let Some(row) = s {
                return row;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Timed out waiting for settings to sync to server");

    assert_eq!(synced_settings.data, r#"{"vimMode":true}"#);
}
