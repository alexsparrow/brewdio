use std::sync::{Arc, Mutex};

pub use brewdio_core::beerjson_types::*;
use brewdio_core::equipment_profile::EquipmentProfile;
use brewdio_persistence::batch::{self, Batch, BatchData};
use brewdio_persistence::recipe::Recipe;
use brewdio_persistence::{db, equipment_profile, settings, sync_worker};
use flutter_rust_bridge::frb;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

static DB_CONN: std::sync::OnceLock<Arc<Mutex<rusqlite::Connection>>> =
    std::sync::OnceLock::new();

static TOKIO_RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

static SYNC_HANDLE: std::sync::OnceLock<Mutex<Option<tokio::task::JoinHandle<()>>>> =
    std::sync::OnceLock::new();

fn conn() -> &'static Arc<Mutex<rusqlite::Connection>> {
    DB_CONN.get().expect("DB not initialized")
}

fn runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get().expect("Tokio runtime not initialized")
}

fn sync_handle() -> &'static Mutex<Option<tokio::task::JoinHandle<()>>> {
    SYNC_HANDLE.get_or_init(|| Mutex::new(None))
}

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[frb(sync)]
pub fn init_app(db_path: String) -> Result<(), String> {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("brewdio"),
    );

    let rt = tokio::runtime::Runtime::new().map_err(map_err)?;
    TOKIO_RT
        .set(rt)
        .map_err(|_| "Runtime already initialized")?;

    let connection =
        brewdio_persistence::connection_native::open(&db_path).map_err(map_err)?;
    DB_CONN
        .set(Arc::new(Mutex::new(connection)))
        .map_err(|_| "DB already initialized")?;

    log::info!("[mobile] initialized db at {}", db_path);
    Ok(())
}

// ---------------------------------------------------------------------------
// Bridge-only types (not in core crates)
// ---------------------------------------------------------------------------

/// Lightweight recipe info for list views.
#[frb]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub style_name: Option<String>,
    pub recipe_type: String,
    pub batch_size: VolumeType,
}

/// Lightweight batch info for list views.
#[frb]
pub struct BatchSummary {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub recipe_name: String,
    pub brew_date: u64,
    pub batch_size: VolumeType,
}

/// Calculated values for a recipe.
#[frb]
pub struct RecipeCalculations {
    pub og: f64,
    pub fg: f64,
    pub abv: f64,
    pub ibu: f64,
    pub srm: f64,
    pub color_hex: String,
}

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

pub fn list_recipes() -> Result<Vec<RecipeSummary>, String> {
    let c = conn().lock().map_err(map_err)?;
    let recipes = db::list_recipes(&*c).map_err(map_err)?;
    Ok(recipes
        .iter()
        .map(|r| RecipeSummary {
            id: r.id.clone(),
            name: r.name.clone(),
            style_name: r.recipe.style.as_ref().map(|s| s.name.clone()),
            recipe_type: format!("{:?}", r.recipe.type_),
            batch_size: r.recipe.batch_size.clone(),
        })
        .collect())
}

pub fn get_recipe(id: String) -> Result<Option<Recipe>, String> {
    let c = conn().lock().map_err(map_err)?;
    db::get_recipe(&*c, &id).map_err(map_err)
}

pub fn create_recipe(name: String, recipe: RecipeType) -> Result<String, String> {
    let c = conn().lock().map_err(map_err)?;
    let row = db::create_recipe(&*c, &name, &recipe, None, None).map_err(map_err)?;
    Ok(row.id)
}

pub fn update_recipe(
    id: String,
    name: String,
    recipe: RecipeType,
    equipment: Option<EquipmentType>,
) -> Result<(), String> {
    let c = conn().lock().map_err(map_err)?;
    db::update_recipe(&*c, &id, &name, &recipe, equipment.as_ref()).map_err(map_err)
}

pub fn set_recipe_equipment(
    id: String,
    equipment: Option<EquipmentType>,
    equipment_id: Option<String>,
) -> Result<(), String> {
    let c = conn().lock().map_err(map_err)?;
    db::set_recipe_equipment(&*c, &id, equipment.as_ref(), equipment_id.as_deref())
        .map_err(map_err)
}

