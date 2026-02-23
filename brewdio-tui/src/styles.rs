use brewdio_core::beerjson_types::{StyleBase, StyleCategories, StyleType};

pub fn all_styles() -> Vec<StyleType> {
    brewdio_core::data::styles()
}

pub fn to_style_base(style: &StyleType) -> StyleBase {
    StyleBase {
        name: style.name.clone(),
        category: style.category.clone(),
        style_guide: style.style_guide.clone(),
        type_: StyleCategories::Beer,
        category_number: style.category_number,
        style_letter: None,
    }
}
