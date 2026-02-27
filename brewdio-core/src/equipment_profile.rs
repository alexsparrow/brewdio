use crate::beerjson_types::{EfficiencyType, EquipmentType, PercentType, PercentUnitType};

crate::brewing_model! {
    pub struct EquipmentProfile {
        pub id: String,
        pub equipment: EquipmentType,
        pub efficiency: EfficiencyType,
    }
}

impl EquipmentProfile {
    pub fn name(&self) -> &str {
        &self.equipment.name
    }

    pub fn from_equipment(id: &str, equipment: EquipmentType) -> Self {
        Self {
            id: id.to_string(),
            equipment,
            efficiency: EfficiencyType {
                brewhouse: PercentType {
                    unit: PercentUnitType::X,
                    value: 72.0,
                },
                conversion: None,
                lauter: None,
                mash: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let profile = EquipmentProfile {
            id: "test-id".to_string(),
            equipment: EquipmentType {
                name: "Test Setup".to_string(),
                equipment_items: vec![],
            },
            efficiency: EfficiencyType {
                brewhouse: PercentType {
                    unit: PercentUnitType::X,
                    value: 75.0,
                },
                conversion: None,
                lauter: None,
                mash: None,
            },
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: EquipmentProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.name(), "Test Setup");
        assert_eq!(deserialized.efficiency.brewhouse.value, 75.0);
    }

    #[test]
    fn from_equipment_defaults() {
        let equipment = EquipmentType {
            name: "My Rig".to_string(),
            equipment_items: vec![],
        };
        let profile = EquipmentProfile::from_equipment("my-rig", equipment);
        assert_eq!(profile.id, "my-rig");
        assert_eq!(profile.name(), "My Rig");
        assert_eq!(profile.efficiency.brewhouse.value, 72.0);
    }
}
