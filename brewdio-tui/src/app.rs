use brewdio_core::beerjson_types::{
    CultureAdditionType, CultureAdditionTypeAmount, CultureAdditionTypeForm,
    CultureAdditionTypeType, CultureInformationForm, CultureInformationType,
    EfficiencyType, FermentableAdditionType, FermentableAdditionTypeAmount,
    FermentableAdditionTypeType, FermentableTypeType, HopAdditionType, HopAdditionTypeAmount,
    IngredientsType, MassType, MassUnitType, PercentType, PercentUnitType, RecipeStyleType,
    RecipeType, RecipeTypeType, TimeType, TimeUnitType, TimingType, UnitType, UnitUnitType,
    UseType, VolumeType, VolumeUnitType,
};
use brewdio_persistence::batch;
use brewdio_persistence::db;
use brewdio_persistence::recipe::RecipeDocument;
use brewdio_persistence::settings;
use rusqlite::Connection;
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};

use brewdio_core::units;

use crate::search_selector::{SearchItem, SearchSelector};
use crate::styles;

pub const MASS_UNITS: [MassUnitType; 4] = [MassUnitType::Lb, MassUnitType::Oz, MassUnitType::G, MassUnitType::Kg];
pub const VOLUME_UNITS: [VolumeUnitType; 3] = [VolumeUnitType::Gal, VolumeUnitType::L, VolumeUnitType::Bbl];

#[derive(Debug, Clone, PartialEq)]
pub enum FermentableDialogStep {
    SelectFermentable,
    EnterAmount,
    SelectUnit,
}

pub struct FermentableDialog {
    pub step: FermentableDialogStep,
    pub selector: SearchSelector,
    pub selected_fermentable_index: usize,
    pub amount_input: String,
    pub unit_index: usize,
    pub editing_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchSizeDialogStep {
    EnterValue,
    SelectUnit,
}

pub struct BatchSizeDialog {
    pub step: BatchSizeDialogStep,
    pub value_input: String,
    pub unit_index: usize,
}

pub const USE_TYPES: [UseType; 4] = [
    UseType::AddToBoil,
    UseType::AddToMash,
    UseType::AddToFermentation,
    UseType::AddToPackage,
];

#[derive(Debug, Clone, PartialEq)]
pub enum HopDialogStep {
    SelectHop,
    EnterAmount,
    SelectUnit,
    SelectUse,
    EnterTime,
}

pub struct HopDialog {
    pub step: HopDialogStep,
    pub selector: SearchSelector,
    pub selected_hop_index: usize,
    pub amount_input: String,
    pub unit_index: usize,
    pub use_index: usize,
    pub time_input: String,
    pub editing_index: Option<usize>,
}

pub const CULTURE_UNITS: [UnitUnitType; 3] = [UnitUnitType::Pkg, UnitUnitType::Each, UnitUnitType::Unit];

#[derive(Debug, Clone, PartialEq)]
pub enum CultureDialogStep {
    SelectCulture,
    EnterAmount,
    SelectUnit,
}

pub struct CultureDialog {
    pub step: CultureDialogStep,
    pub selector: SearchSelector,
    pub selected_culture_index: usize,
    pub amount_input: String,
    pub unit_index: usize,
    pub editing_index: Option<usize>,
}

fn culture_info_form_to_addition_form(f: &CultureInformationForm) -> CultureAdditionTypeForm {
    match f {
        CultureInformationForm::Liquid => CultureAdditionTypeForm::Liquid,
        CultureInformationForm::Dry => CultureAdditionTypeForm::Dry,
        CultureInformationForm::Slant => CultureAdditionTypeForm::Slant,
        CultureInformationForm::Culture => CultureAdditionTypeForm::Culture,
        CultureInformationForm::Dregs => CultureAdditionTypeForm::Dregs,
    }
}

fn culture_info_type_to_addition_type(t: &CultureInformationType) -> CultureAdditionTypeType {
    match t {
        CultureInformationType::Ale => CultureAdditionTypeType::Ale,
        CultureInformationType::Bacteria => CultureAdditionTypeType::Bacteria,
        CultureInformationType::Brett => CultureAdditionTypeType::Brett,
        CultureInformationType::Champagne => CultureAdditionTypeType::Champagne,
        CultureInformationType::Kveik => CultureAdditionTypeType::Kveik,
        CultureInformationType::Lacto => CultureAdditionTypeType::Lacto,
        CultureInformationType::Lager => CultureAdditionTypeType::Lager,
        CultureInformationType::Malolactic => CultureAdditionTypeType::Malolactic,
        CultureInformationType::MixedCulture => CultureAdditionTypeType::MixedCulture,
        CultureInformationType::Other => CultureAdditionTypeType::Other,
        CultureInformationType::Pedio => CultureAdditionTypeType::Pedio,
        CultureInformationType::Spontaneous => CultureAdditionTypeType::Spontaneous,
        CultureInformationType::Wine => CultureAdditionTypeType::Wine,
    }
}

pub fn use_type_label(u: &UseType) -> &'static str {
    match u {
        UseType::AddToBoil => "Boil",
        UseType::AddToMash => "Mash",
        UseType::AddToFermentation => "Fermentation",
        UseType::AddToPackage => "Package",
    }
}