pub fn delete_recipe(id: String) -> Result<(), String> {
    let c = conn().lock().map_err(map_err)?;
    db::delete_recipe(&*c, &id).map_err(map_err)
}

// ---------------------------------------------------------------------------
// Batches
// ---------------------------------------------------------------------------

pub fn list_batches() -> Result<Vec<BatchSummary>, String> {
    let c = conn().lock().map_err(map_err)?;
    let batches = batch::list_batches(&*c).map_err(map_err)?;
    Ok(batches
        .iter()
        .map(|b| BatchSummary {
            id: b.id.clone(),
            name: b.name.clone(),
            recipe_id: b.recipe_id.clone(),
            recipe_name: b.data.recipe.name.clone(),
            brew_date: b.data.brew_date,
            batch_size: b.data.recipe.batch_size.clone(),
        })
        .collect())
}

pub fn get_batch(id: String) -> Result<Option<Batch>, String> {
    let c = conn().lock().map_err(map_err)?;
    batch::get_batch(&*c, &id).map_err(map_err)
}

pub fn create_batch_from_recipe(name: String, recipe_id: String) -> Result<String, String> {
    let c = conn().lock().map_err(map_err)?;
    let recipe = db::get_recipe(&*c, &recipe_id)
        .map_err(map_err)?
        .ok_or("Recipe not found")?;

    let db_profiles = equipment_profile::list_equipment_profiles(&*c).map_err(map_err)?;
    let all = merge_equipment_profiles(&db_profiles);
    let (equipment, eq_id) = resolve_equipment(&recipe, &all);

    let row = batch::create_batch_from_recipe(
        &*c,
        &name,
        &recipe_id,
        &recipe.recipe,
        &equipment,
        &eq_id,
    )
    .map_err(map_err)?;
    Ok(row.id)
}

pub fn update_batch(id: String, name: String, data: BatchData) -> Result<(), String> {
    let data_json = serde_json::to_string(&data).map_err(map_err)?;
    let c = conn().lock().map_err(map_err)?;
    batch::update_batch(&*c, &id, &name, &data_json).map_err(map_err)
}

