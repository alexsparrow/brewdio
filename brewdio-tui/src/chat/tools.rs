use brewdio_core::beerjson_types::RecipeType;
use brewdio_persistence::db;
use rusqlite::Connection;
use serde_json::{json, Value as JsonValue};

/// Returns OpenAI function-calling tool definitions.
/// When `has_recipe_context` is true, includes getCurrentRecipe and updateCurrentRecipe.
pub fn tool_definitions(has_recipe_context: bool) -> Vec<JsonValue> {
    let mut tools = vec![
        json!({
            "type": "function",
            "function": {
                "name": "searchFermentables",
                "description": "Search for fermentables (grains, malts, sugars) usable in a beer recipe. Returns fermentables matching the search term.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "search": {
                            "type": "string",
                            "description": "Search term to filter fermentables by name"
                        }
                    },
                    "required": ["search"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "searchHops",
                "description": "Search for hops usable in a beer recipe. Returns hops with their alpha acid, beta acid, and origin information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "search": {
                            "type": "string",
                            "description": "Search term to filter hops by name"
                        }
                    },
                    "required": ["search"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "searchCultures",
                "description": "Search for yeast cultures (ale, lager, etc) usable in a beer recipe. Returns cultures with their type, form, attenuation, and producer information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "search": {
                            "type": "string",
                            "description": "Search term to filter cultures by name or producer"
                        }
                    },
                    "required": ["search"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "listStyles",
                "description": "List all available beer styles that can be used when creating a recipe.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "createRecipe",
                "description": "Create a NEW beer recipe from scratch. This creates an entirely new recipe in the database. DO NOT use this to modify an existing recipe - use updateCurrentRecipe instead when in a recipe context.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the new recipe"
                        },
                        "batchSize": {
                            "type": "number",
                            "description": "Batch size in gallons"
                        },
                        "styleName": {
                            "type": "string",
                            "description": "Name of the beer style (must match exactly from listStyles)"
                        }
                    },
                    "required": ["name", "batchSize", "styleName"]
                }
            }
        }),
    ];

    if has_recipe_context {
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "getCurrentRecipe",
                "description": "Get the CURRENT recipe that the user is viewing. Use this to read the recipe data before making modifications. Returns the full BeerJSON recipe data including all ingredients.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        }));
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "updateCurrentRecipe",
                "description": "Update the CURRENT recipe that the user is viewing. Use this to modify the recipe by providing the complete updated BeerJSON data. IMPORTANT: Always call getCurrentRecipe first to get the existing recipe, then modify it, then pass the complete modified recipe to this tool.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "recipe": {
                            "type": "object",
                            "description": "Complete BeerJSON RecipeType object with ALL fields including your modifications."
                        }
                    },
                    "required": ["recipe"]
                }
            }
        }));
    }

    tools
}

