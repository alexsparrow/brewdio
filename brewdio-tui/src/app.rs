use brewdio_core::beerjson_types::{
    EfficiencyType, IngredientsType, PercentType, PercentUnitType, RecipeStyleType, RecipeType,
    RecipeTypeType, VolumeType, VolumeUnitType,
};
use persistence::db;
use persistence::recipe::RecipeDocument;
use rusqlite::Connection;

use crate::styles::{self, BEER_STYLES};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    RecipeList,
    RecipeEdit { recipe_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub struct RecipeListItem {
    pub id: String,
    pub name: String,
}

pub struct App {
    pub conn: Connection,
    pub screen: Screen,
    // Recipe list state
    pub recipes: Vec<RecipeListItem>,
    pub list_index: usize,
    // Recipe edit state
    pub current_doc: Option<RecipeDocument>,
    pub edit_focus: EditFocus,
    pub active_tab: Tab,
    pub name_input: String,
    pub editing_name: bool,
    pub style_index: usize,
    pub editing_style: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(conn: Connection) -> Self {
        let mut app = App {
            conn,
            screen: Screen::RecipeList,
            recipes: Vec::new(),
            list_index: 0,
            current_doc: None,
            edit_focus: EditFocus::Name,
            active_tab: Tab::Fermentables,
            name_input: String::new(),
            editing_name: false,
            style_index: 0,
            editing_style: false,
            should_quit: false,
        };
        app.refresh_recipes();
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

    pub fn open_recipe(&mut self, id: &str) {
        if let Ok(Some(row)) = db::get_recipe(&self.conn, id) {
            if let Ok(doc) = row.to_document() {
                self.name_input = doc.name.clone();
                self.style_index = doc
                    .recipe
                    .style
                    .as_ref()
                    .and_then(|s| {
                        BEER_STYLES
                            .iter()
                            .position(|bs| bs.name == s.0.name)
                    })
                    .unwrap_or(0);
                self.current_doc = Some(doc);
                self.screen = Screen::RecipeEdit {
                    recipe_id: id.to_string(),
                };
                self.editing_name = false;
                self.editing_style = false;
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
        self.screen = Screen::RecipeList;
        self.current_doc = None;
        self.editing_name = false;
        self.editing_style = false;
        self.refresh_recipes();
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

    pub fn confirm_style(&mut self) {
        if let Some(ref mut doc) = self.current_doc {
            let style = &BEER_STYLES[self.style_index];
            doc.recipe.style = Some(RecipeStyleType(styles::to_style_base(style)));
        }
        self.editing_style = false;
        self.save_current();
    }

    pub fn cancel_style(&mut self) {
        self.editing_style = false;
    }

    pub fn set_tab(&mut self, n: usize) {
        if n < ALL_TABS.len() {
            self.active_tab = ALL_TABS[n];
        }
    }

    pub fn style_name(&self) -> String {
        self.current_doc
            .as_ref()
            .and_then(|d| d.recipe.style.as_ref())
            .map(|s| s.0.name.clone())
            .unwrap_or_else(|| "(none)".to_string())
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
