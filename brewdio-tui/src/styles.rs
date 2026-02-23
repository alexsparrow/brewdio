use brewdio_core::beerjson_types::{StyleBase, StyleCategories};

pub struct BeerStyle {
    pub name: &'static str,
    pub category: &'static str,
}

pub const BEER_STYLES: &[BeerStyle] = &[
    BeerStyle { name: "American Light Lager", category: "1A" },
    BeerStyle { name: "American Lager", category: "1B" },
    BeerStyle { name: "American Wheat Beer", category: "1D" },
    BeerStyle { name: "German Pilsner", category: "5D" },
    BeerStyle { name: "Czech Premium Pale Lager", category: "3B" },
    BeerStyle { name: "Munich Helles", category: "4A" },
    BeerStyle { name: "Vienna Lager", category: "7A" },
    BeerStyle { name: "Märzen", category: "6A" },
    BeerStyle { name: "Munich Dunkel", category: "8A" },
    BeerStyle { name: "Schwarzbier", category: "8B" },
    BeerStyle { name: "Doppelbock", category: "9A" },
    BeerStyle { name: "Weissbier", category: "10A" },
    BeerStyle { name: "Witbier", category: "24A" },
    BeerStyle { name: "Belgian Pale Ale", category: "24B" },
    BeerStyle { name: "Saison", category: "25B" },
    BeerStyle { name: "Belgian Tripel", category: "26C" },
    BeerStyle { name: "British Bitter", category: "11A" },
    BeerStyle { name: "English IPA", category: "12C" },
    BeerStyle { name: "Irish Stout", category: "15B" },
    BeerStyle { name: "American Pale Ale", category: "18B" },
    BeerStyle { name: "American IPA", category: "21A" },
    BeerStyle { name: "New England IPA", category: "21B" },
    BeerStyle { name: "Double IPA", category: "22A" },
    BeerStyle { name: "American Amber Ale", category: "19A" },
    BeerStyle { name: "American Brown Ale", category: "19C" },
    BeerStyle { name: "American Porter", category: "20A" },
    BeerStyle { name: "American Stout", category: "20B" },
    BeerStyle { name: "Imperial Stout", category: "20C" },
];

pub fn to_style_base(style: &BeerStyle) -> StyleBase {
    StyleBase {
        name: style.name.to_string(),
        category: style.category.to_string(),
        style_guide: "BJCP 2021".to_string(),
        type_: StyleCategories::Beer,
        category_number: None,
        style_letter: None,
    }
}
