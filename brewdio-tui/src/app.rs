use brewdio_core::beerjson_types::{
    CultureAdditionType, CultureAdditionTypeAmount, CultureAdditionTypeForm,
    CultureAdditionTypeType, CultureInformationForm, CultureInformationType,
    EfficiencyType, FermentableAdditionType, FermentableAdditionTypeAmount,
    FermentableAdditionTypeType, FermentableTypeType, HopAdditionType, HopAdditionTypeAmount,
    IngredientsType, MassType, MassUnitType, PercentType, PercentUnitType, RecipeStyleType,
    RecipeType, RecipeTypeType, TemperatureType, TemperatureUnitType, TimeType, TimeUnitType,
    TimingType, UnitType, UnitUnitType, UseType, VolumeType, VolumeUnitType,
};
use brewdio_core::water::{
    WaterCalculatorInput, WaterCalculatorOptions, WaterCalculatorResult,
};
use brewdio_persistence::automerge::HistoryEntry;
use brewdio_persistence::batch::{self, BatchData};
use brewdio_persistence::db;
use brewdio_persistence::recipe::Recipe;
use brewdio_persistence::settings;
use rusqlite::Connection;
use serde_json::Value as JsonValue;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use ratatui::style::{Color, Style};
use tui_textarea::TextArea;

use brewdio_core::units;

use crate::chat::{self, ChatEvent, ChatMessage, ChatRole, ChatState};
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
    BatchEdit { batch_id: String },
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
    Batches,
    History,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Fermentables => "Fermentables",
            Tab::Hops => "Hops",
            Tab::Cultures => "Cultures",
            Tab::Water => "Water",
            Tab::Mash => "Mash",
            Tab::Batches => "Batches",
            Tab::History => "History",
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
            Tab::Batches => 5,
            Tab::History => 6,
        }
    }
}