pub fn delete_batch(id: String) -> Result<(), String> {
    let c = conn().lock().map_err(map_err)?;
    batch::delete_batch(&*c, &id).map_err(map_err)
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub fn get_settings() -> Result<settings::SettingsDocument, String> {
    let c = conn().lock().map_err(map_err)?;
    settings::get_settings_document(&*c).map_err(map_err)
}

pub fn save_settings(doc: settings::SettingsDocument) -> Result<(), String> {
    let c = conn().lock().map_err(map_err)?;
    settings::save_settings(&*c, &doc).map_err(map_err)
}

// ---------------------------------------------------------------------------
// Equipment Profiles
// ---------------------------------------------------------------------------

/// Merge DB profiles with built-in defaults.
/// TODO: lift this into brewdio-persistence so all platforms share the logic.
fn merge_equipment_profiles(db_profiles: &[EquipmentProfile]) -> Vec<EquipmentProfile> {
    let mut result: Vec<EquipmentProfile> = db_profiles.to_vec();
    for ep in brewdio_core::data::equipment() {
        if !result.iter().any(|r| r.id == ep.id) {
            result.push(ep);
        }
    }
    result
}

fn resolve_equipment(
    recipe: &Recipe,
    all_profiles: &[EquipmentProfile],
) -> (EquipmentType, String) {
    if let Some(eid) = &recipe.equipment_id {
        if let Some(p) = all_profiles.iter().find(|p| p.id == *eid) {
            return (p.equipment.clone(), p.id.clone());
        }
    }
    let default = &brewdio_core::data::equipment()[0];
    (default.equipment.clone(), default.id.clone())
}

pub fn list_equipment_profiles() -> Result<Vec<EquipmentProfile>, String> {
    let c = conn().lock().map_err(map_err)?;
    let db_profiles = equipment_profile::list_equipment_profiles(&*c).map_err(map_err)?;
    Ok(merge_equipment_profiles(&db_profiles))
}

pub fn get_equipment_profile(id: String) -> Result<Option<EquipmentProfile>, String> {
    for ep in brewdio_core::data::equipment() {
        if ep.id == id {
            return Ok(Some(ep));
        }
    }
    let c = conn().lock().map_err(map_err)?;
    equipment_profile::get_equipment_profile(&*c, &id).map_err(map_err)
}

// ---------------------------------------------------------------------------
// Calculations
// ---------------------------------------------------------------------------

pub fn calculate_recipe(recipe: RecipeType) -> RecipeCalculations {
    let og = brewdio_core::og::calculate_og(
        &recipe.ingredients.fermentable_additions,
        &recipe.batch_size,
        &recipe.efficiency.brewhouse,
    );
    let fg = brewdio_core::fg::calculate_fg(og, &recipe.ingredients.culture_additions);
    let abv = brewdio_core::abv::calculate_abv(og, fg);
    let ibu = brewdio_core::ibu::calculate_ibu(
        &recipe.ingredients.hop_additions,
        &recipe.batch_size,
        og,
    );
    let srm = brewdio_core::color::calculate_color(
        &recipe.ingredients.fermentable_additions,
        &recipe.batch_size,
    );
    let rgb = brewdio_core::olfarve::srm_to_srgb(srm, None);
    let color_hex = brewdio_core::olfarve::rgb_to_hex(&rgb);

    RecipeCalculations {
        og,
        fg,
        abv,
        ibu,
        srm,
        color_hex,
    }
}

// ---------------------------------------------------------------------------
// Reference Data
// ---------------------------------------------------------------------------

pub fn get_fermentable_catalog() -> Vec<FermentableType> {
    brewdio_core::data::fermentables()
}

pub fn get_hop_catalog() -> Vec<HopVarietyBase> {
    brewdio_core::data::hops()
}

pub fn get_culture_catalog() -> Vec<CultureInformation> {
    brewdio_core::data::cultures()
}

pub fn get_style_catalog() -> Vec<StyleType> {
    brewdio_core::data::styles()
}

// ---------------------------------------------------------------------------
// Default recipe template
// ---------------------------------------------------------------------------

pub fn default_recipe(name: String, author: String) -> RecipeType {
    RecipeType {
        name,
        author,
        type_: RecipeTypeType::AllGrain,
        batch_size: VolumeType {
            unit: VolumeUnitType::Gal,
            value: 5.0,
        },
        efficiency: EfficiencyType {
            brewhouse: PercentType {
                unit: PercentUnitType::X,
                value: 72.0,
            },
            conversion: None,
            lauter: None,
            mash: None,
        },
        ingredients: IngredientsType {
            fermentable_additions: Vec::new(),
            hop_additions: Vec::new(),
            culture_additions: Vec::new(),
            miscellaneous_additions: Vec::new(),
            water_additions: Vec::new(),
        },
        alcohol_by_volume: None,
        apparent_attenuation: None,
        beer_p_h: None,
        boil: None,
        calories_per_pint: None,
        carbonation: None,
        coauthor: None,
        color_estimate: None,
        created: None,
        fermentation: None,
        final_gravity: None,
        ibu_estimate: None,
        mash: None,
        notes: None,
        original_gravity: None,
        packaging: None,
        style: None,
        taste: None,
    }
}

// ---------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------

pub fn start_sync(url: String) -> Result<(), String> {
    let conn_arc = conn().clone();
    let rt = runtime();

    {
        let mut handle = sync_handle().lock().map_err(map_err)?;
        if let Some(h) = handle.take() {
            h.abort();
        }
    }

    let join_handle = rt.spawn(async move {
        loop {
            let result = sync_worker::run_sync(
                &*conn_arc,
                &url,
                |status| {
                    let s = match &status {
                        sync_worker::SyncStatus::Connected => "connected",
                        sync_worker::SyncStatus::Disconnected => "disconnected",
                        sync_worker::SyncStatus::VersionMismatch { .. } => "version_mismatch",
                    };
                    log::info!("[mobile-sync] status: {}", s);
                },
                |doc_type| {
                    log::info!("[mobile-sync] change: {:?}", doc_type);
                },
            )
            .await;

            if let Err(e) = result {
                log::warn!("[mobile-sync] error: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });

    let mut handle = sync_handle().lock().map_err(map_err)?;
    *handle = Some(join_handle);
    Ok(())
}

pub fn stop_sync() -> Result<(), String> {
    let mut handle = sync_handle().lock().map_err(map_err)?;
    if let Some(h) = handle.take() {
        h.abort();
    }
    Ok(())
}
