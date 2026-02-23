use crate::beerjson_types::{CultureInformation, EquipmentType, FermentableType, HopVarietyBase, MashProcedureType, RecipeType, StyleType};

pub fn fermentables() -> Vec<FermentableType> {
    serde_json::from_str(include_str!("data/fermentables.json"))
        .expect("Failed to deserialize fermentables.json")
}

pub fn hops() -> Vec<HopVarietyBase> {
    serde_json::from_str(include_str!("data/hops.json"))
        .expect("Failed to deserialize hops.json")
}

pub fn cultures() -> Vec<CultureInformation> {
    serde_json::from_str(include_str!("data/cultures.json"))
        .expect("Failed to deserialize cultures.json")
}

pub fn styles() -> Vec<StyleType> {
    serde_json::from_str(include_str!("data/styles.json"))
        .expect("Failed to deserialize styles.json")
}

pub fn equipment() -> Vec<EquipmentType> {
    serde_json::from_str(include_str!("data/equipment.json"))
        .expect("Failed to deserialize equipment.json")
}

pub fn mash_profiles() -> Vec<MashProcedureType> {
    serde_json::from_str(include_str!("data/mash_profiles.json"))
        .expect("Failed to deserialize mash_profiles.json")
}

pub fn style_for_recipe(recipe: &RecipeType) -> Option<StyleType> {
    let style_name = recipe.style.as_ref().map(|s| &s.name)?;
    styles().into_iter().find(|s| &s.name == style_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fermentables_deserialize() {
        let f = fermentables();
        assert_eq!(f.len(), 249);
    }

    #[test]
    fn test_hops_deserialize() {
        let h = hops();
        assert_eq!(h.len(), 62);
    }

    #[test]
    fn test_cultures_deserialize() {
        let c = cultures();
        assert_eq!(c.len(), 135);
    }

    #[test]
    fn test_styles_deserialize() {
        let s = styles();
        assert_eq!(s.len(), 98);
    }

    #[test]
    fn test_equipment_deserialize() {
        let e = equipment();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].name, "Default Setup");
    }

    #[test]
    fn test_mash_profiles_deserialize() {
        let m = mash_profiles();
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].name, "Single Infusion (F)");
        assert_eq!(m[1].name, "Single Infusion (C)");
    }

    #[test]
    fn test_style_names_are_unique() {
        let s = styles();
        let mut names: Vec<&str> = s.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        for window in names.windows(2) {
            assert_ne!(window[0], window[1], "Duplicate style name: {}", window[0]);
        }
    }

    #[test]
    fn test_style_for_recipe() {
        use crate::beerjson_types::*;

        // Recipe with a known style
        let recipe = RecipeType {
            name: "Test".to_string(),
            author: String::new(),
            type_: RecipeTypeType::AllGrain,
            batch_size: VolumeType { unit: VolumeUnitType::L, value: 20.0 },
            efficiency: EfficiencyType {
                brewhouse: PercentType { unit: PercentUnitType::X, value: 72.0 },
                conversion: None, lauter: None, mash: None,
            },
            ingredients: IngredientsType {
                fermentable_additions: Vec::new(),
                hop_additions: Vec::new(),
                culture_additions: Vec::new(),
                miscellaneous_additions: Vec::new(),
                water_additions: Vec::new(),
            },
            style: Some(RecipeStyleType(StyleBase {
                name: "American IPA".to_string(),
                category: "IPA".to_string(),
                style_guide: "BJCP 2021".to_string(),
                type_: StyleCategories::Beer,
                category_number: None,
                style_letter: None,
            })),
            alcohol_by_volume: None, apparent_attenuation: None,
            beer_p_h: None, boil: None, calories_per_pint: None,
            carbonation: None, coauthor: None, color_estimate: None,
            created: None, fermentation: None, final_gravity: None,
            ibu_estimate: None, mash: None, notes: None,
            original_gravity: None, packaging: None, taste: None,
        };

        let found = style_for_recipe(&recipe);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "American IPA");

        // Recipe with no style
        let mut no_style = recipe.clone();
        no_style.style = None;
        assert!(style_for_recipe(&no_style).is_none());

        // Recipe with unknown style
        let mut unknown = recipe.clone();
        unknown.style = Some(RecipeStyleType(StyleBase {
            name: "Nonexistent Style XYZ".to_string(),
            category: String::new(),
            style_guide: String::new(),
            type_: StyleCategories::Beer,
            category_number: None,
            style_letter: None,
        }));
        assert!(style_for_recipe(&unknown).is_none());
    }
}