const ALL_TABS: [Tab; 7] = [
    Tab::Fermentables,
    Tab::Hops,
    Tab::Cultures,
    Tab::Water,
    Tab::Mash,
    Tab::Batches,
    Tab::History,
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

pub enum SettingEditState {
    /// Freeform text input (Text and Secret kinds)
    TextInput { input: String },
    /// Cycle through options (Bool, VolumeUnit, MassUnit, TemperatureUnit)
    Selector { options: Vec<String>, index: usize },
}

pub struct App {
    pub conn: Arc<Mutex<Connection>>,
    pub sync_connected: Option<Arc<std::sync::atomic::AtomicU8>>,
    pub sync_changed: Option<Arc<std::sync::atomic::AtomicBool>>,
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
    pub settings_doc: settings::SettingsDocument,
    pub settings_index: usize,
    pub setting_edit: Option<SettingEditState>,
    // Recipe edit state
    pub current_doc: Option<Recipe>,
    pub edit_focus: EditFocus,
    pub active_tab: Tab,
    pub name_input: String,
    pub editing_name: bool,
    pub style_selector: Option<SearchSelector>,
    pub equipment_selector: Option<SearchSelector>,
    pub confirm_equipment_idx: Option<usize>, // pending equipment change awaiting confirmation
    pub fermentable_list_index: usize,
    pub fermentable_dialog: Option<FermentableDialog>,
    pub hop_list_index: usize,
    pub hop_dialog: Option<HopDialog>,
    pub culture_list_index: usize,
    pub culture_dialog: Option<CultureDialog>,
    pub batch_size_dialog: Option<BatchSizeDialog>,
    // Batches tab in recipe edit
    pub recipe_batches: Vec<BatchListItem>,
    pub recipe_batch_list_index: usize,
    // Batch edit state
    pub current_batch_id: Option<String>,
    pub current_batch_data: Option<BatchData>,
    pub batch_notes_text: String,
    // Brew date editing (batch edit only)
    pub editing_brew_date: bool,
    pub brew_date_input: String,
    // Notes editor (shared popup for recipe and batch notes)
    pub notes_editor: Option<TextArea<'static>>,
    pub notes_target: NotesTarget,
    // History tab state
    pub history_entries: Vec<HistoryEntry>,
    pub history_scroll: usize,
    pub history_loading: bool,
    history_rx: Option<mpsc::Receiver<Vec<HistoryEntry>>>,
    // Chat state
    pub chat: Option<ChatState>,
    pub chat_visible: bool,
    pub chat_rx: Option<mpsc::Receiver<ChatEvent>>,
    pub runtime_handle: tokio::runtime::Handle,
    pub should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotesTarget {
    Recipe,
    Batch,
}

impl App {
    pub fn new(conn: Arc<Mutex<Connection>>, runtime_handle: tokio::runtime::Handle) -> Self {
        let mut app = App {
            conn,
            sync_connected: None,
            sync_changed: None,
            screen: Screen::Home,
            home_tab: HomeTab::Recipes,
            recipes: Vec::new(),
            list_index: 0,
            show_deleted: false,
            confirm_delete: None,
            batches: Vec::new(),
            batch_list_index: 0,
            settings_doc: settings::SettingsDocument::default(),
            settings_index: 0,
            setting_edit: None,
            current_doc: None,
            edit_focus: EditFocus::Name,
            active_tab: Tab::Fermentables,
            name_input: String::new(),
            editing_name: false,
            style_selector: None,
            equipment_selector: None,
            confirm_equipment_idx: None,
            fermentable_list_index: 0,
            fermentable_dialog: None,
            hop_list_index: 0,
            hop_dialog: None,
            culture_list_index: 0,
            culture_dialog: None,
            batch_size_dialog: None,
            recipe_batches: Vec::new(),
            recipe_batch_list_index: 0,
            current_batch_id: None,
            current_batch_data: None,
            batch_notes_text: String::new(),
            editing_brew_date: false,
            brew_date_input: String::new(),
            notes_editor: None,
            notes_target: NotesTarget::Recipe,
            history_entries: Vec::new(),
            history_scroll: 0,
            history_loading: false,
            history_rx: None,
            chat: None,
            chat_visible: false,
            chat_rx: None,
            runtime_handle,
            should_quit: false,
        };
        app.refresh_recipes();
        app.refresh_batches();
        app.refresh_settings();
        app
    }

    /// Check if the sync worker has applied remote changes and refresh from DB if so.
    pub fn poll_sync_changes(&mut self) {
        let changed = match self.sync_changed {
            Some(ref flag) => flag.swap(false, std::sync::atomic::Ordering::Relaxed),
            None => false,
        };
        if !changed {
            return;
        }
        self.refresh_recipes();
        self.refresh_batches();
        self.refresh_settings();
        // Reload the currently open recipe/batch from DB
        match self.screen.clone() {
            Screen::RecipeEdit { recipe_id } => {
                let recipe = {
                    let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
                    db::get_recipe(&*c, &recipe_id).ok().flatten()
                };
                if let Some(recipe) = recipe {
                    self.name_input = recipe.name.clone();
                    self.current_doc = Some(recipe);
                }
            }
            Screen::BatchEdit { ref batch_id } => {
                let b = {
                    let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
                    batch::get_batch(&*c, batch_id).ok().flatten()
                };
                if let Some(b) = b {
                    self.name_input = b.name.clone();
                    self.batch_notes_text = b.data.notes.clone().unwrap_or_default();
                    self.current_batch_data = Some(b.data);
                }
            }
            _ => {}
        }
    }

    /// Open the chat overlay. If viewing a recipe, includes recipe context.
    /// Reuses existing chat history if available.
    pub fn open_chat(&mut self) {
        let recipe_context = self.current_doc.as_ref().map(|doc| {
            (doc.id.clone(), doc.recipe.clone())
        });
        if let Some(ref mut chat) = self.chat {
            // Update recipe context for existing chat session
            chat.recipe_context = recipe_context;
            // Scroll to bottom on re-open
            chat.user_scrolled = false;
            chat.scroll_offset = 0;
        } else {
            self.chat = Some(ChatState::new(recipe_context));
        }
        self.chat_visible = true;
    }

    /// Close the chat overlay (hides it, preserves history).
    pub fn close_chat(&mut self) {
        self.chat_visible = false;
    }

    /// Submit the current chat input as a message and start streaming.
    pub fn submit_chat_message(&mut self) {
        let chat = match self.chat.as_mut() {
            Some(c) => c,
            None => return,
        };

        let input = chat.input_text();
        let input = input.trim().to_string();
        if input.is_empty() || chat.is_streaming {
            return;
        }

        // Check for API key
        let api_key = self.settings_doc.openai_api_key.clone();
        if api_key.is_empty() {
            return;
        }

        // Add user message
        chat.messages.push(ChatMessage {
            role: ChatRole::User,
            content: input.clone(),
            tool_calls: Vec::new(),
        });
        chat.clear_input();
        chat.is_streaming = true;
        chat.user_scrolled = false;

        // Clone state for async task
        let messages = chat.messages.clone();
        let recipe_context = chat.recipe_context.clone();
        let conn = self.conn.clone();
        let (tx, rx) = mpsc::channel();
        self.chat_rx = Some(rx);

        let base_url = self.settings_doc.ai_base_url.clone();
        let model = self.settings_doc.ai_model.clone();

        self.runtime_handle.spawn(async move {
            chat::client::stream_chat(api_key, base_url, model, messages, recipe_context, conn, tx).await;
        });
    }

    /// Poll for chat events from the streaming task. Called each frame.
    pub fn poll_chat(&mut self) {
        // Collect all pending events first to avoid borrow issues
        let events: Vec<ChatEvent> = {
            let rx = match self.chat_rx.as_ref() {
                Some(rx) => rx,
                None => return,
            };
            let mut events = Vec::new();
            loop {
                match rx.try_recv() {
                    Ok(event) => events.push(event),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if !events.is_empty() {
                            // Already collected events (likely Done/Error) — process them
                            break;
                        }
                        if let Some(chat) = self.chat.as_mut() {
                            if chat.is_streaming {
                                // Task dropped without sending Done/Error
                                chat.is_streaming = false;
                                chat.messages.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    content: "Error: Connection to AI lost unexpectedly.".to_string(),
                                    tool_calls: Vec::new(),
                                });
                            }
                        }
                        self.chat_rx = None;
                        return;
                    }
                }
            }
            events
        };

        let mut needs_refresh = false;

        for event in events {
            let chat = match self.chat.as_mut() {
                Some(c) => c,
                None => return,
            };

            match event {
                ChatEvent::TokenDelta(text) => {
                    // Append to the last assistant message, or create one
                    if let Some(last) = chat.messages.last_mut() {
                        if last.role == ChatRole::Assistant && last.tool_calls.is_empty() {
                            last.content.push_str(&text);
                            continue;
                        }
                    }
                    chat.messages.push(ChatMessage {
                        role: ChatRole::Assistant,
                        content: text,
                        tool_calls: Vec::new(),
                    });
                }
                ChatEvent::ToolCallStart { id, name, arguments } => {
                    let needs_new = chat.messages.last()
                        .map_or(true, |m| m.role != ChatRole::Assistant || (!m.content.is_empty() && m.tool_calls.is_empty()));

                    if needs_new {
                        chat.messages.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: String::new(),
                            tool_calls: Vec::new(),
                        });
                    }

                    if let Some(last) = chat.messages.last_mut() {
                        last.tool_calls.push(chat::ToolCallInfo {
                            id,
                            name,
                            arguments,
                            result: None,
                            error: None,
                            extra: serde_json::Map::new(),
                        });
                    }
                }
                ChatEvent::ToolCallResult { id, result } => {
                    for msg in chat.messages.iter_mut().rev() {
                        for tc in &mut msg.tool_calls {
                            if tc.id == id {
                                tc.result = Some(result.clone());
                                break;
                            }
                        }
                    }
                    // Flag for refresh if recipe was modified
                    if result.get("recipeId").is_some() || result.get("success") == Some(&serde_json::json!(true)) {
                        needs_refresh = true;
                    }
                }
                ChatEvent::ToolCallError { id, error } => {
                    for msg in chat.messages.iter_mut().rev() {
                        for tc in &mut msg.tool_calls {
                            if tc.id == id {
                                tc.error = Some(error.clone());
                                break;
                            }
                        }
                    }
                }
                ChatEvent::Done => {
                    chat.is_streaming = false;
                    self.chat_rx = None;
                    break;
                }
                ChatEvent::Error(msg) => {
                    chat.is_streaming = false;
                    chat.messages.push(ChatMessage {
                        role: ChatRole::Assistant,
                        content: format!("Error: {}", msg),
                        tool_calls: Vec::new(),
                    });
                    self.chat_rx = None;
                    break;
                }
            }
        }

        // Refresh TUI state after processing all events
        if needs_refresh {
            self.refresh_recipes();
            if let Screen::RecipeEdit { ref recipe_id } = self.screen {
                let recipe_id = recipe_id.clone();
                let recipe = {
                    let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
                    db::get_recipe(&*c, &recipe_id).ok().flatten()
                };
                if let Some(recipe) = recipe {
                    self.name_input = recipe.name.clone();
                    self.current_doc = Some(recipe);
                }
            }
        }
    }

    pub fn refresh_recipes(&mut self) {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        if self.show_deleted {
            if let Ok(rows) = db::list_all_recipes(&*conn) {
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
        } else {
            if let Ok(recipes) = db::list_recipes(&*conn) {
                self.recipes = recipes
                    .into_iter()
                    .map(|r| {
                        let style = r.recipe.style.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                        RecipeListItem {
                            id: r.id,
                            name: r.name,
                            style,
                            is_deleted: false,
                        }
                    })
                    .collect();
            }
        }
    }

    pub fn refresh_batches(&mut self) {
        if let Ok(batches) = batch::list_batches(&*self.conn.lock().unwrap_or_else(|e| e.into_inner())) {
            self.batches = batches
                .into_iter()
                .map(|b| {
                    BatchListItem {
                        id: b.id,
                        name: b.name,
                        recipe_name: b.data.recipe.name.clone(),
                        brew_date: format_epoch_millis(b.data.brew_date),
                    }
                })
                .collect();
        }
    }

    pub fn refresh_settings(&mut self) {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let existing_row = settings::get_settings(&*conn).ok().flatten();
        self.settings_doc = settings::get_settings_document(&*conn).unwrap_or_default();

        // Ensure settings are persisted with valid am_data (needed for sync)
        let needs_save = match existing_row {
            None => true,
            Some(ref row) => row.am_data.is_empty() || row.data == "{}",
        };
        if needs_save {
            let _ = settings::save_settings(&*conn, &self.settings_doc);
        }
    }

    pub fn save_settings(&mut self) {
        let _ = settings::save_settings(
            &*self.conn.lock().unwrap_or_else(|e| e.into_inner()),
            &self.settings_doc,
        );
    }

    pub fn create_recipe(&mut self) {
        let recipe = default_recipe();
        let result = db::create_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), "New Recipe", &recipe, None, None);
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
            let default_profile = brewdio_core::data::equipment()[0].clone();
            let equip = doc.equipment.clone().unwrap_or_else(|| {
                default_profile.equipment.clone()
            });
            let equipment_profile_id = doc.equipment_id.clone().unwrap_or_else(|| {
                default_profile.id.clone()
            });
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            let count = batch::count_batches_for_recipe(&*conn, recipe_id).unwrap_or(0);
            let name = format!("{} Batch {}", doc.name, count + 1);
            let result = batch::create_batch_from_recipe(
                &*conn,
                &name,
                recipe_id,
                &doc.recipe,
                &equip,
                &equipment_profile_id,
            );
            drop(conn);
            if let Ok(row) = result {
                let batch_id = row.id.clone();
                self.save_current();
                self.open_batch(&batch_id);
            }
        }
    }

    pub fn open_recipe(&mut self, id: &str) {
        let recipe = {
            let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            db::get_recipe(&*c, id).ok().flatten()
        };
        if let Some(recipe) = recipe {
            self.name_input = recipe.name.clone();
            self.current_doc = Some(recipe);
            self.history_entries.clear();
            self.history_scroll = 0;
            self.history_loading = false;
            self.history_rx = None;
            self.screen = Screen::RecipeEdit {
                recipe_id: id.to_string(),
            };
            self.editing_name = false;
            self.style_selector = None;
            self.equipment_selector = None;
            self.edit_focus = EditFocus::Name;
            self.active_tab = Tab::Fermentables;
            self.recipe_batch_list_index = 0;
            self.refresh_recipe_batches();
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
            let _ = db::update_recipe(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), recipe_id, &doc.name, &doc.recipe, doc.equipment.as_ref());
        }
        if self.active_tab == Tab::History {
            self.refresh_history();
        }
    }

    pub fn back_to_list(&mut self) {
        self.screen = Screen::Home;
        self.current_doc = None;
        self.history_entries.clear();
        self.history_loading = false;
        self.history_rx = None;
        self.editing_name = false;
        self.style_selector = None;
        self.equipment_selector = None;
        self.refresh_recipes();
        self.refresh_batches();
    }

    pub fn open_batch(&mut self, batch_id: &str) {
        if let Ok(Some(b)) =
            batch::get_batch(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()), batch_id)
        {
            // Build a virtual Recipe from the batch's recipe copy
            let doc = Recipe {
                id: batch_id.to_string(),
                name: b.name.clone(),
                recipe: b.data.recipe.clone(),
                equipment: Some(b.data.equipment.clone()),
                equipment_id: Some(b.data.equipment_id.clone()),
            };
            self.name_input = b.name.clone();
            self.brew_date_input = format_epoch_millis(b.data.brew_date);
            self.current_doc = Some(doc);
            self.history_entries.clear();
            self.history_scroll = 0;
            self.history_loading = false;
            self.history_rx = None;
            self.current_batch_id = Some(batch_id.to_string());
            self.batch_notes_text = b.data.notes.clone().unwrap_or_default();
            self.current_batch_data = Some(b.data);
            self.screen = Screen::BatchEdit {
                batch_id: batch_id.to_string(),
            };
            self.editing_name = false;
            self.editing_brew_date = false;
            self.style_selector = None;
            self.equipment_selector = None;
            self.edit_focus = EditFocus::Name;
            self.active_tab = Tab::Fermentables;
        }
    }

    pub fn save_current_batch(&mut self) {
        if let (
            Screen::BatchEdit { ref batch_id },
            Some(ref doc),
            Some(ref mut batch_data),
        ) = (&self.screen, &self.current_doc, &mut self.current_batch_data)
        {
            batch_data.recipe = doc.recipe.clone();
            batch_data.notes = if self.batch_notes_text.is_empty() {
                None
            } else {
                Some(self.batch_notes_text.clone())
            };
            batch_data.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let data_json = serde_json::to_string(&batch_data).unwrap_or_default();
            let _ = batch::update_batch(
                &*self.conn.lock().unwrap_or_else(|e| e.into_inner()),
                batch_id,
                &doc.name,
                &data_json,
            );
        }
        if self.active_tab == Tab::History {
            self.refresh_history();
        }
    }

    pub fn back_from_batch(&mut self) {
        self.screen = Screen::Home;
        self.current_doc = None;
        self.history_entries.clear();
        self.history_loading = false;
        self.history_rx = None;
        self.current_batch_id = None;
        self.current_batch_data = None;
        self.editing_name = false;
        self.style_selector = None;
        self.equipment_selector = None;
        self.refresh_recipes();
        self.refresh_batches();
    }

    pub fn open_recipe_from_batch(&mut self) {
        let recipe_id = self.current_batch_id.as_ref().and_then(|bid| {
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            batch::get_batch(&*conn, bid).ok().flatten().map(|b| b.recipe_id)
        });
        if let Some(rid) = recipe_id {
            self.save_current_batch();
            self.current_doc = None;
            self.current_batch_id = None;
            self.current_batch_data = None;
            self.editing_name = false;
            self.style_selector = None;
            self.equipment_selector = None;
            self.open_recipe(&rid);
        }
    }

    pub fn refresh_recipe_batches(&mut self) {
        if let Screen::RecipeEdit { ref recipe_id } = self.screen {
            let recipe_id = recipe_id.clone();
            if let Ok(batches) =
                batch::list_batches(&*self.conn.lock().unwrap_or_else(|e| e.into_inner()))
            {
                self.recipe_batches = batches
                    .into_iter()
                    .filter(|b| b.recipe_id == recipe_id)
                    .map(|b| {
                        BatchListItem {
                            id: b.id,
                            name: b.name,
                            recipe_name: b.data.recipe.name.clone(),
                            brew_date: format_epoch_millis(b.data.brew_date),
                        }
                    })
                    .collect();
            }
        }
    }

    pub fn open_notes_editor(&mut self, target: NotesTarget) {
        let text = match target {
            NotesTarget::Recipe => self
                .current_doc
                .as_ref()
                .and_then(|d| d.recipe.notes.clone())
                .unwrap_or_default(),
            NotesTarget::Batch => self.batch_notes_text.clone(),
        };
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };
        let mut ta = TextArea::new(lines);
        ta.set_style(Style::default().bg(Color::Black));
        ta.set_cursor_line_style(Style::default().bg(Color::Black));
        ta.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
        self.notes_editor = Some(ta);
        self.notes_target = target;
    }

    pub fn save_notes(&mut self) {
        if let Some(editor) = self.notes_editor.take() {
            let text = editor.lines().join("\n");
            match self.notes_target {
                NotesTarget::Recipe => {
                    if let Some(ref mut doc) = self.current_doc {
                        doc.recipe.notes = if text.is_empty() {
                            None
                        } else {
                            Some(text)
                        };
                    }
                    self.save_current();
                }
                NotesTarget::Batch => {
                    self.batch_notes_text = text;
                    self.save_current_batch();
                }
            }
        }
    }

    pub fn cancel_notes(&mut self) {
        self.notes_editor = None;
    }

    pub fn start_edit_brew_date(&mut self) {
        self.editing_brew_date = true;
    }

    pub fn confirm_brew_date(&mut self) {
        if let Some(millis) = parse_date_to_epoch_millis(&self.brew_date_input) {
            if let Some(ref mut data) = self.current_batch_data {
                data.brew_date = millis;
            }
            self.editing_brew_date = false;
            self.save_current_batch();
        }
        // If parse fails, stay in editing mode
    }

    pub fn cancel_brew_date(&mut self) {
        if let Some(ref data) = self.current_batch_data {
            self.brew_date_input = format_epoch_millis(data.brew_date);
        }
        self.editing_brew_date = false;
    }

    pub fn confirm_name(&mut self) {
        if let Some(ref mut doc) = self.current_doc {
            doc.name = self.name_input.clone();
            doc.recipe.name = self.name_input.clone();
        }
        self.editing_name = false;
        match self.screen {
            Screen::BatchEdit { .. } => self.save_current_batch(),
            _ => self.save_current(),
        }
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

    pub fn equipment_name(&self) -> String {
        self.current_doc
            .as_ref()
            .and_then(|d| d.equipment.as_ref())
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "(none)".to_string())
    }

    pub fn open_equipment_selector(&mut self) {
        let all = brewdio_core::data::equipment();
        // TODO: also load custom profiles from DB and combine
        let items: Vec<SearchItem> = all
            .iter()
            .enumerate()
            .map(|(i, p)| SearchItem {
                label: p.name().to_string(),
                detail: format!("{:.0}% eff, {}", p.efficiency.brewhouse.value, p.equipment.equipment_items.iter().map(|item| item.name.clone()).collect::<Vec<_>>().join(", ")),
                index: i,
            })
            .collect();
        let mut selector = SearchSelector::new("Select Equipment", items);
        // Pre-position cursor on current equipment
        if let Some(ref doc) = self.current_doc {
            if let Some(ref eq) = doc.equipment {
                if let Some(pos) = all.iter().position(|p| p.equipment.name == eq.name) {
                    selector.set_cursor_to_index(pos);
                }
            }
        }
        self.equipment_selector = Some(selector);
    }

    pub fn confirm_equipment(&mut self, idx: usize) {
        self.equipment_selector = None;
        self.confirm_equipment_idx = Some(idx);
    }

    pub fn confirm_equipment_yes(&mut self) {
        if let Some(idx) = self.confirm_equipment_idx.take() {
            let all = brewdio_core::data::equipment();
            let profile = &all[idx];
            if let Some(ref mut doc) = self.current_doc {
                doc.equipment = Some(profile.equipment.clone());
                doc.equipment_id = Some(profile.id.clone());
                // Copy efficiency into recipe
                doc.recipe.efficiency = profile.efficiency.clone();
            }
            match self.screen {
                Screen::RecipeEdit { ref recipe_id } => {
                    let _ = db::set_recipe_equipment(
                        &*self.conn.lock().unwrap_or_else(|e| e.into_inner()),
                        recipe_id,
                        Some(&profile.equipment),
                        Some(&profile.id),
                    );
                    self.save_current();
                }
                Screen::BatchEdit { .. } => {
                    // Update batch data's equipment too
                    if let Some(ref mut batch_data) = self.current_batch_data {
                        batch_data.equipment = profile.equipment.clone();
                        batch_data.equipment_id = profile.id.clone();
                    }
                    self.save_current_batch();
                }
                _ => {}
            }
        }
    }

    pub fn confirm_equipment_no(&mut self) {
        self.confirm_equipment_idx = None;
    }

    pub fn cancel_equipment(&mut self) {
        self.equipment_selector = None;
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

    pub fn set_tab(&mut self, n: usize) {
        if n < ALL_TABS.len() {
            self.active_tab = ALL_TABS[n];
            if self.active_tab == Tab::Batches {
                self.refresh_recipe_batches();
            }
            if self.active_tab == Tab::History {
                self.refresh_history();
            }
        }
    }

    /// Kick off an async history refresh on a background thread.
    /// Results are picked up by `poll_history()` in the event loop.
    pub fn refresh_history(&mut self) {
        // Read am_data from DB (fast) on main thread, then spawn computation
        let am_data = match &self.screen {
            Screen::RecipeEdit { ref recipe_id } => {
                let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
                db::get_recipe_row(&*c, recipe_id)
                    .ok()
                    .flatten()
                    .map(|row| row.am_data)
            }
            Screen::BatchEdit { ref batch_id } => {
                let c = self.conn.lock().unwrap_or_else(|e| e.into_inner());
                batch::get_batch_row(&*c, batch_id)
                    .ok()
                    .flatten()
                    .map(|row| row.am_data)
            }
            _ => None,
        };

        let Some(am_data) = am_data else { return };
        let is_recipe = matches!(self.screen, Screen::RecipeEdit { .. });

        let (tx, rx) = mpsc::channel();
        self.history_rx = Some(rx);
        self.history_loading = true;

        std::thread::spawn(move || {
            let entries = if is_recipe {
                brewdio_persistence::automerge::get_recipe_history(&am_data)
            } else {
                brewdio_persistence::automerge::get_batch_history(&am_data)
            };
            let _ = tx.send(entries);
        });
    }

    /// Non-blocking check for completed history computation.
    /// Call this from the event loop.
    pub fn poll_history(&mut self) {
        if let Some(ref rx) = self.history_rx {
            if let Ok(entries) = rx.try_recv() {
                self.history_entries = entries;
                self.history_scroll = 0;
                self.history_loading = false;
                self.history_rx = None;
            }
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

    pub fn compute_water(&self) -> Option<WaterCalculatorResult> {
        let doc = self.current_doc.as_ref()?;
        let recipe = &doc.recipe;

        let vol_unit = recipe.batch_size.unit.clone();
        let temp_unit = match vol_unit {
            VolumeUnitType::Gal => TemperatureUnitType::F,
            _ => TemperatureUnitType::C,
        };

        let input = WaterCalculatorInput {
            target_batch_size: recipe.batch_size.clone(),
            boil_time: recipe
                .boil
                .as_ref()
                .map(|b| b.boil_time.clone())
                .unwrap_or(TimeType {
                    value: 60,
                    unit: TimeUnitType::Min,
                }),
            grain_bill: recipe.ingredients.fermentable_additions.clone(),
            mash_steps: recipe
                .mash
                .as_ref()
                .map(|m| m.mash_steps.clone())
                .unwrap_or_default(),
            grain_temperature: recipe
                .mash
                .as_ref()
                .map(|m| m.grain_temperature.clone())
                .unwrap_or(TemperatureType {
                    value: 20.0,
                    unit: TemperatureUnitType::C,
                }),
            equipment: doc.equipment.clone(),
            units: Some(WaterCalculatorOptions {
                volume_unit: vol_unit,
                temperature_unit: temp_unit,
            }),
        };

        Some(brewdio_core::water::calculate_water(&input))
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

fn parse_date_to_epoch_millis(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: usize = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    let mut days: i64 = 0;
    for y in 1970..year {
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        days += if leap { 366 } else { 365 };
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    for m in 0..(month - 1) {
        days += month_days[m] as i64;
    }
    days += day - 1;
    Some((days as u64) * 86400 * 1000)
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
