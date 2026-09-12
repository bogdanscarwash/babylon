//! Shared admitted material foundation for durable database integration tests.

pub fn foundation() -> super::material_runtime::MaterialRuntimeFoundation {
    let catalog = super::michigan_material::MichiganMaterialCatalog::from_defines_toml(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        )),
    )
    .expect("current authored material parameters");
    super::michigan_content::MichiganContentPreset::new_campaign(
        super::michigan_material::MichiganDeliveryPreset::Standard,
    )
    .create_foundation(&catalog)
    .expect("current material foundation")
}