pub fn format_hop_timing(timing: &TimingType) -> String {
    let use_label = timing
        .use_
        .as_ref()
        .map(|u| use_type_label(u))
        .unwrap_or("Boil");
    if let Some(ref dur) = timing.time {
        let unit_str = match dur.unit {
            TimeUnitType::Min => "min",
            TimeUnitType::Hr => "hr",
            TimeUnitType::Day => "day",
            TimeUnitType::Week => "wk",
            TimeUnitType::Sec => "sec",
        };
        format!("{} @ {} {}", use_label, dur.value, unit_str)
    } else {
        use_label.to_string()
    }
}

fn fermentable_type_to_addition_type(t: &FermentableTypeType) -> FermentableAdditionTypeType {
    match t {
        FermentableTypeType::Grain => FermentableAdditionTypeType::Grain,
        FermentableTypeType::Sugar => FermentableAdditionTypeType::Sugar,
        FermentableTypeType::Extract => FermentableAdditionTypeType::Extract,
        FermentableTypeType::DryExtract => FermentableAdditionTypeType::DryExtract,
        FermentableTypeType::Fruit => FermentableAdditionTypeType::Fruit,
        FermentableTypeType::Juice => FermentableAdditionTypeType::Juice,
        FermentableTypeType::Honey => FermentableAdditionTypeType::Honey,
        FermentableTypeType::Other => FermentableAdditionTypeType::Other,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    RecipeEdit { recipe_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HomeTab {
    Recipes,
    Batches,
    Settings,
}

const ALL_HOME_TABS: [HomeTab; 3] = [HomeTab::Recipes, HomeTab::Batches, HomeTab::Settings];

impl HomeTab {
    pub fn label(&self) -> &'static str {
        match self {
            HomeTab::Recipes => "Recipes",
            HomeTab::Batches => "Batches",
            HomeTab::Settings => "Settings",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            HomeTab::Recipes => 0,
            HomeTab::Batches => 1,
            HomeTab::Settings => 2,
        }
    }

    pub fn next(&self) -> HomeTab {
        ALL_HOME_TABS[(self.index() + 1) % ALL_HOME_TABS.len()]
    }

    pub fn prev(&self) -> HomeTab {
        ALL_HOME_TABS[(self.index() + ALL_HOME_TABS.len() - 1) % ALL_HOME_TABS.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum EditFocus {
    Name,
    Style,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Fermentables,
    Hops,
    Cultures,
    Water,
    Mash,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Fermentables => "Fermentables",
            Tab::Hops => "Hops",
            Tab::Cultures => "Cultures",
            Tab::Water => "Water",
            Tab::Mash => "Mash",
        }
    }

    #[allow(dead_code)]
    pub fn index(&self) -> usize {
        match self {
            Tab::Fermentables => 0,
            Tab::Hops => 1,
            Tab::Cultures => 2,
            Tab::Water => 3,
            Tab::Mash => 4,
        }
    }
}

const ALL_TABS: [Tab; 5] = [
    Tab::Fermentables,
    Tab::Hops,
    Tab::Cultures,
    Tab::Water,
    Tab::Mash,
];

pub struct StyleRange {
    pub min: f64,
    pub max: f64,
}

pub struct VitalDisplay {
    pub label: &'static str,
    pub value: f64,
    pub formatted: String,
    pub style_range: Option<StyleRange>,
    pub normal_min: f64,
    pub normal_max: f64,
}

pub struct RecipeListItem {
    pub id: String,
    pub name: String,
    pub style: String,
    pub is_deleted: bool,
}

pub struct BatchListItem {
    pub id: String,
    pub name: String,
    pub recipe_name: String,
    pub brew_date: String,
}

pub struct SettingsEntry {
    pub key: String,
    pub value: String,
}

const DEFAULT_SETTINGS: &[(&str, &str)] = &[
    ("vimMode", "false"),
    ("defaultVolumeUnit", "gal"),
    ("defaultMassUnit", "lb"),
    ("defaultTemperatureUnit", "F"),
    ("openaiApiKey", ""),
    ("defaultAuthor", "Brewdio User"),
];

pub struct App {
    pub conn: Arc<Mutex<Connection>>,
    pub sync_connected: Option<Arc<std::sync::atomic::AtomicBool>>,
    pub screen: Screen,
    // Home tab
    pub home_tab: HomeTab,
    // Recipe list state
    pub recipes: Vec<RecipeListItem>,
    pub list_index: usize,
    pub show_deleted: bool,
    pub confirm_delete: Option<(String, String)>, // (recipe_id, recipe_name)
    // Batch list state
    pub batches: Vec<BatchListItem>,
    pub batch_list_index: usize,
    // Settings state
    pub settings_entries: Vec<SettingsEntry>,
    pub settings_index: usize,
    pub editing_setting: bool,
    pub setting_input: String,
    // Recipe edit state
    pub current_doc: Option<RecipeDocument>,
    pub edit_focus: EditFocus,
    pub active_tab: Tab,
    pub name_input: String,
    pub editing_name: bool,
    pub style_selector: Option<SearchSelector>,
    pub fermentable_list_index: usize,
    pub fermentable_dialog: Option<FermentableDialog>,
    pub hop_list_index: usize,
    pub hop_dialog: Option<HopDialog>,
    pub culture_list_index: usize,
    pub culture_dialog: Option<CultureDialog>,
    pub batch_size_dialog: Option<BatchSizeDialog>,
    pub should_quit: bool,
}

impl App {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let mut app = App {
            conn,
            sync_connected: None,
            screen: Screen::Home,
            home_tab: HomeTab::Recipes,
            recipes: Vec::new(),
            list_index: 0,
            show_deleted: false,
            confirm_delete: None,
            batches: Vec::new(),
            batch_list_index: 0,
            settings_entries: Vec::new(),
            settings_index: 0,
            editing_setting: false,
            setting_input: String::new(),
            current_doc: None,
            edit_focus: EditFocus::Name,
            active_tab: Tab::Fermentables,
            name_input: String::new(),
            editing_name: false,
            style_selector: None,
            fermentable_list_index: 0,
            fermentable_dialog: None,
            hop_list_index: 0,
            hop_dialog: None,
            culture_list_index: 0,
            culture_dialog: None,
            batch_size_dialog: None,
            should_quit: false,
        };
        app.refresh_recipes();
        app.refresh_batches();
        app.refresh_settings();
        app
    }

    pub fn refresh_recipes(&mut self) {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let result = if self.show_deleted {
            db::list_all_recipes(&*conn)
        } else {
            db::list_recipes(&*conn)
        };
        if let Ok(rows) = result {
            self.recipes = rows
                .into_iter()
                .map(|r| {
                    let style = serde_json::from_str::<JsonValue>(&r.recipe)
                        .ok()
                        .and_then(|v| v.get("style")?.get("name")?.as_str().map(String::from))
                        .unwrap_or_default();
                    RecipeListItem {
                        id: r.id,
                        name: r.name,
                        style,
                        is_deleted: r.is_deleted,
                    }
                })
                .collect();
        }
    }

    pub fn refresh_batches(&mut self) {
        if let Ok(rows) = batch::list_batches(&*self.conn.lock().unwrap_or_else(|e| e.into_inner())) {
            self.batches = rows
                .into_iter()
                .map(|r| {
                    let (recipe_name, brew_date) = r
                        .to_data()
                        .map(|d| {
                            (d.recipe.name.clone(), format_epoch_millis(d.brew_date))
                        })
                        .unwrap_or_else(|_| ("(unknown)".to_string(), String::new()));
                    BatchListItem {
                        id: r.id,
                        name: r.name,
                        recipe_name,
                        brew_date,
                    }
                })
                .collect();
        }
    }

    pub fn refresh_settings(&mut self) {
        let saved: serde_json::Map<String, JsonValue> = settings::get_settings(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()))
            .ok()
            .flatten()
            .and_then(|row| serde_json::from_str(&row.data).ok())
            .unwrap_or_default();

        self.settings_entries = DEFAULT_SETTINGS
            .iter()
            .map(|(key, default)| {
                let value = saved
                    .get(*key)
                    .map(|v| match v {
                        JsonValue::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_else(|| default.to_string());
                SettingsEntry {
                    key: key.to_string(),
                    value,
                }
            })
            .collect();
    }

    pub fn save_setting_value(&mut self) {
        // Build JSON from current entries
        let mut map = serde_json::Map::new();
        for entry in &self.settings_entries {
            // Try to preserve booleans
            let val = if entry.value == "true" {
                JsonValue::Bool(true)
            } else if entry.value == "false" {
                JsonValue::Bool(false)
            } else {
                JsonValue::String(entry.value.clone())
            };
            map.insert(entry.key.clone(), val);
        }
        let json = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());
        let _ = settings::save_settings(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), &json);
    }

    pub fn create_recipe(&mut self) {
        let recipe = default_recipe();
        let result = db::create_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), "New Recipe", &recipe);
        if let Ok(row) = result {
            let id = row.id.clone();
            self.refresh_recipes();
            self.open_recipe(&id);
        }
    }

    pub fn prompt_delete_selected(&mut self) {
        if self.recipes.is_empty() {
            return;
        }
        let id = self.recipes[self.list_index].id.clone();
        let name = self.recipes[self.list_index].name.clone();
        self.confirm_delete = Some((id, name));
    }

    pub fn confirm_delete_yes(&mut self) {
        if let Some((ref id, _)) = self.confirm_delete {
            let _ = db::delete_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), id);
        }
        self.confirm_delete = None;
        self.refresh_recipes();
        if self.list_index >= self.recipes.len() && !self.recipes.is_empty() {
            self.list_index = self.recipes.len() - 1;
        }
    }

    pub fn confirm_delete_no(&mut self) {
        self.confirm_delete = None;
    }

    pub fn undelete_selected(&mut self) {
        if self.recipes.is_empty() || !self.show_deleted {
            return;
        }
        let id = self.recipes[self.list_index].id.clone();
        let _ = db::undelete_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), &id);
        self.refresh_recipes();
        if self.list_index >= self.recipes.len() && !self.recipes.is_empty() {
            self.list_index = self.recipes.len() - 1;
        }
    }

    pub fn toggle_show_deleted(&mut self) {
        self.show_deleted = !self.show_deleted;
        self.list_index = 0;
        self.refresh_recipes();
    }

    pub fn delete_selected_batch(&mut self) {
        if self.batches.is_empty() {
            return;
        }
        let id = self.batches[self.batch_list_index].id.clone();
        let _ = batch::delete_batch(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), &id);
        self.refresh_batches();
        if self.batch_list_index >= self.batches.len() && !self.batches.is_empty() {
            self.batch_list_index = self.batches.len() - 1;
        }
    }

    pub fn create_batch_from_current(&mut self) {
        if let (Screen::RecipeEdit { ref recipe_id }, Some(ref doc)) =
            (&self.screen, &self.current_doc)
        {
            let equipment_list = brewdio_core::data::equipment();
            if let Some(equip) = equipment_list.first() {
                let name = format!("{} — Batch", doc.name);
                let _ = batch::create_batch_from_recipe(
                    &*self.conn.lock().unwrap_or_else(|e| e.into_inner()),
                    &name,
                    recipe_id,
                    &doc.recipe,
                    equip,
                );
            }
        }
    }

    pub fn open_recipe(&mut self, id: &str) {
        if let Ok(Some(row)) = db::get_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), id) {
            if let Ok(doc) = row.to_document() {
                self.name_input = doc.name.clone();
                self.current_doc = Some(doc);
                self.screen = Screen::RecipeEdit {
                    recipe_id: id.to_string(),
                };
                self.editing_name = false;
                self.style_selector = None;
                self.edit_focus = EditFocus::Name;
                self.active_tab = Tab::Fermentables;
            }
        }
    }

    pub fn open_selected(&mut self) {
        if self.recipes.is_empty() {
            return;
        }
        let id = self.recipes[self.list_index].id.clone();
        self.open_recipe(&id);
    }

    pub fn save_current(&mut self) {
        if let (Screen::RecipeEdit { ref recipe_id }, Some(ref doc)) =
            (&self.screen, &self.current_doc)
        {
            let _ = db::update_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), recipe_id, &doc.name, &doc.recipe);
        }
    }

    pub fn back_to_list(&mut self) {
        self.save_current();
        self.screen = Screen::Home;
        self.current_doc = None;
        self.editing_name = false;
        self.style_selector = None;
        self.refresh_recipes();
        self.refresh_batches();
    }

    pub fn confirm_name(&mut self) {
        if let Some(ref mut doc) = self.current_doc {
            doc.name = self.name_input.clone();
            doc.recipe.name = self.name_input.clone();
        }
        self.editing_name = false;
        self.save_current();
    }

    pub fn cancel_name(&mut self) {
        if let Some(ref doc) = self.current_doc {
            self.name_input = doc.name.clone();
        }
        self.editing_name = false;
    }

    pub fn open_style_selector(&mut self) {
        let all = styles::all_styles();
        let items: Vec<SearchItem> = all
            .iter()
            .enumerate()
            .map(|(i, s)| SearchItem {
                label: s.name.clone(),
                detail: s.category.clone(),
                index: i,
            })
            .collect();
        let mut selector = SearchSelector::new("Select Style", items);
        // Pre-position cursor on current style
        if let Some(ref doc) = self.current_doc {
            if let Some(ref style) = doc.recipe.style {
                if let Some(pos) = all.iter().position(|s| s.name == style.0.name) {
                    selector.set_cursor_to_index(pos);
                }
            }
        }
        self.style_selector = Some(selector);
    }

    pub fn confirm_style(&mut self, idx: usize) {
        if let Some(ref mut doc) = self.current_doc {
            let all = styles::all_styles();
            let style = &all[idx];
            doc.recipe.style = Some(RecipeStyleType(styles::to_style_base(style)));
        }
        self.style_selector = None;
        self.save_current();
    }

    pub fn cancel_style(&mut self) {
        self.style_selector = None;
    }

    fn make_fermentable_selector() -> SearchSelector {
        let all = brewdio_core::data::fermentables();
        let items: Vec<SearchItem> = all
            .iter()
            .enumerate()
            .map(|(i, f)| SearchItem {
                label: f.name.clone(),
                detail: format!("{:?}", f.type_),
                index: i,
            })
            .collect();
        SearchSelector::new("Select Fermentable", items)
    }

    pub fn open_add_fermentable(&mut self) {
        self.fermentable_dialog = Some(FermentableDialog {
            step: FermentableDialogStep::SelectFermentable,
            selector: Self::make_fermentable_selector(),
            selected_fermentable_index: 0,
            amount_input: "1.0".to_string(),
            unit_index: 0,
            editing_index: None,
        });
    }

    pub fn open_edit_fermentable(&mut self) {
        let doc = match self.current_doc.as_ref() {
            Some(d) => d,
            None => return,
        };
        let additions = &doc.recipe.ingredients.fermentable_additions;
        if additions.is_empty() || self.fermentable_list_index >= additions.len() {
            return;
        }
        let addition = &additions[self.fermentable_list_index];

        let all = brewdio_core::data::fermentables();
        let mut selector = Self::make_fermentable_selector();
        // Try to find the matching fermentable by name
        let ferm_idx = all.iter().position(|f| f.name == addition.name).unwrap_or(0);
        selector.set_cursor_to_index(ferm_idx);

        let (amount_str, unit_idx) = match &addition.amount {
            FermentableAdditionTypeAmount::MassType(m) => {
                let ui = MASS_UNITS.iter().position(|u| *u == m.unit).unwrap_or(0);
                (format!("{}", m.value), ui)
            }
            FermentableAdditionTypeAmount::VolumeType(v) => {
                (format!("{}", v.value), 0)
            }
        };

        self.fermentable_dialog = Some(FermentableDialog {
            step: FermentableDialogStep::SelectFermentable,
            selector,
            selected_fermentable_index: ferm_idx,
            amount_input: amount_str,
            unit_index: unit_idx,
            editing_index: Some(self.fermentable_list_index),
        });
    }

    pub fn confirm_fermentable_dialog(&mut self) {
        let dialog = match self.fermentable_dialog.take() {
            Some(d) => d,
            None => return,
        };
        let amount_value: f64 = dialog.amount_input.parse().unwrap_or(1.0);
        let all = brewdio_core::data::fermentables();
        let ferm = &all[dialog.selected_fermentable_index];

        let addition = FermentableAdditionType {
            name: ferm.name.clone(),
            type_: fermentable_type_to_addition_type(&ferm.type_),
            color: ferm.color.clone(),
            yield_: ferm.yield_.clone(),
            producer: ferm.producer.clone(),
            amount: FermentableAdditionTypeAmount::MassType(MassType {
                unit: MASS_UNITS[dialog.unit_index],
                value: amount_value,
            }),
            grain_group: None,
            origin: ferm.origin.clone(),
            product_id: None,
            timing: None,
        };

        if let Some(ref mut doc) = self.current_doc {
            if let Some(idx) = dialog.editing_index {
                doc.recipe.ingredients.fermentable_additions[idx] = addition;
            } else {
                doc.recipe.ingredients.fermentable_additions.push(addition);
                self.fermentable_list_index =
                    doc.recipe.ingredients.fermentable_additions.len() - 1;
            }
        }
        self.save_current();
    }

    pub fn delete_selected_fermentable(&mut self) {
        let doc = match self.current_doc.as_mut() {
            Some(d) => d,
            None => return,
        };
        let additions = &mut doc.recipe.ingredients.fermentable_additions;
        if additions.is_empty() || self.fermentable_list_index >= additions.len() {
            return;
        }
        additions.remove(self.fermentable_list_index);
        if self.fermentable_list_index >= additions.len() && !additions.is_empty() {
            self.fermentable_list_index = additions.len() - 1;
        }
        self.save_current();
    }

    fn make_hop_selector() -> SearchSelector {
        let all = brewdio_core::data::hops();
        let items: Vec<SearchItem> = all
            .iter()
            .enumerate()
            .map(|(i, h)| SearchItem {
                label: h.name.clone(),
                detail: h
                    .origin
                    .clone()
                    .unwrap_or_default(),
                index: i,
            })
            .collect();
        SearchSelector::new("Select Hop", items)
    }

    pub fn open_add_hop(&mut self) {
        self.hop_dialog = Some(HopDialog {
            step: HopDialogStep::SelectHop,
            selector: Self::make_hop_selector(),
            selected_hop_index: 0,
            amount_input: "1.0".to_string(),
            unit_index: 2, // default to grams
            use_index: 0,  // default to boil
            time_input: "60".to_string(),
            editing_index: None,
        });
    }

    pub fn open_edit_hop(&mut self) {
        let doc = match self.current_doc.as_ref() {
            Some(d) => d,
            None => return,
        };
        let additions = &doc.recipe.ingredients.hop_additions;
        if additions.is_empty() || self.hop_list_index >= additions.len() {
            return;
        }
        let addition = &additions[self.hop_list_index];

        let all = brewdio_core::data::hops();
        let mut selector = Self::make_hop_selector();
        let hop_idx = all.iter().position(|h| h.name == addition.name).unwrap_or(0);
        selector.set_cursor_to_index(hop_idx);

        let (amount_str, unit_idx) = match &addition.amount {
            HopAdditionTypeAmount::MassType(m) => {
                let ui = MASS_UNITS.iter().position(|u| *u == m.unit).unwrap_or(0);
                (format!("{}", m.value), ui)
            }
            HopAdditionTypeAmount::VolumeType(v) => (format!("{}", v.value), 0),
        };

        let use_idx = addition
            .timing
            .use_
            .as_ref()
            .and_then(|u| USE_TYPES.iter().position(|ut| ut == u))
            .unwrap_or(0);

        let time_str = addition
            .timing
            .time
            .as_ref()
            .map(|t| format!("{}", t.value))
            .unwrap_or_else(|| "60".to_string());

        self.hop_dialog = Some(HopDialog {
            step: HopDialogStep::SelectHop,
            selector,
            selected_hop_index: hop_idx,
            amount_input: amount_str,
            unit_index: unit_idx,
            use_index: use_idx,
            time_input: time_str,
            editing_index: Some(self.hop_list_index),
        });
    }

    pub fn confirm_hop_dialog(&mut self) {
        let dialog = match self.hop_dialog.take() {
            Some(d) => d,
            None => return,
        };
        let amount_value: f64 = dialog.amount_input.parse().unwrap_or(1.0);
        let time_value: i64 = dialog.time_input.parse().unwrap_or(60);
        let all = brewdio_core::data::hops();
        let hop = &all[dialog.selected_hop_index];

        let use_type = USE_TYPES[dialog.use_index].clone();
        let time_unit = match use_type {
            UseType::AddToFermentation => TimeUnitType::Day,
            _ => TimeUnitType::Min,
        };

        let addition = HopAdditionType {
            name: hop.name.clone(),
            alpha_acid: hop.alpha_acid.clone(),
            beta_acid: hop.beta_acid.clone(),
            form: None,
            origin: hop.origin.clone(),
            producer: hop.producer.clone(),
            product_id: hop.product_id.clone(),
            year: None,
            amount: HopAdditionTypeAmount::MassType(MassType {
                unit: MASS_UNITS[dialog.unit_index],
                value: amount_value,
            }),
            timing: TimingType {
                use_: Some(use_type),
                duration: None,
                time: Some(TimeType {
                    unit: time_unit,
                    value: time_value,
                }),
                continuous: None,
                p_h: None,
                specific_gravity: None,
                step: None,
            },
        };

        if let Some(ref mut doc) = self.current_doc {
            if let Some(idx) = dialog.editing_index {
                doc.recipe.ingredients.hop_additions[idx] = addition;
            } else {
                doc.recipe.ingredients.hop_additions.push(addition);
                self.hop_list_index = doc.recipe.ingredients.hop_additions.len() - 1;
            }
        }
        self.save_current();
    }

    pub fn delete_selected_hop(&mut self) {
        let doc = match self.current_doc.as_mut() {
            Some(d) => d,
            None => return,
        };
        let additions = &mut doc.recipe.ingredients.hop_additions;
        if additions.is_empty() || self.hop_list_index >= additions.len() {
            return;
        }
        additions.remove(self.hop_list_index);
        if self.hop_list_index >= additions.len() && !additions.is_empty() {
            self.hop_list_index = additions.len() - 1;
        }
        self.save_current();
    }

    fn make_culture_selector() -> SearchSelector {
        let all = brewdio_core::data::cultures();
        let items: Vec<SearchItem> = all
            .iter()
            .enumerate()
            .map(|(i, c)| SearchItem {
                label: c.name.clone(),
                detail: c.producer.clone().unwrap_or_default(),
                index: i,
            })
            .collect();
        SearchSelector::new("Select Culture", items)
    }

    pub fn open_add_culture(&mut self) {
        self.culture_dialog = Some(CultureDialog {
            step: CultureDialogStep::SelectCulture,
            selector: Self::make_culture_selector(),
            selected_culture_index: 0,
            amount_input: "1".to_string(),
            unit_index: 0, // default to pkg
            editing_index: None,
        });
    }

    pub fn open_edit_culture(&mut self) {
        let doc = match self.current_doc.as_ref() {
            Some(d) => d,
            None => return,
        };
        let additions = &doc.recipe.ingredients.culture_additions;
        if additions.is_empty() || self.culture_list_index >= additions.len() {
            return;
        }
        let addition = &additions[self.culture_list_index];

        let all = brewdio_core::data::cultures();
        let mut selector = Self::make_culture_selector();
        let culture_idx = all.iter().position(|c| c.name == addition.name).unwrap_or(0);
        selector.set_cursor_to_index(culture_idx);

        let (amount_str, unit_idx) = match &addition.amount {
            CultureAdditionTypeAmount::UnitType(u) => {
                let ui = CULTURE_UNITS.iter().position(|cu| *cu == u.unit).unwrap_or(0);
                (format!("{}", u.value), ui)
            }
            CultureAdditionTypeAmount::MassType(m) => (format!("{}", m.value), 0),
            CultureAdditionTypeAmount::VolumeType(v) => (format!("{}", v.value), 0),
        };

        self.culture_dialog = Some(CultureDialog {
            step: CultureDialogStep::SelectCulture,
            selector,
            selected_culture_index: culture_idx,
            amount_input: amount_str,
            unit_index: unit_idx,
            editing_index: Some(self.culture_list_index),
        });
    }

    pub fn confirm_culture_dialog(&mut self) {
        let dialog = match self.culture_dialog.take() {
            Some(d) => d,
            None => return,
        };
        let amount_value: f64 = dialog.amount_input.parse().unwrap_or(1.0);
        let all = brewdio_core::data::cultures();
        let culture = &all[dialog.selected_culture_index];

        let attenuation = culture.attenuation_range.as_ref().map(|r| r.minimum.clone());

        let addition = CultureAdditionType {
            name: culture.name.clone(),
            type_: culture_info_type_to_addition_type(&culture.type_),
            form: culture_info_form_to_addition_form(&culture.form),
            producer: culture.producer.clone(),
            product_id: culture.product_id.clone(),
            amount: CultureAdditionTypeAmount::UnitType(UnitType {
                unit: CULTURE_UNITS[dialog.unit_index],
                value: amount_value,
            }),
            attenuation,
            cell_count_billions: None,
            times_cultured: None,
            timing: None,
        };

        if let Some(ref mut doc) = self.current_doc {
            if let Some(idx) = dialog.editing_index {
                doc.recipe.ingredients.culture_additions[idx] = addition;
            } else {
                doc.recipe.ingredients.culture_additions.push(addition);
                self.culture_list_index =
                    doc.recipe.ingredients.culture_additions.len() - 1;
            }
        }
        self.save_current();
    }

    pub fn delete_selected_culture(&mut self) {
        let doc = match self.current_doc.as_mut() {
            Some(d) => d,
            None => return,
        };
        let additions = &mut doc.recipe.ingredients.culture_additions;
        if additions.is_empty() || self.culture_list_index >= additions.len() {
            return;
        }
        additions.remove(self.culture_list_index);
        if self.culture_list_index >= additions.len() && !additions.is_empty() {
            self.culture_list_index = additions.len() - 1;
        }
        self.save_current();
    }

    pub fn open_batch_size_dialog(&mut self) {
        let (value_str, unit_idx) = match self.current_doc.as_ref() {
            Some(doc) => {
                let bs = &doc.recipe.batch_size;
                let ui = VOLUME_UNITS.iter().position(|u| *u == bs.unit).unwrap_or(0);
                (format!("{}", bs.value), ui)
            }
            None => ("5".to_string(), 0),
        };
        self.batch_size_dialog = Some(BatchSizeDialog {
            step: BatchSizeDialogStep::EnterValue,
            value_input: value_str,
            unit_index: unit_idx,
        });
    }

    pub fn confirm_batch_size_dialog(&mut self) {
        let dialog = match self.batch_size_dialog.take() {
            Some(d) => d,
            None => return,
        };
        let value: f64 = dialog.value_input.parse().unwrap_or(5.0);
        if let Some(ref mut doc) = self.current_doc {
            doc.recipe.batch_size = VolumeType {
                unit: VOLUME_UNITS[dialog.unit_index],
                value,
            };
        }
        self.save_current();
    }

    pub fn cancel_batch_size_dialog(&mut self) {
        self.batch_size_dialog = None;
    }

    pub fn batch_size_display(&self) -> String {
        match self.current_doc.as_ref() {
            Some(doc) => {
                let bs = &doc.recipe.batch_size;
                let unit_str = format!("{:?}", bs.unit).to_lowercase();
                format!("{} {}", format_amount(bs.value), unit_str)
            }
            None => "(none)".to_string(),
        }
    }

    pub fn set_tab(&mut self, n: usize) {
        if n < ALL_TABS.len() {
            self.active_tab = ALL_TABS[n];
        }
    }

    pub fn compute_vitals(&self) -> Vec<VitalDisplay> {
        let doc = match self.current_doc.as_ref() {
            Some(d) => d,
            None => return Vec::new(),
        };
        let recipe = &doc.recipe;

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

        let style = brewdio_core::data::style_for_recipe(recipe);

        let og_range = style.as_ref().and_then(|s| {
            s.original_gravity.as_ref().map(|r| StyleRange {
                min: units::gravity_to_sg(&r.minimum),
                max: units::gravity_to_sg(&r.maximum),
            })
        });
        let fg_range = style.as_ref().and_then(|s| {
            s.final_gravity.as_ref().map(|r| StyleRange {
                min: units::gravity_to_sg(&r.minimum),
                max: units::gravity_to_sg(&r.maximum),
            })
        });
        let ibu_range = style.as_ref().and_then(|s| {
            s.international_bitterness_units.as_ref().map(|r| StyleRange {
                min: r.minimum.value,
                max: r.maximum.value,
            })
        });
        let srm_range = style.as_ref().and_then(|s| {
            s.color.as_ref().map(|r| StyleRange {
                min: units::color_to_srm(&r.minimum),
                max: units::color_to_srm(&r.maximum),
            })
        });
        let abv_range = style.as_ref().and_then(|s| {
            s.alcohol_by_volume.as_ref().map(|r| StyleRange {
                min: r.minimum.value,
                max: r.maximum.value,
            })
        });

        vec![
            VitalDisplay {
                label: "OG",
                value: og,
                formatted: format!("{:.3}", og),
                style_range: og_range,
                normal_min: 1.030,
                normal_max: 1.120,
            },
            VitalDisplay {
                label: "FG",
                value: fg,
                formatted: format!("{:.3}", fg),
                style_range: fg_range,
                normal_min: 1.000,
                normal_max: 1.040,
            },
            VitalDisplay {
                label: "IBU",
                value: ibu,
                formatted: format!("{:.0}", ibu),
                style_range: ibu_range,
                normal_min: 0.0,
                normal_max: 120.0,
            },
            VitalDisplay {
                label: "SRM",
                value: srm,
                formatted: format!("{:.0}", srm),
                style_range: srm_range,
                normal_min: 1.0,
                normal_max: 40.0,
            },
            VitalDisplay {
                label: "ABV",
                value: abv,
                formatted: format!("{:.1}%", abv),
                style_range: abv_range,
                normal_min: 2.0,
                normal_max: 14.0,
            },
        ]
    }

    pub fn style_name(&self) -> String {
        self.current_doc
            .as_ref()
            .and_then(|d| d.recipe.style.as_ref())
            .map(|s| s.0.name.clone())
            .unwrap_or_else(|| "(none)".to_string())
    }
}

fn format_epoch_millis(millis: u64) -> String {
    let secs = (millis / 1000) as i64;
    let days_since_epoch = secs / 86400;
    // Simple date formatting: calculate year/month/day from days since epoch
    let mut days = days_since_epoch;
    let mut year = 1970i64;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = i + 1;
            break;
        }
        days -= md;
    }
    let day = days + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn format_amount(value: f64) -> String {
    if value == value.floor() {
        format!("{:.0}", value)
    } else {
        let s = format!("{:.2}", value);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn default_recipe() -> RecipeType {
    RecipeType {
        name: "New Recipe".to_string(),
        author: String::new(),
        type_: RecipeTypeType::AllGrain,
        batch_size: VolumeType {
            unit: VolumeUnitType::L,
            value: 20.0,
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