/// Execute a tool call and return the result as JSON.
pub fn execute_tool(
    name: &str,
    arguments: &JsonValue,
    conn: &Connection,
    recipe_context: &mut Option<(String, RecipeType)>,
) -> Result<JsonValue, String> {
    match name {
        "searchFermentables" => {
            let search = arguments
                .get("search")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let all = brewdio_core::data::fermentables();
            let results: Vec<JsonValue> = all
                .iter()
                .filter(|f| f.name.to_lowercase().contains(&search))
                .take(20)
                .map(|f| {
                    json!({
                        "name": f.name,
                        "type": format!("{:?}", f.type_),
                        "yield": f.yield_.fine_grind.as_ref().map(|fg| fg.value),
                        "color": f.color.value,
                    })
                })
                .collect();
            Ok(json!(results))
        }
        "searchHops" => {
            let search = arguments
                .get("search")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let all = brewdio_core::data::hops();
            let results: Vec<JsonValue> = all
                .iter()
                .filter(|h| h.name.to_lowercase().contains(&search))
                .take(20)
                .map(|h| {
                    json!({
                        "name": h.name,
                        "alpha_acid": h.alpha_acid.value,
                        "beta_acid": h.beta_acid.as_ref().map(|b| b.value),
                        "origin": h.origin,
                    })
                })
                .collect();
            Ok(json!(results))
        }
        "searchCultures" => {
            let search = arguments
                .get("search")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let all = brewdio_core::data::cultures();
            let results: Vec<JsonValue> = all
                .iter()
                .filter(|c| {
                    c.name.to_lowercase().contains(&search)
                        || c.producer
                            .as_ref()
                            .map_or(false, |p| p.to_lowercase().contains(&search))
                })
                .take(20)
                .map(|c| {
                    json!({
                        "name": c.name,
                        "type": format!("{:?}", c.type_),
                        "form": format!("{:?}", c.form),
                        "producer": c.producer,
                        "attenuation": c.attenuation_range.as_ref().map(|a| {
                            json!({
                                "min": a.minimum.value,
                                "max": a.maximum.value,
                            })
                        }),
                    })
                })
                .collect();
            Ok(json!(results))
        }
        "listStyles" => {
            let styles = brewdio_core::data::styles();
            let results: Vec<JsonValue> = styles
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "category": s.category,
                    })
                })
                .collect();
            Ok(json!(results))
        }
        "createRecipe" => {
            let name = arguments
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'name' parameter")?;
            let batch_size = arguments
                .get("batchSize")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0);
            let style_name = arguments
                .get("styleName")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Find the style
            let styles = brewdio_core::data::styles();
            let style = styles
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(style_name));

            use brewdio_core::beerjson_types::*;

            let recipe_style = style.map(|s| {
                RecipeStyleType(StyleBase {
                    name: s.name.clone(),
                    category: s.category.clone(),
                    category_number: s.category_number,
                    style_letter: s.style_letter.as_ref().and_then(|sl| {
                        sl.to_string().parse::<StyleBaseStyleLetter>().ok()
                    }),
                    style_guide: s.style_guide.clone(),
                    type_: s.type_.clone(),
                })
            });

            let recipe = RecipeType {
                name: name.to_string(),
                type_: RecipeTypeType::AllGrain,
                author: String::new(),
                batch_size: VolumeType {
                    value: batch_size,
                    unit: VolumeUnitType::Gal,
                },
                efficiency: EfficiencyType {
                    brewhouse: PercentType {
                        value: 72.0,
                        unit: PercentUnitType::X,
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
                style: recipe_style,
                boil: None,
                mash: None,
                fermentation: None,
                notes: None,
                original_gravity: None,
                final_gravity: None,
                alcohol_by_volume: None,
                ibu_estimate: None,
                color_estimate: None,
                beer_p_h: None,
                carbonation: None,
                apparent_attenuation: None,
                calories_per_pint: None,
                created: None,
                coauthor: None,
                packaging: None,
                taste: None,
            };

            let row = db::create_recipe(conn, name, &recipe, None, None)
                .map_err(|e| format!("Failed to create recipe: {:?}", e))?;

            Ok(json!({
                "success": true,
                "recipeId": row.id,
                "message": format!("Created recipe '{}'", name),
            }))
        }
        "getCurrentRecipe" => {
            let (_, recipe) = recipe_context
                .as_ref()
                .ok_or("No recipe context available")?;
            Ok(json!({
                "success": true,
                "recipe": serde_json::to_value(recipe).map_err(|e| e.to_string())?,
            }))
        }
        "updateCurrentRecipe" => {
            let recipe_json = arguments
                .get("recipe")
                .ok_or("Missing 'recipe' parameter")?;
            let updated_recipe: RecipeType =
                serde_json::from_value(recipe_json.clone()).map_err(|e| {
                    format!("Invalid recipe data: {}", e)
                })?;

            let (recipe_id, _) = recipe_context
                .as_ref()
                .ok_or("No recipe context available")?;
            let recipe_id = recipe_id.clone();

            db::update_recipe(conn, &recipe_id, &updated_recipe.name, &updated_recipe, None)
                .map_err(|e| format!("Failed to update recipe: {:?}", e))?;

            // Update the context with the new recipe
            *recipe_context = Some((recipe_id, updated_recipe));

            Ok(json!({
                "success": true,
                "message": "Recipe updated successfully",
            }))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}
