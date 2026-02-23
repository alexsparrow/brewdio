use brewdio_core::beerjson_types::{
    EfficiencyType, FermentableAdditionType, FermentableAdditionTypeAmount,
    FermentableAdditionTypeType, FermentableTypeType, IngredientsType, MassType, MassUnitType,
    PercentType, PercentUnitType, RecipeStyleType, RecipeType, RecipeTypeType, VolumeType,
    VolumeUnitType,
};
use persistence::batch;
use persistence::db;
use persistence::recipe::RecipeDocument;
use persistence::settings;
use rusqlite::Connection;
use serde_json::Value as JsonValue;

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
    pub conn: Connection,
    pub screen: Screen,
    // Home tab
    pub home_tab: HomeTab,
    // Recipe list state
    pub recipes: Vec<RecipeListItem>,
    pub list_index: usize,
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
    pub batch_size_dialog: Option<BatchSizeDialog>,
    pub should_quit: bool,
}

impl App {
    pub fn new(conn: Connection) -> Self {
        let mut app = App {
            conn,
            screen: Screen::Home,
            home_tab: HomeTab::Recipes,
            recipes: Vec::new(),
            list_index: 0,
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
            batch_size_dialog: None,
            should_quit: false,
        };
        app.refresh_recipes();
        app.refresh_batches();
        app.refresh_settings();
        app
    }

    pub fn refresh_recipes(&mut self) {
        if let Ok(rows) = db::list_recipes(&self.conn) {
            self.recipes = rows
                .into_iter()
                .map(|r| RecipeListItem {
                    id: r.id,
                    name: r.name,
                })
                .collect();
        }
    }

    pub fn refresh_batches(&mut self) {
        if let Ok(rows) = batch::list_batches(&self.conn) {
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
        let saved: serde_json::Map<String, JsonValue> = settings::get_settings(&self.conn)
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
        let _ = settings::save_settings(&self.conn, &json);
    }

    pub fn create_recipe(&mut self) {
        let recipe = default_recipe();
        if let Ok(row) = db::create_recipe(&self.conn, "New Recipe", &recipe) {
            let id = row.id.clone();
            self.refresh_recipes();
            self.open_recipe(&id);
        }
    }

    pub fn delete_selected(&mut self) {
        if self.recipes.is_empty() {
            return;
        }
        let id = self.recipes[self.list_index].id.clone();
        let _ = db::delete_recipe(&self.conn, &id);
        self.refresh_recipes();
        if self.list_index >= self.recipes.len() && !self.recipes.is_empty() {
            self.list_index = self.recipes.len() - 1;
        }
    }

    pub fn delete_selected_batch(&mut self) {
        if self.batches.is_empty() {
            return;
        }
        let id = self.batches[self.batch_list_index].id.clone();
        let _ = batch::delete_batch(&self.conn, &id);
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
                    &self.conn,
                    &name,
                    recipe_id,
                    &doc.recipe,
                    equip,
                );
            }
        }
    }

    pub fn open_recipe(&mut self, id: &str) {
        if let Ok(Some(row)) = db::get_recipe(&self.conn, id) {
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
            let _ = db::update_recipe(&self.conn, recipe_id, &doc.name, &doc.recipe);
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
