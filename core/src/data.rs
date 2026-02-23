use crate::beerjson_types::{CultureInformation, EquipmentType, FermentableType, HopVarietyBase, MashProcedureType, StyleType};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fermentables_deserialize() {
        let f = fermentables();
        assert_eq!(f.len(), 248);
    }

    #[test]
    fn test_hops_deserialize() {
        let h = hops();
        assert_eq!(h.len(), 62);
    }

    #[test]
    fn test_cultures_deserialize() {
        let c = cultures();
        assert_eq!(c.len(), 134);
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
}
