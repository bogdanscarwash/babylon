//! Shared admitted material foundation for durable database integration tests.

pub fn foundation() -> babylon_persistence::material_runtime::MaterialRuntimeFoundation {
    let catalog =
        babylon_persistence::michigan_material::MichiganMaterialCatalog::from_defines_toml(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../content/scenarios/michigan/defines.toml"
            )),
        )
        .expect("current authored material parameters");
    babylon_persistence::michigan_content::MichiganContentPreset::new_campaign(
        babylon_persistence::michigan_material::MichiganDeliveryPreset::Standard,
    )
    .create_foundation(&catalog)
    .expect("current material foundation")
}
